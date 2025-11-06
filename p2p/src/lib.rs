//! Peer-to-peer networking for VPN tunnels
//!
//! This crate provides libp2p-based peer discovery and connection management.
//!
//! # Features
//!
//! - mDNS local peer discovery
//! - QUIC transport (UDP-based, multiplexed)
//! - Automatic Kyber768 key rotation (5 minute intervals)
//! - Secure key zeroization on rotation
//!
//! # Architecture
//!
//! Uses libp2p Swarm with:
//! - QUIC for transport
//! - mDNS for local network peer discovery
//! - Global key storage with RwLock protection
//!
//! # Example
//!
//! ```no_run
//! use p2p::{get_current_pk, start_key_rotation};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Start automatic key rotation
//! tokio::spawn(start_key_rotation());
//!
//! // Get current public key
//! let pk = get_current_pk()?;
//! println!("Public key: {} bytes", pk.len());
//! # Ok(())
//! # }
//! ```

use libp2p::{
    identity, mdns,
    swarm::{NetworkBehaviour, Swarm},
    PeerId, SwarmBuilder,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;
use zeroize::Zeroize;
use cryprq_crypto::make_kyber_keys;

mod error;
pub use error::P2PError;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    mdns: mdns::tokio::Behaviour,
}

pub struct Connection;

type KeyPair = (Vec<u8>, Vec<u8>);
type SharedKeys = Arc<RwLock<KeyPair>>;

static KEYS: once_cell::sync::Lazy<SharedKeys> = 
    once_cell::sync::Lazy::new(|| {
        let (pk, sk) = make_kyber_keys();
        Arc::new(RwLock::new((pk.to_vec(), sk.to_vec())))
    });

/// Establish connection to remote peer
///
/// # Arguments
///
/// * `peer_id` - libp2p peer identifier to connect to
///
/// # Returns
///
/// Returns `Connection` handle on success, or I/O error on failure.
///
/// # Status
///
/// Stub implementation - returns success immediately.
/// Production version will perform actual QUIC connection.
pub async fn dial_peer(_peer_id: PeerId) -> Result<Connection, std::io::Error> {
    // TODO: implement actual dialing
    Ok(Connection)
}

/// Get current Kyber768 public key
///
/// Returns the current post-quantum public key for key exchange.
/// Keys are automatically rotated every 5 minutes.
///
/// # Returns
///
/// Returns public key bytes on success, or `P2PError::LockPoisoned` if
/// the key storage lock is poisoned.
///
/// # Example
///
/// ```
/// use p2p::get_current_pk;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pk = get_current_pk()?;
/// assert_eq!(pk.len(), 32); // Stub implementation
/// # Ok(())
/// # }
/// ```
pub fn get_current_pk() -> Result<Vec<u8>, P2PError> {
    KEYS.read()
        .map(|guard| guard.0.clone())
        .map_err(|e| P2PError::LockPoisoned(e.to_string()))
}

/// Start automatic key rotation background task
///
/// Rotates Kyber768 keys every 5 minutes for forward secrecy.
/// Old keys are securely zeroized before replacement.
///
/// This function runs indefinitely and should be spawned as a task:
///
/// ```no_run
/// use p2p::start_key_rotation;
///
/// # async fn example() {
/// tokio::spawn(start_key_rotation());
/// # }
/// ```
///
/// # Security
///
/// - Keys zeroized before replacement (prevents memory dumps)
/// - 5 minute rotation interval balances security vs performance
/// - Continues running even if key generation fails
pub async fn start_key_rotation() {
    let mut interval = time::interval(Duration::from_secs(300)); // 5 minutes
    interval.tick().await; // Skip the first immediate tick
    loop {
        interval.tick().await;
        let (new_pk, new_sk) = make_kyber_keys();
        
        if let Ok(mut guard) = KEYS.write() {
            guard.0.zeroize();
            guard.1.zeroize();
            *guard = (new_pk.to_vec(), new_sk.to_vec());
        }
        
        println!(" ransom rotate");
    }
}

pub async fn init_swarm() -> Result<Swarm<Behaviour>, Box<dyn std::error::Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    
    let behaviour = Behaviour {
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

#[cfg(test)]
mod tests;
