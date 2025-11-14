// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId};
use node::{FileMetadata, TunConfig, TunInterface};
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
#[command(name = "cryprq", about = "Post-Quantum VPN")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,

    #[arg(long, help = "Enable VPN mode (system-wide routing)")]
    vpn: bool,

    #[arg(long, default_value = "cryprq0", help = "TUN interface name")]
    tun_name: String,

    #[arg(long, default_value = "10.0.0.1", help = "TUN interface IP address")]
    tun_address: String,

    #[arg(long, help = "Metrics server address")]
    metrics: Option<SocketAddr>,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    // Handle file transfer subcommands
    if let Some(command) = args.command {
        match command {
            Command::SendFile { peer, file } => {
                return handle_send_file(peer, file).await;
            }
            Command::ReceiveFile { listen, output_dir } => {
                return handle_receive_file(listen, output_dir).await;
            }
        }
    }

    // Start metrics server if requested
    if let Some(addr) = args.metrics {
        tokio::spawn(async move {
            if let Err(e) = start_metrics_server(addr).await {
                eprintln!("Metrics server error: {}", e);
            }
        });
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
            netmask: "255.255.255.0".to_string(),
            mtu: 1420,
        };

        let tun = TunInterface::create(tun_config)
            .await
            .context("Failed to create TUN interface")?;

        // Try to configure IP (may fail without root/admin)
        if let Err(e) = tun.configure_ip().await {
            log::warn!(
                "Failed to configure TUN interface IP (may need root/admin): {}",
                e
            );
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
        println!("Starting listener on {}", addr);
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
                    log::info!("TUN interface {} ready - packets will be forwarded through encrypted tunnel", tun_name_clone);

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
                            log::error!("Failed to start packet forwarding: {}", e);
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
        println!("Dialing peer {}", peer_addr);
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
                    log::info!("TUN interface {} ready - packets will be forwarded through encrypted tunnel", tun_name_clone);

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
                            log::error!("Failed to start packet forwarding: {}", e);
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

async fn handle_send_file(peer_addr: String, file_path: PathBuf) -> Result<()> {
    use node::FileMetadata;
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    log::info!("Sending file: {:?} to peer: {}", file_path, peer_addr);

    // Parse peer address - extract UDP socket address
    let (peer_ip, peer_port) = parse_udp_addr(&peer_addr)?;
    let peer_socket = format!("{}:{}", peer_ip, peer_port);

    // Generate temporary keys for testing (in production, these would come from config/identity)
    // Using simple test keys that will pass test mode verification
    let local_sk = [0x01; 32];
    let peer_pk = [0x02; 32];
    let peer_identity_key = [0x03; 32];
    let peer_signature = [0x04; 64];

    // Create tunnel (bind to any available port for sender)
    let tunnel = Arc::new(
        node::create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            "0.0.0.0:0", // Bind to any port
        )
        .await
        .context("Failed to create tunnel")?,
    );

    // Set peer address for sending
    let peer_addr_parsed: std::net::SocketAddr = peer_socket
        .parse()
        .context("Failed to parse peer socket address")?;
    {
        let mut peer_addr_guard = tunnel
            .peer_addr()
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire peer_addr lock: {}", e))?;
        *peer_addr_guard = Some(peer_addr_parsed);
    }

    log::info!("Tunnel created, connecting to peer at {}", peer_socket);

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
    let mut buffer = vec![0u8; 65536];
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
    log::info!(
        "Allocated stream_id={} for file transfer: {} ({} bytes)",
        stream_id,
        filename,
        file_size
    );

    // Send FILE_META
    tunnel
        .send_file_meta(stream_id, &meta)
        .await
        .context("Failed to send file metadata")?;
    log::info!(
        "Sent FILE_META: stream_id={}, filename={}",
        stream_id,
        filename
    );

    // Send file chunks
    let mut file = File::open(&file_path)?;
    let mut chunk_num = 0;
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        tunnel
            .send_file_chunk(stream_id, &buffer[..bytes_read])
            .await
            .context(format!("Failed to send chunk {}", chunk_num))?;

        log::debug!(
            "Sent FILE_CHUNK: stream_id={}, chunk={}, size={}",
            stream_id,
            chunk_num,
            bytes_read
        );

        chunk_num += 1;
    }

    log::info!(
        "File transfer complete: {} sent ({} chunks) on stream_id={}",
        filename,
        chunk_num,
        stream_id
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

    log::info!("Sending file: {:?} to peer: {}", file_path, peer_addr);

    // Initialize swarm
    let mut swarm = init_swarm()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init swarm: {}", e))?;

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
                log::info!("Connected to peer: {}", peer_id);
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
                        "Expected {} responses (1 metadata + {} chunks + 1 end)",
                        responses_expected,
                        num_chunks
                    );

                    tokio::spawn(async move {
                        // Wait longer for protocol negotiation after connection
                        // Protocol negotiation happens asynchronously and needs event loop processing
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        if let Err(e) =
                            send_file_to_peer(swarm_for_send, peer_for_send, file_for_send).await
                        {
                            log::error!("Failed to send file: {}", e);
                        } else {
                            log::info!("File transfer task completed - all packets sent");
                        }
                    });
                    file_sent = true;
                }
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                anyhow::bail!("Failed to connect: {:?}", error);
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
                        "All responses received ({}/{}) - exiting",
                        responses_received,
                        responses_expected
                    );
                    break;
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    error, request_id, ..
                },
            )) => {
                log::error!("[FILE_TX] Request failed for {:?}: {:?}", request_id, error);
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
                            "Timeout waiting for responses ({}/{}) - exiting anyway",
                            responses_received,
                            responses_expected
                        );
                    } else {
                        log::info!(
                            "All responses received ({}/{}) - exiting",
                            responses_received,
                            responses_expected
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

async fn handle_receive_file(listen_addr: String, output_dir: PathBuf) -> Result<()> {
    log::info!(
        "Receiving files on: {}, output directory: {:?}",
        listen_addr,
        output_dir
    );

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)?;

    // Parse listen address - extract UDP socket address from multiaddr
    // For now, assume format like "/ip4/0.0.0.0/udp/20440/quic-v1"
    // We'll use UDP directly, so extract IP and port
    let udp_addr = parse_udp_addr(&listen_addr)?;
    let listen_socket = format!("{}:{}", udp_addr.0, udp_addr.1);

    // Generate temporary keys for testing (in production, these would come from config/identity)
    // Using simple test keys that will pass test mode verification
    let local_sk = [0x01; 32];
    let peer_pk = [0x02; 32]; // Will be updated when peer connects
    let peer_identity_key = [0x03; 32];
    let peer_signature = [0x04; 64];

    // Create tunnel with output directory
    let tunnel = Arc::new(
        node::create_tunnel_with_output_dir(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            &listen_socket,
            Some(output_dir.clone()),
        )
        .await
        .context("Failed to create tunnel")?,
    );

    log::info!(
        "Tunnel created, listening for file transfers on {}",
        listen_socket
    );

    // Spawn receive loop
    let tunnel_clone = tunnel.clone();
    tokio::spawn(async move {
        loop {
            match tunnel_clone.recv_and_handle_record().await {
                Ok(()) => {
                    // Record handled successfully
                }
                Err(e) => {
                    log::error!("Receive loop error: {}", e);
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

    log::info!(
        "Receiving files on: {}, output directory: {:?}",
        listen_addr,
        output_dir
    );

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
            .map_err(|e| format!("Lock poisoned: {}", e))?;
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
                    Err(e) => Err(format!("Failed to parse metadata: {}", e)),
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
                    Err(e) => Err(format!("Failed to parse chunk: {}", e)),
                }
            }
            2 => {
                // End packet
                log::debug!("End packet received from peer {}", peer_id);
                if let Some(metadata) = &state.metadata {
                    // Write file
                    let output_path = output_dir_clone.join(&metadata.filename);
                    let mut file = File::create(&output_path)
                        .map_err(|e| format!("Failed to create file: {}", e))?;

                    // Write chunks in order
                    for chunk_id in 0..state.total_chunks {
                        if let Some(chunk_data) = state.chunks.get(&chunk_id) {
                            file.write_all(chunk_data).map_err(|e| {
                                format!("Failed to write chunk {}: {}", chunk_id, e)
                            })?;
                        } else {
                            return Err(format!("Missing chunk {}", chunk_id));
                        }
                    }

                    // Verify hash
                    let received_hash = p2p::calculate_file_hash(&output_path)
                        .map_err(|e| format!("Failed to calculate hash: {}", e))?;
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
