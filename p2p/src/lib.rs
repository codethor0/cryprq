<<<<<<< HEAD
/// Peer-to-peer networking for VPN tunnels
use anyhow::Result;
use libp2p::{
	identity,
	swarm::{Swarm, SwarmEvent},
	Multiaddr, PeerId,
	quic,
	Transport,
};
use libp2p::futures::StreamExt;
=======
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

use log;
use void;
use libp2p::{
    identity, mdns, PeerId, Swarm, SwarmBuilder,
    swarm::{dummy, NetworkBehaviour},
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;
use cryprq_crypto::make_kyber_keys;
>>>>>>> 504720d (p2p: move crate doc to top of file)


<<<<<<< HEAD
pub async fn start_listener(addr: &str) -> Result<()> {
	let id_keys = identity::Keypair::generate_ed25519();
	let peer_id = PeerId::from(id_keys.public());
	println!("Local PeerId: {peer_id}");

	let quic_config = quic::Config::new(&id_keys);
	let transport = quic::tokio::Transport::new(quic_config);

	let behaviour = libp2p::swarm::dummy::Behaviour;
	let boxed_transport = libp2p::Transport::map(
		transport,
		|(peer, muxer), _point| (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer))
	).boxed();
	let mut swarm = Swarm::new(boxed_transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());

	let listen_addr: Multiaddr = addr.parse()?;
	swarm.listen_on(listen_addr)?;

	loop {
		match swarm.next().await {
			Some(SwarmEvent::NewListenAddr { address, .. }) => {
				println!("Listening on {address}");
			}
			_ => {}
		}
	}
}

pub async fn dial_peer(addr: String) -> Result<()> {
	let id_keys = identity::Keypair::generate_ed25519();
	let peer_id = PeerId::from(id_keys.public());
	println!("Local PeerId: {peer_id}");

	let quic_config = quic::Config::new(&id_keys);
	let transport = quic::tokio::Transport::new(quic_config);
=======
#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub mdns: mdns::tokio::Behaviour,
}

type KeyPair = (Vec<u8>, Vec<u8>);
type SharedKeys = Arc<RwLock<KeyPair>>;
>>>>>>> 504720d (p2p: move crate doc to top of file)

	let behaviour = libp2p::swarm::dummy::Behaviour;
	let boxed_transport = libp2p::Transport::map(
		transport,
		|(peer, muxer), _point| (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer))
	).boxed();
	let mut swarm = Swarm::new(boxed_transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());

<<<<<<< HEAD
	let dial_addr: Multiaddr = addr.parse()?;
	swarm.dial(dial_addr)?;

	loop {
		match swarm.next().await {
			Some(SwarmEvent::ConnectionEstablished { peer_id: remote, .. }) => {
				println!("Connected to {remote}");
				break;
			}
			Some(SwarmEvent::OutgoingConnectionError { error, .. }) => {
				anyhow::bail!("Dial error: {error}");
			}
			_ => {}
		}
	}
	Ok(())
}
=======
/// Establish connection to remote peer
///
/// # Arguments
///

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
        let (pk, sk) = make_kyber_keys();
        // Replace old keys with new ones
        if let Err(e) = KEYS.write().map(|mut guard| {
            // Zeroize old keys
            guard.0.fill(0);
            guard.1.fill(0);
            // Replace with new keys
            guard.0.copy_from_slice(pk.as_ref());
            guard.1.copy_from_slice(sk.as_ref());
        }) {
            // Log error but keep running
            log::error!("Key rotation error: {}", e);
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
pub struct MyBehaviour {
    cryprq: dummy::Behaviour, // real NetworkBehaviour that does nothing
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Cryprq(void::Void),        // dummy never produces events
    Mdns(mdns::Event),
}

// convert dummy event (which is !) into our enum
impl From<void::Void> for MyBehaviourEvent {
    fn from(event: void::Void) -> Self {
        match event {}
    }
}
impl From<mdns::Event> for MyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        MyBehaviourEvent::Mdns(event)
    }
}
    pub async fn init_swarm() -> Result<Swarm<MyBehaviour>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        let behaviour = MyBehaviour {
            cryprq: dummy::Behaviour,   // no constructor, just the type
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
    // Example event loop for listen/dial (add to your listen/dial functions)
    // loop {
    //     match swarm.next_event().await {
    //         SwarmEvent::NewListenAddr { address, .. } => {
    //             println!("Listening on {}", address);
    //         }
    //         SwarmEvent::ConnectionEstablished { .. } => {
    //             println!("Connection established");
    //         }
    //         SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
    //             for (peer_id, _multiaddr) in list {
    //                 println!("Discovered peer: {}", peer_id);
    //             }
    //         }
    //         SwarmEvent::Behaviour(MyBehaviourEvent::Cryprq(event)) => {
    //             println!("Cryprq event: {:?}", event);
    //         }
    //         _ => {}
    //     }
    // }

#[cfg(test)]
mod tests;
>>>>>>> 504720d (p2p: move crate doc to top of file)
