// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Peer-to-peer networking for VPN tunnels
//!
//! This crate provides libp2p-based peer discovery and connection management.

use anyhow::Result;
use libp2p::{
    connection_limits::{Behaviour as ConnectionLimitBehaviour, ConnectionLimits},
    identity, mdns,
    multiaddr::Protocol,
    noise, ping,
    swarm::{dial_opts::DialOpts, ListenError, NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use log::{info, warn};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    convert::Infallible,
    env,
    sync::Mutex,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::RwLock;

mod metrics;

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

#[derive(Debug, Clone)]
struct BackoffConfig {
    base_ms: u64,
    max_ms: u64,
}

static BACKOFF_CONFIG: Lazy<BackoffConfig> = Lazy::new(|| BackoffConfig {
    base_ms: read_env_u64("CRYPRQ_BACKOFF_BASE_MS").unwrap_or(500),
    max_ms: read_env_u64("CRYPRQ_BACKOFF_MAX_MS").unwrap_or(30_000),
});

#[derive(Debug)]
struct BackoffState {
    failures: u32,
    next_allowed: Instant,
    last_warn: Option<Instant>,
}

static BACKOFF: Lazy<Mutex<HashMap<String, BackoffState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const MAX_BACKOFF_FAILURES: u32 = 12;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
pub struct MyBehaviour {
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
    limits: ConnectionLimitBehaviour,
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

impl From<Infallible> for MyBehaviourEvent {
    fn from(_: Infallible) -> Self {
        unreachable!("infallible event cannot occur")
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
        metrics::record_rotation();
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
        .with_behaviour(
            |key| -> Result<MyBehaviour, Box<dyn std::error::Error + Send + Sync + 'static>> {
                let peer_id = PeerId::from(key.public());
                let mdns_behaviour = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
                Ok(MyBehaviour {
                    mdns: mdns_behaviour,
                    ping: ping::Behaviour::new(ping::Config::new()),
                    limits: connection_limits_behaviour(),
                })
            },
        )?
        .with_swarm_config(|c| c)
        .build();
    metrics::mark_swarm_ready();
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
    metrics::set_current_peers(0);
    let mut peer_connections: HashMap<PeerId, u32> = HashMap::new();

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                metrics::record_handshake_success();
                if endpoint.is_listener() {
                    clear_backoff_for(endpoint.get_remote_address());
                }
                *peer_connections.entry(peer_id).or_insert(0) += 1;
                metrics::set_current_peers(peer_connections.len());
                println!("Inbound connection established with {peer_id} via {endpoint:?}");
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                if let Some(count) = peer_connections.get_mut(&peer_id) {
                    if *count <= 1 {
                        peer_connections.remove(&peer_id);
                    } else {
                        *count -= 1;
                    }
                    metrics::set_current_peers(peer_connections.len());
                }
                println!("Connection with {peer_id} closed");
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                send_back_addr,
                ..
            } => {
                metrics::record_handshake_attempt();
                if allow_incoming(&send_back_addr) {
                    println!("Incoming connection attempt from {send_back_addr}");
                } else {
                    let _ = swarm.close_connection(connection_id);
                }
            }
            SwarmEvent::IncomingConnectionError {
                send_back_addr,
                error,
                ..
            } => {
                metrics::record_handshake_failure();
                record_failure(&send_back_addr);
                if matches!(error, ListenError::Denied { .. }) {
                    warn!(
                        "event=inbound_rate_limit addr=\"{send_back_addr}\" message=\"connection denied\""
                    );
                }
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

fn read_env_u64(key: &str) -> Option<u64> {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|v| *v > 0)
}

fn compute_backoff_duration(base_ms: u64, max_ms: u64, failures: u32) -> Duration {
    if failures == 0 || base_ms == 0 {
        return Duration::from_millis(0);
    }
    let exponent = (failures - 1).min(30);
    let multiplier = 1u128 << exponent;
    let mut ms = (base_ms as u128).saturating_mul(multiplier);
    if max_ms > 0 {
        ms = ms.min(max_ms as u128);
    }
    Duration::from_millis(ms.min(u64::MAX as u128) as u64)
}

fn connection_limits_behaviour() -> ConnectionLimitBehaviour {
    ConnectionLimitBehaviour::new(connection_limits_config())
}

fn connection_limits_config() -> ConnectionLimits {
    let max_inbound = read_env_u64("CRYPRQ_MAX_INBOUND")
        .unwrap_or(64)
        .min(u64::from(u32::MAX)) as u32;
    ConnectionLimits::default()
        .with_max_pending_incoming(Some(max_inbound))
        .with_max_established_incoming(Some(max_inbound))
}

pub async fn dial_peer(addr: String) -> Result<()> {
    let mut swarm = init_swarm()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init swarm: {}", e))?;
    let local_peer_id = *swarm.local_peer_id();
    println!("Local peer id: {local_peer_id}");
    metrics::set_current_peers(0);
    let mut peer_connections: HashMap<PeerId, u32> = HashMap::new();

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
        metrics::record_handshake_attempt();
        let opts = DialOpts::peer_id(peer_id)
            .addresses(vec![dial_addr])
            .build();
        swarm.dial(opts)?;
    } else {
        metrics::record_handshake_attempt();
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
                metrics::record_handshake_success();
                clear_backoff_for(endpoint.get_remote_address());
                *peer_connections.entry(remote).or_insert(0) += 1;
                metrics::set_current_peers(peer_connections.len());
                println!("Connected to {remote} via {endpoint:?}");
                break;
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                metrics::record_handshake_failure();
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
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                if let Some(count) = peer_connections.get_mut(&peer_id) {
                    if *count <= 1 {
                        peer_connections.remove(&peer_id);
                    } else {
                        *count -= 1;
                    }
                    metrics::set_current_peers(peer_connections.len());
                }
                println!("Connection with {peer_id} closed");
            }
            other => {
                println!("Unhandled event: {other:?}");
            }
        }
    }
    metrics::set_current_peers(0);
    Ok(())
}

