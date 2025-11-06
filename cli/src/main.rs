use clap::Parser;
use tokio::time::{interval, Duration};

#[derive(Parser, Debug)]
#[command(name = "cryprq")]
#[command(about = "CrypRQ - Post-Quantum VPN", long_about = None)]
struct Args {
    /// Peer address to connect to (multiaddr format)
    #[arg(long)]
    peer: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    let pk = p2p::get_current_pk();
    println!("CrypRQ v0.0.1 – Kyber pk: {:02x}{:02x}…", pk[0], pk[1]);
    
    tokio::spawn(p2p::start_key_rotation());
    
    // Spawn WireGuard tunnel
    let local_sk = [0u8; 32]; // TODO: use real keys
    let peer_pk = if let Some(peer_addr) = args.peer {
        // Parse peer address and dial
        let peer_id: libp2p::PeerId = peer_addr.parse()
            .expect("Invalid peer address");
        
        let _conn = p2p::dial_peer(peer_id).await
            .expect("Failed to dial peer");
        
        // Exchange PQ keys over libp2p stream (simplified)
        // In real implementation, would read peer's PQ key from stream
        let mut exchanged_pk = [0u8; 32];
        exchanged_pk.copy_from_slice(&p2p::get_current_pk()[..32]);
        
        println!("PQ handshake complete – tunnel ready");
        exchanged_pk
    } else {
        [1u8; 32] // Default dummy peer key
    };
    
    let _tunnel = node::create_tunnel(&local_sk, &peer_pk, "127.0.0.1:51820")
        .await
        .expect("Failed to create tunnel");
    println!("TUN up at 127.0.0.1:51820");
    
    let mut tick = interval(Duration::from_secs(300)); // 5 minutes
    tick.tick().await; // Skip the first immediate tick
    loop {
        tick.tick().await;
        let pk = p2p::get_current_pk();
        println!("CrypRQ v0.0.1 – Kyber pk: {:02x}{:02x}…", pk[0], pk[1]);
    }
}
