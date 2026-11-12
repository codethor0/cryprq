// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures::future::Either;
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId};
use node::{
    FileMetadata, NodeIdentity, PeerPublicKey, TrustedPeers, TunConfig, TunInterface, Tunnel,
    TunnelConfig, TunnelRole,
};
use p2p::secure_vpn::{run_secure_vpn, PacketIo, VpnMode, VpnSettings};
use p2p::{
    dial_peer, register_packet_recv_tx, send_file_to_peer, set_file_transfer_callback,
    start_key_rotation, start_listener, start_metrics_server, DataChunk, Libp2pPacketForwarder,
    CHUNK_SIZE,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::{env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

#[derive(Parser, Debug)]
#[command(
    name = "cryprq",
    version,
    about = "CrypRQ research prototype: authenticated hybrid (ML-KEM-768 + X25519) point-to-point VPN link and file transfer"
)]
struct Args {
    #[arg(
        long,
        help = "WARNING: Use test-mode keys (insecure, for testing only)"
    )]
    test_mode: bool,

    #[command(subcommand)]
    command: Option<Command>,

    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,

    #[arg(
        long,
        help = "Enable VPN mode: an authenticated point-to-point link on a TUN device. Installs no routes and no DNS settings"
    )]
    vpn: bool,

    #[arg(long, default_value = "cryprq0", help = "TUN interface name")]
    tun_name: String,

    #[arg(long, default_value = "10.0.0.1", help = "TUN interface IP address")]
    tun_address: String,

    #[arg(
        long,
        default_value = "255.255.255.0",
        help = "TUN interface netmask; a point-to-point link needs no more than 255.255.255.252 (/30)"
    )]
    tun_netmask: String,

    #[arg(long, help = "Metrics server address")]
    metrics: Option<SocketAddr>,

    #[arg(
        long,
        value_name = "KEY",
        help = "With --vpn --peer: the pinned public key (ed25519:<hex>) of the peer to dial; optional when exactly one peer is pinned"
    )]
    peer_key: Option<String>,

    #[arg(
        long,
        value_name = "ADDR",
        help = "With --vpn: exchange IP packets with a loopback UDP socket at ADDR instead of a TUN device (userspace network stacks, tests without root)"
    )]
    packet_bridge: Option<SocketAddr>,

    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Identity key file [default: ~/.cryprq/identity.key]"
    )]
    identity: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "File of pinned peer keys, one ed25519:<hex> per line [default: ~/.cryprq/trusted_peers]"
    )]
    trusted_peers: Option<PathBuf>,
}

/// Where the node's identity and its pinned peers are stored.
struct TrustPaths {
    identity: PathBuf,
    trusted_peers: PathBuf,
}

impl TrustPaths {
    fn resolve(identity: Option<PathBuf>, trusted_peers: Option<PathBuf>) -> Result<Self> {
        let default = |name: &str| -> Result<PathBuf> {
            let home = env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("HOME is not set; pass --identity and --trusted-peers")
                })?;
            Ok(PathBuf::from(home).join(".cryprq").join(name))
        };
        Ok(Self {
            identity: match identity {
                Some(path) => path,
                None => default("identity.key")?,
            },
            trusted_peers: match trusted_peers {
                Some(path) => path,
                None => default("trusted_peers")?,
            },
        })
    }

    /// Loads the identity and the pinned peers. Nothing is generated and
    /// nobody is trusted implicitly: a missing file is an error.
    fn load(&self) -> Result<TunnelConfig> {
        let identity = NodeIdentity::load(&self.identity).with_context(|| {
            format!(
                "cannot load the node identity from {}; create one with `cryprq identity init`",
                self.identity.display()
            )
        })?;
        let trusted = TrustedPeers::load(&self.trusted_peers).with_context(|| {
            format!(
                "cannot load pinned peers from {}",
                self.trusted_peers.display()
            )
        })?;
        if trusted.is_empty() {
            anyhow::bail!(
                "{} pins no peers; add the peer's `ed25519:<hex>` key",
                self.trusted_peers.display()
            );
        }
        log::info!(
            "event=identity_loaded public_key={} pinned_peers={}",
            identity.public_key(),
            trusted.len()
        );
        Ok(TunnelConfig::new(Arc::new(identity), trusted))
    }
}

#[derive(Subcommand, Debug)]
enum IdentityCommand {
    /// Create this node's identity key. Never replaces an existing key.
    Init,
    /// Print this node's public key, to be pinned by its peers.
    Show,
}