fn allow_incoming(addr: &Multiaddr) -> bool {
    let key = addr.to_string();
    let now = Instant::now();
    let mut guard = BACKOFF.lock().expect("backoff lock poisoned");

    if let Some(entry) = guard.get_mut(&key) {
        if now < entry.next_allowed {
            let should_warn = entry.last_warn.map_or(true, |last| {
                now.duration_since(last) > Duration::from_secs(60)
            });
            if should_warn {
                let wait_ms = entry
                    .next_allowed
                    .saturating_duration_since(now)
                    .as_millis();
                warn!("event=inbound_backoff addr=\"{key}\" wait_ms={wait_ms}");
                entry.last_warn = Some(now);
            }
            return false;
        } else {
            guard.remove(&key);
        }
    }

    true
}

fn record_failure(addr: &Multiaddr) {
    let key = addr.to_string();
    let now = Instant::now();
    let mut guard = BACKOFF.lock().expect("backoff lock poisoned");
    let entry = guard.entry(key).or_insert(BackoffState {
        failures: 0,
        next_allowed: now,
        last_warn: None,
    });
    entry.failures = (entry.failures + 1).min(MAX_BACKOFF_FAILURES);
    let backoff = compute_backoff_duration(
        BACKOFF_CONFIG.base_ms,
        BACKOFF_CONFIG.max_ms,
        entry.failures,
    );
    entry.next_allowed = now + backoff;
    entry.last_warn = None;
}

fn clear_backoff_for(addr: &Multiaddr) {
    let key = addr.to_string();
    let mut guard = BACKOFF.lock().expect("backoff lock poisoned");
    guard.remove(&key);
}

pub use metrics::spawn_metrics_server;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod backoff_tests {
    use super::compute_backoff_duration;
    use std::time::Duration;

    #[test]
    fn backoff_doubles_until_cap() {
        let base = 100u64;
        let max = 2_000u64;
        let d1 = compute_backoff_duration(base, max, 1);
        let d2 = compute_backoff_duration(base, max, 2);
        let d5 = compute_backoff_duration(base, max, 5);
        assert_eq!(d1, Duration::from_millis(100));
        assert_eq!(d2, Duration::from_millis(200));
        assert_eq!(d5, Duration::from_millis(1_600));
    }

    #[test]
    fn backoff_respects_max() {
        let d = compute_backoff_duration(500, 2_000, 10);
        assert_eq!(d, Duration::from_millis(2_000));
    }

    #[test]
    fn zero_failures_zero_backoff() {
        let d = compute_backoff_duration(500, 2_000, 0);
        assert_eq!(d, Duration::from_millis(0));
    }
}
