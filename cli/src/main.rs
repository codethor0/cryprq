// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use clap::Parser;
use p2p::{dial_peer, start_key_rotation, start_listener, start_metrics_server};
use std::{env, net::SocketAddr, process, sync::Arc, time::Duration};
use node::{TunConfig, TunInterface};

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

    // Handle VPN mode
    let mut tun_interface = if args.vpn {
        log::info!("🔒 VPN MODE ENABLED - System-wide routing mode");
        log::info!("Creating TUN interface for packet forwarding...");
        
        let tun_config = TunConfig {
            name: args.tun_name.clone(),
            address: args.tun_address.clone(),
            netmask: "255.255.255.0".to_string(),
            mtu: 1420,
        };

        let tun = TunInterface::create(tun_config).await
            .context("Failed to create TUN interface")?;
        
        // Try to configure IP (may fail without root/admin)
        if let Err(e) = tun.configure_ip().await {
            log::warn!("Failed to configure TUN interface IP (may need root/admin): {}", e);
            log::warn!("VPN mode: P2P tunnel encryption is active, but system routing requires Network Extension");
        } else {
            log::info!("✅ TUN interface {} configured with IP {}", tun.name(), args.tun_address);
        }

        Some(tun)
    } else {
        None
    };

    // Start listener or dialer
    if let Some(addr) = args.listen {
        println!("Starting listener on {}", addr);
        if args.vpn {
            log::info!("VPN Mode: Listener will accept connections and route traffic through TUN interface");
            log::warn!("Note: Full system-wide routing requires Network Extension framework on macOS");
            
            // Set up callback to start packet forwarding when connection is established
            if let Some(ref mut tun) = tun_interface {
                let tun_name = tun.name().to_string();
                p2p::set_connection_callback(Arc::new(move |peer_id, swarm| {
                    log::info!("✅ Connection established with {peer_id} - Starting VPN packet forwarding");
                    log::info!("TUN interface {} ready - packets will be forwarded through encrypted tunnel", tun_name);
                    
                    // TODO: Start actual packet forwarding loop here
                    // This requires:
                    // 1. Create PacketForwarder implementation for libp2p
                    // 2. Call tun.start_forwarding(forwarder)
                    // 3. Handle packet forwarding between TUN and libp2p streams
                    log::warn!("Packet forwarding loop not yet implemented - TUN interface ready but not forwarding");
                })).await;
            }
        }
        start_listener(&addr).await?;
    } else if let Some(peer_addr) = args.peer {
        println!("Dialing peer {}", peer_addr);
        if args.vpn {
            log::info!("VPN Mode: Dialer will establish encrypted tunnel and route traffic through TUN interface");
            log::warn!("Note: Full system-wide routing requires Network Extension framework on macOS");
            
            // Set up callback to start packet forwarding when connection is established
            if let Some(ref mut tun) = tun_interface {
                let tun_name = tun.name().to_string();
                p2p::set_connection_callback(Arc::new(move |peer_id, swarm| {
                    log::info!("✅ Connected to {peer_id} - Starting VPN packet forwarding");
                    log::info!("TUN interface {} ready - packets will be forwarded through encrypted tunnel", tun_name);
                    
                    // TODO: Start actual packet forwarding loop here
                    // This requires:
                    // 1. Create PacketForwarder implementation for libp2p
                    // 2. Call tun.start_forwarding(forwarder)
                    // 3. Handle packet forwarding between TUN and libp2p streams
                    log::warn!("Packet forwarding loop not yet implemented - TUN interface ready but not forwarding");
                })).await;
            }
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
