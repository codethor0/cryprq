use anyhow::Result;
use clap::Parser;
use p2p::{start_key_rotation, start_listener, dial_peer};
use std::process;

#[derive(Parser, Debug)]
#[command(name = "cryprq", about = "Post-Quantum VPN")]
struct Args {
    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.listen.is_some() == args.peer.is_some() {
        eprintln!("Error: supply exactly one of --listen or --peer");
        process::exit(1);
    }

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
