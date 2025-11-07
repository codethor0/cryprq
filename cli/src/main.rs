<<<<<<< Updated upstream
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
=======
use anyhow::Result;
// use clap::{Parser, Subcommand}; // Not used
use p2p::{get_current_pk, start_key_rotation, init_swarm, MyBehaviourEvent};
use libp2p::{
    mdns,
    swarm::SwarmEvent,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut swarm = init_swarm().await.map_err(|e| anyhow::anyhow!(e))?;
    use futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer, addr) in list {
                    println!("Discovered {} @ {}", peer, addr);
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            _ => {}
        }
>>>>>>> Stashed changes
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
