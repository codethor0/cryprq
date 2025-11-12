// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use clap::Parser;
use p2p::{dial_peer, start_key_rotation, start_listener, start_metrics_server};
use std::{env, net::SocketAddr, process, time::Duration};

#[derive(Parser, Debug)]
#[command(name = "cryprq", about = "Post-Quantum VPN")]
struct Args {
    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,

    #[arg(
        long,
        help = "Address to bind the metrics/health server (e.g., 127.0.0.1:9464)"
    )]
    metrics_addr: Option<String>,

    #[arg(long, help = "Override rotation interval in seconds (defaults to 300)")]
    rotate_secs: Option<u64>,

    #[arg(
        long = "allow-peer",
        help = "Allowlisted peer ID (may be supplied multiple times)",
        action = clap::ArgAction::Append
    )]
    allow_peers: Vec<String>,

    #[arg(
        long = "no-post-quantum",
        help = "Disable post-quantum encryption (fallback to X25519-only). Not recommended. Default: post-quantum enabled.",
        action = clap::ArgAction::SetTrue
    )]
    no_post_quantum: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    if args.listen.is_some() == args.peer.is_some() {
        eprintln!("Error: supply exactly one of --listen or --peer");
        process::exit(1);
    }

    let rotation_secs = args
        .rotate_secs
        .or_else(|| {
            env::var("CRYPRQ_ROTATE_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
        })
        .unwrap_or(300)
        .max(1);
    let rotation_interval = Duration::from_secs(rotation_secs);

    // Post-quantum encryption flag (default: enabled)
    let post_quantum_enabled = !args.no_post_quantum;
    if !post_quantum_enabled {
        log::warn!("Post-quantum encryption is DISABLED. Using X25519-only (not recommended for long-term security).");
    } else {
        log::info!("Post-quantum encryption enabled: ML-KEM (Kyber768) + X25519 hybrid");
    }

    if let Some(metrics_addr_str) = args
        .metrics_addr
        .or_else(|| env::var("CRYPRQ_METRICS_ADDR").ok())
    {
        let metrics_addr: SocketAddr = metrics_addr_str
            .parse()
            .with_context(|| format!("Invalid metrics address: {metrics_addr_str}"))?;
        tokio::spawn(async move {
            if let Err(err) = start_metrics_server(metrics_addr).await {
                log::error!(
                    "event=metrics_server_error addr={} error={}",
                    metrics_addr,
                    err
                );
            }
        });
    }

    let env_allow: Vec<String> = env::var("CRYPRQ_ALLOW_PEERS")
        .ok()
        .map(|v| {
            v.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    let combined_allow = args
        .allow_peers
        .into_iter()
        .chain(env_allow.into_iter())
        .collect::<Vec<_>>();
    if !combined_allow.is_empty() {
        p2p::set_allowed_peers(&combined_allow)
            .await
            .with_context(|| "Failed to configure allowed peers")?;
    }

    // Start key rotation task
    tokio::spawn(async move {
        start_key_rotation(rotation_interval).await;
    });

    if let Some(addr) = args.listen {
        println!("Starting listener on {}", addr);
        start_listener(&addr).await?;
    } else if let Some(peer_addr) = args.peer {
        println!("Dialing peer {}", peer_addr);
        dial_peer(peer_addr).await?;
    }

    Ok(())
}