fn handle_identity(action: IdentityCommand, paths: &TrustPaths) -> Result<()> {
    let identity = match action {
        IdentityCommand::Init => NodeIdentity::generate_new(&paths.identity)
            .with_context(|| format!("cannot create {}", paths.identity.display()))?,
        IdentityCommand::Show => NodeIdentity::load(&paths.identity)
            .with_context(|| format!("cannot load {}", paths.identity.display()))?,
    };
    println!("{}", identity.public_key());
    Ok(())
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Send a file to a peer
    SendFile {
        /// Peer address (multiaddr)
        #[arg(long)]
        peer: String,
        /// File path to send
        #[arg(long)]
        file: PathBuf,
        /// The receiver's pinned public key (ed25519:<hex>). May be omitted
        /// when exactly one peer is pinned.
        #[arg(long, value_name = "KEY")]
        peer_key: Option<String>,
    },
    /// Receive a file from a peer (listener mode)
    ReceiveFile {
        /// Address to listen on (multiaddr)
        #[arg(long)]
        listen: String,
        /// Output directory for received files
        #[arg(long, default_value = ".")]
        output_dir: PathBuf,
    },
    /// Manage this node's long-term identity
    Identity {
        #[command(subcommand)]
        action: IdentityCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    // Handle file transfer subcommands
    if let Some(command) = args.command {
        match command {
            Command::SendFile {
                peer,
                file,
                peer_key,
            } => {
                let paths = TrustPaths::resolve(args.identity, args.trusted_peers);
                return handle_send_file(peer, file, peer_key, args.test_mode, paths).await;
            }
            Command::ReceiveFile { listen, output_dir } => {
                let paths = TrustPaths::resolve(args.identity, args.trusted_peers);
                return handle_receive_file(listen, output_dir, args.test_mode, paths).await;
            }
            Command::Identity { action } => {
                let paths = TrustPaths::resolve(args.identity, args.trusted_peers)?;
                return handle_identity(action, &paths);
            }
        }
    }

    // Start metrics server if requested
    if let Some(addr) = args.metrics {
        tokio::spawn(async move {
            if let Err(e) = start_metrics_server(addr).await {
                eprintln!("Metrics server error: {e}");
            }
        });
    }

    if args.vpn && !args.test_mode {
        let paths = TrustPaths::resolve(args.identity, args.trusted_peers)?;
        return run_authenticated_vpn(
            paths.load()?,
            args.listen,
            args.peer,
            args.peer_key,
            args.packet_bridge,
            TunConfig {
                name: args.tun_name,
                address: args.tun_address,
                netmask: args.tun_netmask,
                mtu: 1420,
            },
        )
        .await;
    }
    if args.vpn {
        // The legacy link below sends packets over libp2p without the CrypRQ
        // handshake and with a throwaway identity.
        if !node::test_mode_enabled() {
            anyhow::bail!(
                "--vpn --test-mode runs an unauthenticated link and requires {}=1",
                node::TEST_MODE_ENV
            );
        }
        log::warn!("TEST MODE: the VPN link is NOT authenticated. INSECURE.");
    }
    if args.packet_bridge.is_some() || args.peer_key.is_some() {
        anyhow::bail!("--packet-bridge and --peer-key require --vpn without --test-mode");
    }

    // Start key rotation task
    let rotation_interval = Duration::from_secs(
        env::var("CRYPRQ_ROTATE_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300),
    );
    tokio::spawn(async move {
        start_key_rotation(rotation_interval).await;
    });

    // Handle VPN mode - store TUN interface in shared state for callback access
    let tun_interface_shared: Arc<tokio::sync::Mutex<Option<TunInterface>>> =
        Arc::new(tokio::sync::Mutex::new(None));

    if args.vpn {
        log::info!("VPN MODE ENABLED - System-wide routing mode");
        log::info!("Creating TUN interface for packet forwarding...");

        let tun_config = TunConfig {
            name: args.tun_name.clone(),
            address: args.tun_address.clone(),
            netmask: args.tun_netmask.clone(),
            mtu: 1420,
        };

        let tun = TunInterface::create(tun_config)
            .await
            .context("Failed to create TUN interface")?;

        // Try to configure IP (may fail without root/admin)
        if let Err(e) = tun.configure_ip().await {
            log::warn!("Failed to configure TUN interface IP (may need root/admin): {e}");
            log::warn!("VPN mode: P2P tunnel encryption is active, but system routing requires Network Extension");
        } else {
            log::info!(
                "TUN interface {} configured with IP {}",
                tun.name(),
                args.tun_address
            );
        }

        // Store TUN interface in shared state
        *tun_interface_shared.lock().await = Some(tun);
    }

    // Start listener or dialer
    if let Some(addr) = args.listen {
        println!("Starting listener on {addr}");
        if args.vpn {
            log::info!("VPN Mode: Listener will accept connections and route traffic through TUN interface");
            log::warn!(
                "Note: Full system-wide routing requires Network Extension framework on macOS"
            );

            // Set up callback to start packet forwarding when connection is established
            let tun_shared = tun_interface_shared.clone();
            let tun_name = args.tun_name.clone();

            p2p::set_connection_callback(Arc::new(move |peer_id, swarm, _recv_tx| {
                let tun_shared_clone = tun_shared.clone();
                let tun_name_clone = tun_name.clone();

                tokio::spawn(async move {
                    log::info!("Connection established with {peer_id} - Starting VPN packet forwarding");
                    log::info!("TUN interface {tun_name_clone} ready - packets will be forwarded through encrypted tunnel");

                    // Get TUN interface from shared state
                    let mut tun_guard = tun_shared_clone.lock().await;
                    if let Some(mut tun) = tun_guard.take() {
                        // Create packet forwarder
                        let (forwarder, _send_tx, _recv_rx) = Libp2pPacketForwarder::new(swarm.clone(), peer_id);

                        // Store recv_tx for forwarding incoming packets from swarm events
                        // The swarm event handler will use this to forward packets to TUN
                        let forwarder_recv_tx = forwarder.recv_tx();

                        // Register recv_tx channel so swarm event handler can forward packets
                        register_packet_recv_tx(peer_id, forwarder_recv_tx.clone()).await;

                        let forwarder_arc = Arc::new(tokio::sync::Mutex::new(forwarder));

                        // Start packet forwarding loop
                        log::info!("Starting packet forwarding loop - routing system traffic through encrypted tunnel");
                        if let Err(e) = tun.start_forwarding(forwarder_arc).await {
                            log::error!("Failed to start packet forwarding: {e}");
                        } else {
                            log::info!("Packet forwarding loop started successfully");
                        }
                    } else {
                        log::error!("TUN interface not available for packet forwarding");
                    }
                });
            })).await;
        }
        start_listener(&addr).await?;
    } else if let Some(peer_addr) = args.peer {
        println!("Dialing peer {peer_addr}");
        if args.vpn {
            log::info!("VPN Mode: Dialer will establish encrypted tunnel and route traffic through TUN interface");
            log::warn!(
                "Note: Full system-wide routing requires Network Extension framework on macOS"
            );

            // Set up callback to start packet forwarding when connection is established
            let tun_shared = tun_interface_shared.clone();
            let tun_name = args.tun_name.clone();

            p2p::set_connection_callback(Arc::new(move |peer_id, swarm, _recv_tx| {
                let tun_shared_clone = tun_shared.clone();
                let tun_name_clone = tun_name.clone();

                tokio::spawn(async move {
                    log::info!("Connected to {peer_id} - Starting VPN packet forwarding");
                    log::info!("TUN interface {tun_name_clone} ready - packets will be forwarded through encrypted tunnel");

                    // Get TUN interface from shared state
                    let mut tun_guard = tun_shared_clone.lock().await;
                    if let Some(mut tun) = tun_guard.take() {
                        // Create packet forwarder
                        let (forwarder, _send_tx, _recv_rx) = Libp2pPacketForwarder::new(swarm.clone(), peer_id);
                        let forwarder_recv_tx = forwarder.recv_tx();

                        // Register recv_tx channel so swarm event handler can forward packets
                        register_packet_recv_tx(peer_id, forwarder_recv_tx.clone()).await;

                        let forwarder_arc = Arc::new(tokio::sync::Mutex::new(forwarder));

                        // Start packet forwarding loop
                        log::info!("Starting packet forwarding loop - routing system traffic through encrypted tunnel");
                        if let Err(e) = tun.start_forwarding(forwarder_arc).await {
                            log::error!("Failed to start packet forwarding: {e}");
                        } else {
                            log::info!("Packet forwarding loop started successfully");
                        }
                    } else {
                        log::error!("TUN interface not available for packet forwarding");
                    }
                });
            })).await;
        }
        // Keep connection alive - don't exit immediately
        dial_peer(peer_addr).await?;
        // Connection established - keep running for VPN mode
        if args.vpn {
            log::info!("Connection established - keeping alive for VPN mode");
            // Keep the process running
            tokio::signal::ctrl_c().await?;
        }
    }

    Ok(())
}

/// Seconds from an environment variable, if it holds a positive number.
fn env_secs(name: &str) -> Option<Duration> {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
}

/// Waits for whichever of two futures completes first and drops the other,
/// like a two-branch `tokio::select!` but without the macro, whose expansion
/// contains `panic!`. `b_first` says which one is polled first when both are
/// ready.
async fn first_of<A, B>(a: A, b: B, b_first: bool) -> Either<A::Output, B::Output>
where
    A: std::future::Future,
    B: std::future::Future,
{
    let a = std::pin::pin!(a);
    let b = std::pin::pin!(b);
    if b_first {
        match futures::future::select(b, a).await {
            Either::Left((b, _)) => Either::Right(b),
            Either::Right((a, _)) => Either::Left(a),
        }
    } else {
        match futures::future::select(a, b).await {
            Either::Left((a, _)) => Either::Left(a),
            Either::Right((b, _)) => Either::Right(b),
        }
    }
}

/// Packets queued between the device and the link before they are dropped.
const DEVICE_QUEUE_LEN: usize = 1024;

/// A loopback UDP socket standing in for a TUN device: every datagram it
/// receives is an IP packet for the tunnel, and every packet from the tunnel is
/// sent to the address that last sent one.
async fn packet_bridge(addr: SocketAddr) -> Result<PacketIo> {
    if !addr.ip().is_loopback() {
        anyhow::bail!("--packet-bridge must be a loopback address, got {addr}");
    }
    let socket = tokio::net::UdpSocket::bind(addr)
        .await
        .with_context(|| format!("Failed to bind the packet bridge to {addr}"))?;
    log::info!("event=vpn_packet_bridge addr={}", socket.local_addr()?);
    let (from_device_tx, from_device) = tokio::sync::mpsc::channel::<Vec<u8>>(DEVICE_QUEUE_LEN);
    let (to_device, mut to_device_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(DEVICE_QUEUE_LEN);

    tokio::spawn(async move {
        let mut buf = vec![0u8; node::tun::MAX_IP_PACKET_LEN];
        let mut client: Option<SocketAddr> = None;
        let mut outgoing_first = false;
        loop {
            // The two directions take turns at being looked at first.
            let wake = first_of(
                socket.recv_from(&mut buf),
                to_device_rx.recv(),
                outgoing_first,
            )
            .await;
            outgoing_first = !outgoing_first;
            match wake {
                Either::Left(received) => match received {
                    Ok((len, from)) if from.ip().is_loopback() => {
                        client = Some(from);
                        if node::tun::ip_version(&buf[..len]).is_some() {
                            let _ = from_device_tx.try_send(buf[..len].to_vec());
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Packet bridge receive error: {e}");
                        break;
                    }
                },
                Either::Right(packet) => match (packet, client) {
                    (Some(packet), Some(client)) => {
                        let _ = socket.send_to(&packet, client).await;
                    }
                    (Some(_), None) => {}
                    (None, _) => break,
                },
            }
        }
    });
    Ok(PacketIo {
        from_device,
        to_device,
    })
}

/// Runs the VPN over an authenticated link: the CrypRQ handshake between
/// pinned identities, and IP packets only as records of its session.
async fn run_authenticated_vpn(
    config: TunnelConfig,
    listen: Option<String>,
    peer: Option<String>,
    peer_key: Option<String>,
    bridge: Option<SocketAddr>,
    tun_config: TunConfig,
) -> Result<()> {
    let mode = match (listen, peer) {
        (Some(addr), None) => {
            if peer_key.is_some() {
                anyhow::bail!("--peer-key selects the peer to dial and is not used with --listen");
            }
            VpnMode::Listen(addr.parse().context("Invalid --listen address")?)
        }
        (None, Some(addr)) => {
            let mut addr: Multiaddr = addr.parse().context("Invalid --peer address")?;
            // The transport peer id is derived from the pinned key.
            if matches!(
                addr.iter().last(),
                Some(libp2p::multiaddr::Protocol::P2p(_))
            ) {
                addr.pop();
            }
            let peer = expected_peer(peer_key.as_deref(), &config.trusted_peers)?;
            VpnMode::Dial { addr, peer }
        }
        _ => anyhow::bail!("--vpn needs exactly one of --listen and --peer"),
    };

    let io = match bridge {
        Some(addr) => packet_bridge(addr).await?,
        None => {
            let tun = TunInterface::create(tun_config.clone())
                .await
                .context("Failed to create TUN interface")?;
            // A device without its address cannot carry the tunnel. Stop here
            // instead of running a link that looks up and forwards nothing.
            tun.configure_ip()
                .await
                .context("Failed to configure the TUN interface; refusing to start the VPN")?;
            log::info!(
                "event=vpn_tun_ready name={} address={} netmask={}",
                tun.name(),
                tun_config.address,
                tun_config.netmask
            );
            let (from_device, to_device) = tun.into_channels(DEVICE_QUEUE_LEN)?;
            PacketIo {
                from_device,
                to_device,
            }
        }
    };

    let mut settings = VpnSettings::new(config);
    if let Some(interval) = env_secs("CRYPRQ_ROTATE_SECS") {
        settings.rotate_interval = interval;
    }
    if let Some(interval) = env_secs("CRYPRQ_REHANDSHAKE_SECS") {
        settings.rehandshake_interval = interval;
    }
    log::info!(
        "event=vpn_starting mode={} rotate_secs={} rehandshake_secs={}",
        match &mode {
            VpnMode::Listen(_) => "listen",
            VpnMode::Dial { .. } => "dial",
        },
        settings.rotate_interval.as_secs(),
        settings.rehandshake_interval.as_secs()
    );
    run_secure_vpn(settings, mode, io, async {
        shutdown_signal().await;
        log::info!("Shutting down...");
    })
    .await
}

/// Completes on Ctrl+C or, on Unix, on SIGTERM, so that a supervised process
/// closes its session instead of leaving the peer to time out.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => {
                first_of(tokio::signal::ctrl_c(), terminate.recv(), false).await;
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// Payload bytes per FILE_CHUNK record. With the 20-byte record header and
/// the 16-byte tag the datagram stays below a 1280-byte path MTU.
const FILE_CHUNK_LEN: usize = 1200;
/// Chunks sent before the sender pauses, and the length of the pause. This
/// keeps the sender below the receiver's per-tunnel rate limit.
const CHUNKS_PER_BURST: u32 = 64;
const BURST_PAUSE: Duration = Duration::from_millis(100);

/// The insecure fixed-key tunnel behind `--test-mode`. The sender is the
/// initiator and the receiver the responder.
async fn test_mode_tunnel(
    role: TunnelRole,
    listen_socket: &str,
    output_dir: Option<PathBuf>,
) -> Result<Tunnel> {
    log::warn!("TEST MODE: fixed public keys, no peer authentication. INSECURE.");
    node::create_test_mode_tunnel(role, listen_socket, output_dir)
        .await
        .with_context(|| {
            format!(
                "Failed to create the test-mode tunnel ({} must be set to 1 or true)",
                node::TEST_MODE_ENV
            )
        })
}

/// The pinned key of the peer to dial: `--peer-key`, or the only pinned peer.
fn expected_peer(peer_key: Option<&str>, trusted: &TrustedPeers) -> Result<PeerPublicKey> {
    if let Some(text) = peer_key {
        return PeerPublicKey::parse(text)
            .map_err(|e| anyhow::anyhow!("--peer-key is not a valid `ed25519:<hex>` key: {e}"));
    }
    let mut pinned = trusted.iter();
    match (pinned.next(), pinned.next()) {
        (Some(only), None) => Ok(*only),
        _ => anyhow::bail!("several peers are pinned; select the receiver with --peer-key"),
    }
}

async fn handle_send_file(
    peer_addr: String,
    file_path: PathBuf,
    peer_key: Option<String>,
    test_mode: bool,
    paths: Result<TrustPaths>,
) -> Result<()> {
    use node::FileMetadata;
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    log::info!("Sending file: {file_path:?} to peer: {peer_addr}");

    // Parse peer address - extract UDP socket address
    let (peer_ip, peer_port) = parse_udp_addr(&peer_addr)?;
    let peer_socket = format!("{peer_ip}:{peer_port}");
    let peer_addr_parsed: std::net::SocketAddr = peer_socket
        .parse()
        .context("Failed to parse peer socket address")?;

    let tunnel = if test_mode {
        let tunnel = test_mode_tunnel(TunnelRole::Initiator, "0.0.0.0:0", None).await?;
        *tunnel
            .peer_addr()
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire peer_addr lock: {e}"))? =
            Some(peer_addr_parsed);
        tunnel
    } else {
        // Authenticated mode: no tunnel exists until the pinned receiver has
        // proven its identity and both sides confirmed the session keys.
        let config = paths?.load()?;
        let expected = expected_peer(peer_key.as_deref(), &config.trusted_peers)?;
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .context("Failed to bind a UDP socket")?;
        let tunnel = node::connect_tunnel(&config, socket, peer_addr_parsed, &expected)
            .await
            .with_context(|| format!("Handshake with {expected} at {peer_socket} failed"))?;
        log::info!(
            "event=tunnel_established role=initiator peer={} session_id={}",
            expected,
            hex::encode(tunnel.session_id())
        );
        tunnel
    };
    let tunnel = Arc::new(tunnel);

    // Calculate file metadata
    let file_size = std::fs::metadata(&file_path)?.len();
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
        .to_string();

    // Calculate SHA-256 hash
    let mut file_for_hash = File::open(&file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; FILE_CHUNK_LEN];
    loop {
        let bytes_read = file_for_hash.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let file_hash: [u8; 32] = hasher.finalize().into();

    let meta = FileMetadata::new(filename.clone(), file_size, file_hash);

    // Allocate stream ID for this transfer
    let stream_id = tunnel.file_transfer().alloc_stream_id();
    log::info!("Allocated stream_id={stream_id} for file transfer: {filename} ({file_size} bytes)");

    // Send FILE_META
    tunnel
        .send_file_meta(stream_id, &meta)
        .await
        .context("Failed to send file metadata")?;
    log::info!("Sent FILE_META: stream_id={stream_id}, filename={filename}");

    // Send file chunks
    let mut file = File::open(&file_path)?;
    let mut chunk_num: u32 = 0;
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        tunnel
            .send_file_chunk(stream_id, &buffer[..bytes_read])
            .await
            .context(format!("Failed to send chunk {chunk_num}"))?;

        log::debug!("Sent FILE_CHUNK: stream_id={stream_id}, chunk={chunk_num}, size={bytes_read}");

        chunk_num += 1;
        if chunk_num % CHUNKS_PER_BURST == 0 {
            tokio::time::sleep(BURST_PAUSE).await;
        }
    }

    log::info!(
        "File transfer complete: {filename} sent ({chunk_num} chunks) on stream_id={stream_id}"
    );

    // Wait a bit for final packets to be sent
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}

// Old implementation - kept for reference but not used
#[allow(dead_code)]
async fn handle_send_file_old(peer_addr: String, file_path: PathBuf) -> Result<()> {
    use libp2p::multiaddr::Protocol;
    use libp2p::request_response;
    use libp2p::swarm::{dial_opts::DialOpts, SwarmEvent};
    use p2p::init_swarm;
    use p2p::MyBehaviourEvent;

    log::info!("Sending file: {file_path:?} to peer: {peer_addr}");

    // Initialize swarm
    let mut swarm = init_swarm()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init swarm: {e}"))?;

    // Parse peer address
    let mut dial_addr: Multiaddr = peer_addr.parse()?;
    let peer_id = if matches!(dial_addr.iter().last(), Some(Protocol::P2p(_))) {
        match dial_addr.pop() {
            Some(Protocol::P2p(pid)) => Some(pid),
            _ => None,
        }
    } else {
        None
    };

    // Dial peer
    if let Some(pid) = peer_id {
        swarm.dial(DialOpts::peer_id(pid).addresses(vec![dial_addr]).build())?;
    } else {
        swarm.dial(dial_addr)?;
    }

    let swarm_arc = Arc::new(tokio::sync::Mutex::new(swarm));

    // Wait for connection and send file
    let mut file_sent = false;
    let mut target_peer_id = None;
    let mut responses_received = 0;
    let mut responses_expected = 0;
    let mut start_time = None;

    loop {
        let event = {
            let mut swarm_guard = swarm_arc.lock().await;
            swarm_guard.select_next_some().await
        };

        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                log::info!("Connected to peer: {peer_id}");
                target_peer_id = Some(peer_id);
                start_time = Some(std::time::Instant::now());

                // Send file once connected - spawn as separate task to avoid blocking event loop
                // The event loop must continue processing for protocol negotiation and request sending
                if !file_sent {
                    let swarm_for_send = swarm_arc.clone();
                    let peer_for_send = peer_id;
                    let file_for_send = file_path.clone();

                    // Calculate expected number of responses (metadata + chunks + end)
                    let file_size = std::fs::metadata(&file_for_send)
                        .ok()
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let num_chunks = (file_size as usize).div_ceil(CHUNK_SIZE) as u32;
                    responses_expected = 1 + num_chunks + 1; // metadata + chunks + end
                    log::info!(
                        "Expected {responses_expected} responses (1 metadata + {num_chunks} chunks + 1 end)"
                    );

                    tokio::spawn(async move {
                        // Wait longer for protocol negotiation after connection
                        // Protocol negotiation happens asynchronously and needs event loop processing
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        if let Err(e) =
                            send_file_to_peer(swarm_for_send, peer_for_send, file_for_send).await
                        {
                            log::error!("Failed to send file: {e}");
                        } else {
                            log::info!("File transfer task completed - all packets sent");
                        }
                    });
                    file_sent = true;
                }
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                anyhow::bail!("Failed to connect: {error:?}");
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    message:
                        request_response::Message::Response {
                            response,
                            request_id,
                            ..
                        },
                    ..
                },
            )) => {
                log::info!(
                    "Received response for request {:?}: {} bytes",
                    request_id,
                    response.len()
                );
                responses_received += 1;
                // Exit when we've received all expected responses
                if file_sent && responses_received >= responses_expected {
                    log::info!(
                        "All responses received ({responses_received}/{responses_expected}) - exiting"
                    );
                    break;
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    error, request_id, ..
                },
            )) => {
                log::error!("[FILE_TX] Request failed for {request_id:?}: {error:?}");
                // Don't fail immediately - might be transient
            }
            _ => {}
        }

        // Exit after timeout if file was sent
        if file_sent && target_peer_id.is_some() {
            if let Some(start) = start_time {
                if start.elapsed() > Duration::from_secs(20) {
                    if responses_received < responses_expected {
                        log::warn!(
                            "Timeout waiting for responses ({responses_received}/{responses_expected}) - exiting anyway"
                        );
                    } else {
                        log::info!(
                            "All responses received ({responses_received}/{responses_expected}) - exiting"
                        );
                    }
                    break;
                }
            }
        }
    }

    log::info!("File transfer event loop exited");
    Ok(())
}

