use clap::Parser;
use std::process;
use p2p::{start_listener, dial_peer};

#[derive(Parser, Debug)]
#[command(name = "cryprq", about = "Post-Quantum VPN")]
struct Args {
    #[arg(long, help = "Peer address to connect to (multiaddr)")]
    peer: Option<String>,

    #[arg(long, help = "Address to listen on (multiaddr)")]
    listen: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.listen.is_some() == args.peer.is_some() {
        eprintln!("Error: supply exactly one of --listen or --peer");
        process::exit(1);
    }

    if let Some(addr) = args.listen {
        println!("Starting listener on {}", addr);
        start_listener(&addr).await?;
    } else {
        println!("Dialing peer {}", args.peer.as_ref().unwrap());
        dial_peer(args.peer.unwrap()).await?;
    }
    Ok(())
}
