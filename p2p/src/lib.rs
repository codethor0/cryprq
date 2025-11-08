//! Peer-to-peer networking for VPN tunnels
//!
//! This crate provides libp2p-based peer discovery and connection management.

use anyhow::Result;
use libp2p::{
    identity, mdns, PeerId, Swarm, SwarmBuilder,
    swarm::{dummy, NetworkBehaviour, SwarmEvent},
    Multiaddr,
};
use log::{error, info};
use std::time::Duration;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use thiserror::Error;
use void::Void;

// Import the *public* items from the crypto crate
use cryprq_crypto::{KyberPublicKey, KyberSecretKey, kyber_keypair};

// Define the error type correctly
#[derive(Debug, Error)]
pub enum P2PError {
    #[error("Key generation failed: {0}")]
    KeyGenFailed(String),
    #[error("Keys not initialized")]
    NotInitialized,
}

// Define the global key store
static KEYS: Lazy<RwLock<Option<(KyberPublicKey, KyberSecretKey)>>> =
    Lazy::new(|| RwLock::new(None));

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
pub struct MyBehaviour {
    cryprq: dummy::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Cryprq(Void),
    Mdns(mdns::Event),
}

impl From<Void> for MyBehaviourEvent {
    fn from(event: Void) -> Self {
        match event {}
    }
}

impl From<mdns::Event> for MyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        MyBehaviourEvent::Mdns(event)
    }
}

// Public function to get the current key
pub async fn get_current_pk() -> Result<KyberPublicKey, P2PError> {
    KEYS.read()
        .await
        .as_ref()
        .map(|(pk, _)| pk.clone())
        .ok_or(P2PError::NotInitialized)
}

// Public function to start key rotation
pub async fn start_key_rotation() {
    info!("Starting key rotation task...");
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 mins
    loop {
        interval.tick().await;
        info!("Rotating Kyber keys...");
        let (pk, sk) = kyber_keypair();

        KEYS.write().await.replace((pk, sk));
        info!("Keys rotated successfully");
    }
}

pub async fn init_swarm() -> Result<Swarm<MyBehaviour>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    let behaviour = MyBehaviour {
        cryprq: dummy::Behaviour,
        mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
    };
    let swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c)
        .build();
    Ok(swarm)
}

pub async fn start_listener(addr: &str) -> Result<()> {
    let mut swarm = init_swarm().await.map_err(|e| anyhow::anyhow!("Failed to init swarm: {}", e))?;
    
    let listen_addr: Multiaddr = addr.parse()?;
    swarm.listen_on(listen_addr)?;

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            _ => {}
        }
    }
}

pub async fn dial_peer(addr: String) -> Result<()> {
    let mut swarm = init_swarm().await.map_err(|e| anyhow::anyhow!("Failed to init swarm: {}", e))?;
    
    let dial_addr: Multiaddr = addr.parse()?;
    swarm.dial(dial_addr)?;

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { peer_id: remote, .. } => {
                println!("Connected to {remote}");
                break;
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                anyhow::bail!("Dial error: {error}");
            }
            _ => {}
        }
    }
    Ok(())
}