struct FileReceiveState {
    metadata: Option<FileMetadata>,
    chunks: HashMap<u32, Vec<u8>>,
    total_chunks: u32,
}

async fn handle_receive_file(
    listen_addr: String,
    output_dir: PathBuf,
    test_mode: bool,
    paths: Result<TrustPaths>,
) -> Result<()> {
    log::info!("Receiving files on: {listen_addr}, output directory: {output_dir:?}");

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)?;

    // Parse listen address - extract UDP socket address from multiaddr
    // For now, assume format like "/ip4/0.0.0.0/udp/20440/quic-v1"
    // We'll use UDP directly, so extract IP and port
    let udp_addr = parse_udp_addr(&listen_addr)?;
    let listen_socket = format!("{}:{}", udp_addr.0, udp_addr.1);

    let tunnel = if test_mode {
        let tunnel = test_mode_tunnel(
            TunnelRole::Responder,
            &listen_socket,
            Some(output_dir.clone()),
        )
        .await?;
        log::info!("Tunnel created, listening for file transfers on {listen_socket}");
        tunnel
    } else {
        // Authenticated mode: nothing is received until a pinned peer has
        // completed the handshake.
        let mut config = paths?.load()?;
        config.file_output_dir = Some(output_dir.clone());
        let socket = tokio::net::UdpSocket::bind(&listen_socket)
            .await
            .with_context(|| format!("Failed to bind {listen_socket}"))?;
        log::info!(
            "event=listening addr={} waiting for a pinned peer",
            socket.local_addr()?
        );
        let accepted = first_of(
            node::accept_tunnel(&config, socket),
            tokio::signal::ctrl_c(),
            false,
        )
        .await;
        let tunnel = match accepted {
            Either::Left(accepted) => accepted.context("Failed to accept a tunnel")?,
            Either::Right(_) => {
                log::info!("Shutting down...");
                return Ok(());
            }
        };
        match tunnel.peer_identity() {
            Some(peer) => log::info!(
                "event=tunnel_established role=responder peer={} session_id={}",
                peer,
                hex::encode(tunnel.session_id())
            ),
            None => anyhow::bail!("accepted a tunnel without an authenticated peer"),
        }
        tunnel
    };
    let tunnel = Arc::new(tunnel);

    // Spawn receive loop
    let tunnel_clone = tunnel.clone();
    tokio::spawn(async move {
        loop {
            match tunnel_clone.recv_and_handle_record().await {
                Ok(()) => {
                    // Record handled successfully
                }
                Err(e) => {
                    log::error!("Receive loop error: {e}");
                    // Continue receiving - don't break on single errors
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    // Keep process alive
    log::info!("File receiver running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    log::info!("Shutting down...");

    Ok(())
}

// Helper to parse UDP address from multiaddr
fn parse_udp_addr(addr: &str) -> Result<(String, u16)> {
    // Parse multiaddr like "/ip4/0.0.0.0/udp/20440/quic-v1"
    // Extract IP and port
    let parts: Vec<&str> = addr.split('/').collect();
    let mut ip = None;
    let mut port = None;

    for (i, part) in parts.iter().enumerate() {
        if *part == "ip4" && i + 1 < parts.len() {
            ip = Some(parts[i + 1].to_string());
        }
        if *part == "udp" && i + 1 < parts.len() {
            port = Some(parts[i + 1].parse::<u16>().context("Invalid UDP port")?);
        }
    }

    Ok((
        ip.unwrap_or_else(|| "0.0.0.0".to_string()),
        port.ok_or_else(|| anyhow::anyhow!("No UDP port found in address"))?,
    ))
}

// Old implementation - kept for reference but not used
#[allow(dead_code)]
async fn handle_receive_file_old(listen_addr: String, output_dir: PathBuf) -> Result<()> {
    use std::sync::Mutex;

    log::info!("Receiving files on: {listen_addr}, output directory: {output_dir:?}");

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)?;

    // State for receiving files
    let file_state: Arc<Mutex<HashMap<PeerId, FileReceiveState>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Set up file transfer callback
    let state_clone = file_state.clone();
    let output_dir_clone = output_dir.clone();
    set_file_transfer_callback(Arc::new(move |peer_id, data| {
        let mut state_guard = state_clone
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        let state = state_guard
            .entry(peer_id)
            .or_insert_with(|| FileReceiveState {
                metadata: None,
                chunks: HashMap::new(),
                total_chunks: 0,
            });

        // Parse packet type
        if data.len() < 4 {
            return Err("Packet too short".to_string());
        }

        let packet_type = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        match packet_type {
            0 => {
                // Metadata packet
                match FileMetadata::deserialize(&data) {
                    Ok(metadata) => {
                        let file_size = metadata.size;
                        log::info!(
                            "Receiving file: {} ({} bytes) from peer {}",
                            metadata.filename,
                            file_size,
                            peer_id
                        );
                        state.total_chunks = (file_size as usize).div_ceil(p2p::CHUNK_SIZE) as u32;
                        state.metadata = Some(metadata);
                        Ok(b"OK".to_vec())
                    }
                    Err(e) => Err(format!("Failed to parse metadata: {e}")),
                }
            }
            1 => {
                // Data chunk
                match DataChunk::deserialize(&data) {
                    Ok(chunk) => {
                        let chunk_data_len = chunk.data.len();
                        state.chunks.insert(chunk.chunk_id, chunk.data);
                        log::info!(
                            "[FILE_RX] Received chunk {} from peer {} ({} bytes)",
                            chunk.chunk_id,
                            peer_id,
                            chunk_data_len
                        );
                        Ok(b"OK".to_vec())
                    }
                    Err(e) => Err(format!("Failed to parse chunk: {e}")),
                }
            }
            2 => {
                // End packet
                log::debug!("End packet received from peer {peer_id}");
                if let Some(metadata) = &state.metadata {
                    // Write file
                    let output_path = output_dir_clone.join(&metadata.filename);
                    let mut file = File::create(&output_path)
                        .map_err(|e| format!("Failed to create file: {e}"))?;

                    // Write chunks in order
                    for chunk_id in 0..state.total_chunks {
                        if let Some(chunk_data) = state.chunks.get(&chunk_id) {
                            file.write_all(chunk_data)
                                .map_err(|e| format!("Failed to write chunk {chunk_id}: {e}"))?;
                        } else {
                            return Err(format!("Missing chunk {chunk_id}"));
                        }
                    }

                    // Verify hash
                    let received_hash = p2p::calculate_file_hash(&output_path)
                        .map_err(|e| format!("Failed to calculate hash: {e}"))?;
                    if received_hash != metadata.hash {
                        std::fs::remove_file(&output_path).ok();
                        return Err("File hash mismatch".to_string());
                    }

                    log::info!(
                        "File received successfully: {} ({} bytes) from peer {}",
                        metadata.filename,
                        metadata.size,
                        peer_id
                    );

                    // Clean up state
                    state_guard.remove(&peer_id);
                    Ok(b"OK".to_vec())
                } else {
                    Err("End packet received but no metadata".to_string())
                }
            }
            _ => Err("Unknown packet type".to_string()),
        }
    }))
    .await;

    // Start listener
    start_listener(&listen_addr).await?;

    Ok(())
}
