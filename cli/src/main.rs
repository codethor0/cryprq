// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use clap::Parser;
use p2p::{dial_peer, spawn_metrics_server, start_key_rotation, start_listener};
use std::{net::SocketAddr, process};

const VERSION: &str = env!("CRYPRQ_VERSION_STRING");
const BUILD_SHA: &str = env!("CRYPRQ_BUILD_SHA");
const BUILD_RUSTC: &str = env!("CRYPRQ_BUILD_RUSTC");
const BUILD_FEATURES: &str = env!("CRYPRQ_BUILD_FEATURES");
const BUILD_TIME: &str = env!("CRYPRQ_BUILD_TIME");

#[derive(Parser, Debug)]
#[command(name = "cryprq", about = "Post-Quantum VPN", version = VERSION)]
struct Args {
    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,

    #[arg(
        long,
        env = "CRYPRQ_METRICS_ADDR",
        default_value = "127.0.0.1:9464",
        value_name = "ADDR",
        help = "Expose /metrics and /healthz on this socket address"
    )]
    metrics_addr: String,

    #[arg(long, help = "Print structured build metadata and exit")]
    build_info: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.build_info {
        print_build_info();
        return Ok(());
    }

    if args.listen.is_some() == args.peer.is_some() {
        eprintln!("Error: supply exactly one of --listen or --peer");
        process::exit(1);
    }

    let metrics_addr: SocketAddr = args
        .metrics_addr
        .parse()
        .context("invalid metrics address")?;
    spawn_metrics_server(metrics_addr)?;

    // Start key rotation task
    tokio::spawn(async {
        start_key_rotation().await;
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

fn print_build_info() {
    let features: Vec<&str> = BUILD_FEATURES
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();
    let features_json = features
        .iter()
        .map(|f| format!("\"{}\"", f))
        .collect::<Vec<_>>()
        .join(", ");

    println!(
        "{{\n  \"version\": \"{}\",\n  \"git_sha\": \"{}\",\n  \"built_at\": \"{}\",\n  \"rustc\": \"{}\",\n  \"features\": [{}]\n}}",
        env!("CARGO_PKG_VERSION"),
        BUILD_SHA,
        BUILD_TIME,
        BUILD_RUSTC,
        features_json
    );
}
