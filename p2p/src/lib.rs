// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Peer-to-peer networking for VPN tunnels
//!
//! This crate provides libp2p-based peer discovery and connection management.

use anyhow::Result;
use libp2p::{
    identity, mdns,
    multiaddr::Protocol,
    noise, ping,
    swarm::{dial_opts::DialOpts, NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use log::info;
use once_cell::sync::Lazy;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

// Import the *public* items from the crypto crate
use cryprq_crypto::{kyber_keypair, KyberPublicKey, KyberSecretKey};

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
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Mdns(mdns::Event),
    Ping(ping::Event),
}

impl From<mdns::Event> for MyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        MyBehaviourEvent::Mdns(event)
    }
}

impl From<ping::Event> for MyBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        MyBehaviourEvent::Ping(event)
    }
}

// Public function to get the current key
pub async fn get_current_pk() -> Result<KyberPublicKey, P2PError> {
    KEYS.read()
        .await
        .as_ref()
        .map(|(pk, _)| *pk)
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

pub async fn init_swarm(
) -> Result<Swarm<MyBehaviour>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let local_key = identity::Keypair::generate_ed25519();
    let swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let peer_id = PeerId::from(key.public());
            let mdns_behaviour = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
            Ok(MyBehaviour {
                mdns: mdns_behaviour,
                ping: ping::Behaviour::new(ping::Config::new()),
            })
        })?
        .with_swarm_config(|c| c)
        .build();
    Ok(swarm)
}

pub async fn start_listener(addr: &str) -> Result<()> {
    let mut swarm = init_swarm()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init swarm: {}", e))?;
    let local_peer_id = *swarm.local_peer_id();
    println!("Local peer id: {local_peer_id}");

    let listen_addr: Multiaddr = addr.parse()?;
    swarm.listen_on(listen_addr)?;

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                println!("Inbound connection established with {peer_id} via {endpoint:?}");
            }
            SwarmEvent::IncomingConnection { send_back_addr, .. } => {
                println!("Incoming connection attempt from {send_back_addr}");
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                println!("Incoming connection error: {error:?}");
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(event)) => {
                println!("Ping event: {event:?}");
            }
            other => {
                println!("Unhandled event: {other:?}");
            }
        }
    }
}

pub async fn dial_peer(addr: String) -> Result<()> {
    let mut swarm = init_swarm()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init swarm: {}", e))?;
    let local_peer_id = *swarm.local_peer_id();
    println!("Local peer id: {local_peer_id}");

    let mut dial_addr: Multiaddr = addr.parse()?;
    let peer_id = if matches!(dial_addr.iter().last(), Some(Protocol::P2p(_))) {
        match dial_addr.pop() {
            Some(Protocol::P2p(peer_id)) => Some(peer_id),
            _ => None,
        }
    } else {
        None
    };

    if let Some(peer_id) = peer_id {
        let opts = DialOpts::peer_id(peer_id)
            .addresses(vec![dial_addr])
            .build();
        swarm.dial(opts)?;
    } else {
        swarm.dial(dial_addr)?;
    }

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished {
                peer_id: remote,
                endpoint,
                ..
            } => {
                println!("Connected to {remote} via {endpoint:?}");
                break;
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                anyhow::bail!("Dial error: {error:?}");
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(event)) => {
                println!("Ping event: {event:?}");
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                println!("Dialing {peer_id:?} (connection {connection_id:?})");
            }
            other => {
                println!("Unhandled event: {other:?}");
            }
        }
    }
    Ok(())
}
