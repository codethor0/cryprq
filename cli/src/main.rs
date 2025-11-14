// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use clap::Parser;
use node::{TunConfig, TunInterface};
use p2p::{
    dial_peer, register_packet_recv_tx, start_key_rotation, start_listener, start_metrics_server,
    Libp2pPacketForwarder,
};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};

#[derive(Parser, Debug)]
#[command(name = "cryprq", about = "Post-Quantum VPN")]
struct Args {
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

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
                        log::info!("🚀 Starting packet forwarding loop - routing system traffic through encrypted tunnel");
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
                        log::info!("🚀 Starting packet forwarding loop - routing system traffic through encrypted tunnel");
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
