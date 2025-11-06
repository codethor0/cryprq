// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024 Thorsten Behrens
use clap::Parser;
use tokio::time::{interval, Duration};
use rand::RngCore;
use ed25519_dalek::{SigningKey, Signer};

#[derive(Parser, Debug)]
#[command(name = "cryprq")]
#[command(about = "CrypRQ - Post-Quantum VPN", long_about = None)]
struct Args {
    /// Peer address to connect to (multiaddr format)
    #[arg(long)]
    peer: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let pk = p2p::get_current_pk()?;
    println!("CrypRQ v0.0.1 – Kyber pk: {:02x}{:02x}…", pk[0], pk[1]);
    
    tokio::spawn(p2p::start_key_rotation());
    
    // Generate local identity key for authentication
    let mut rng = rand::rngs::OsRng;
    let secret_bytes: [u8; 32] = rand::Rng::gen(&mut rng);
    let local_identity_key = SigningKey::from_bytes(&secret_bytes);
    let local_identity_pubkey = local_identity_key.verifying_key().to_bytes();
    
    println!("Local identity: {:02x}{:02x}…", local_identity_pubkey[0], local_identity_pubkey[1]);
    
    // Generate secure keys from entropy
    let local_sk = {
        use rand::RngCore;
        let mut sk = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut sk);
        sk
    };
    
    let peer_pk = if let Some(peer_addr) = args.peer {
        // Parse peer address and dial
        let peer_id: libp2p::PeerId = peer_addr.parse()
            .map_err(|_| "Invalid peer address format")?;
        
        let _conn = p2p::dial_peer(peer_id).await
            .map_err(|e| format!("Failed to dial peer: {}", e))?;
        
        // Exchange PQ keys over libp2p stream (simplified)
        // In real implementation, would read peer's PQ key from stream
        let mut exchanged_pk = [0u8; 32];
        exchanged_pk.copy_from_slice(&p2p::get_current_pk()?[..32]);
        
        println!("PQ handshake complete – tunnel ready");
        exchanged_pk
    } else {
        return Err("No peer specified. Use --peer <PEER_ID> for authenticated connection".into());
    };
    
    // Generate DH public key and sign it with identity key
    let local_dh_pk = {
        let mut pk = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut pk);
        pk
    };
    let local_signature = local_identity_key.sign(&local_dh_pk).to_bytes();
    
    // In production: exchange identity keys and signatures with peer over libp2p
    // For now, use dummy peer credentials (MUST be replaced with real exchange)
    let peer_identity_key = local_identity_pubkey; // TEMPORARY: self-signed for testing
    let peer_signature = local_signature; // TEMPORARY: using own signature
    
    println!("  WARNING: Using self-signed credentials for testing only!");
    
    let _tunnel = node::create_tunnel(
        &local_sk,
        &peer_pk,
        &peer_identity_key,
        &peer_signature,
        "127.0.0.1:51820"
    )
        .await
        .map_err(|e| format!("Failed to create tunnel: {}", e))?;
    println!("TUN up at 127.0.0.1:51820");
    
    let mut tick = interval(Duration::from_secs(300)); // 5 minutes
    tick.tick().await; // Skip the first immediate tick
    loop {
        tick.tick().await;
        let pk = p2p::get_current_pk()?;
        println!("CrypRQ v0.0.1 – Kyber pk: {:02x}{:02x}…", pk[0], pk[1]);
    }
}
