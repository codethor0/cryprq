// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Peer-to-peer networking for VPN tunnels
//!
//! This crate provides libp2p-based peer discovery and connection management.

use anyhow::{Context, Result};
use libp2p::{
    connection_limits::{Behaviour as ConnectionLimitBehaviour, ConnectionLimits},
    identity, mdns,
    multiaddr::Protocol,
    noise, ping,
    swarm::{dial_opts::DialOpts, NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use log::{info, warn};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    env,
    str::FromStr,
    sync::Mutex,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::MissedTickBehavior;

// Import the *public* items from the crypto crate
use cryprq_crypto::{kyber_keypair, KyberPublicKey, KyberSecretKey, PPKStore, PostQuantumPSK};

mod metrics;
pub use metrics::start_metrics_server;

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
static ALLOWED_PEERS: Lazy<RwLock<Option<HashSet<PeerId>>>> = Lazy::new(|| RwLock::new(None));
// PPK store for post-quantum pre-shared keys
static PPK_STORE: Lazy<RwLock<PPKStore>> = Lazy::new(|| RwLock::new(PPKStore::new()));
static BACKOFF_CONFIG: Lazy<BackoffConfig> = Lazy::new(|| BackoffConfig {
    base_ms: read_env_u64("CRYPRQ_BACKOFF_BASE_MS").unwrap_or(500),
    max_ms: read_env_u64("CRYPRQ_BACKOFF_MAX_MS").unwrap_or(30_000),
});
static BACKOFF: Lazy<Mutex<HashMap<String, BackoffState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    Limits(Infallible),
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
        unreachable!("connection limit behaviour is infallible")
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

pub async fn set_allowed_peers(peers: &[String]) -> anyhow::Result<()> {
    let mut set = HashSet::with_capacity(peers.len());
    for entry in peers {
        let peer_id =
            PeerId::from_str(entry).with_context(|| format!("invalid peer id '{}'", entry))?;
        set.insert(peer_id);
    }
    let mut guard = ALLOWED_PEERS.write().await;
    *guard = Some(set);
    Ok(())
}

async fn peer_is_allowed(peer_id: &PeerId) -> bool {
    ALLOWED_PEERS
        .read()
        .await
        .as_ref()
        .map(|set| set.contains(peer_id))
        .unwrap_or(true)
}

// Public function to start key rotation
pub async fn start_key_rotation(interval: Duration) {
    metrics::set_rotation_interval(interval);

    info!(
        "event=rotation_task_started interval_secs={}",
        interval.as_secs()
    );

    rotate_once(interval).await;

    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;
        rotate_once(interval).await;
    }
}

async fn rotate_once(interval: Duration) {
    let start = Instant::now();
    let (pk, sk) = kyber_keypair();

    let mut guard = KEYS.write().await;
    guard.replace((pk, sk));

    // Cleanup expired PPKs on rotation
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut ppk_store = PPK_STORE.write().await;
    ppk_store.cleanup_expired(now);

    let elapsed = start.elapsed();
    let epoch = metrics::record_rotation_success(elapsed);

    info!(
        "event=key_rotation status=success epoch={} duration_ms={} interval_secs={}",
        epoch,
        elapsed.as_millis(),
        interval.as_secs()
    );
}

/// Derive and store a PPK for a peer after ML-KEM key exchange
///
/// # Arguments
///
/// * `kyber_shared` - Shared secret from ML-KEM encapsulation (32 bytes)
/// * `peer_id_bytes` - Peer identity bytes (32 bytes, typically Ed25519 public key)
/// * `rotation_interval_secs` - Key rotation interval in seconds
pub async fn derive_and_store_ppk(
    kyber_shared: &[u8; 32],
    peer_id_bytes: &[u8; 32],
    rotation_interval_secs: u64,
) {
    use rand::RngCore;

    // Generate random salt for this PPK derivation
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let ppk = PostQuantumPSK::derive(
        kyber_shared,
        peer_id_bytes,
        &salt,
        rotation_interval_secs,
        now,
    );

    let mut store = PPK_STORE.write().await;
    store.store(ppk);

    info!(
        "event=ppk_derived peer_id={:x} expires_in_secs={}",
        peer_id_bytes
            .iter()
            .take(8)
            .fold(0u64, |acc, &b| (acc << 8) | b as u64),
        rotation_interval_secs
    );
}

/// Get PPK for a peer (if available and not expired)
pub async fn get_ppk_for_peer(peer_id_bytes: &[u8; 32]) -> Option<PostQuantumPSK> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let store = PPK_STORE.read().await;
    store.get(peer_id_bytes, now).cloned()
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
                limits: connection_limits_behaviour(),
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
    metrics::mark_swarm_initialized();

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if !peer_is_allowed(&peer_id).await {
                    metrics::record_handshake_failure();
                    warn!(
                        "event=peer_denied peer_id={} reason=not_allowlisted",
                        peer_id
                    );
                    let _ = swarm.disconnect_peer_id(peer_id);
                } else {
                    metrics::record_handshake_success();
                    metrics::inc_active_peers();
                    clear_backoff_for(endpoint.get_remote_address());
                    println!("Inbound connection established with {peer_id} via {endpoint:?}");
                }
            }
            SwarmEvent::IncomingConnection { send_back_addr, .. } => {
                metrics::record_handshake_attempt();
                if allow_incoming(&send_back_addr) {
                    println!("Incoming connection attempt from {send_back_addr}");
                } else {
                    println!("Incoming connection denied via backoff for {send_back_addr}");
                }
            }
            SwarmEvent::IncomingConnectionError {
                send_back_addr,
                error,
                ..
            } => {
                metrics::record_handshake_failure();
                record_failure(&send_back_addr);
                println!("Incoming connection error: {error:?}");
            }
            SwarmEvent::ConnectionClosed { .. } => {
                metrics::dec_active_peers();
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(event)) => {
                println!("Ping event: {event:?}");
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Limits(_)) => {}
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
        if !peer_is_allowed(&peer_id).await {
            anyhow::bail!("Peer {peer_id} is not in the allow list");
        }
        let opts = DialOpts::peer_id(peer_id)
            .addresses(vec![dial_addr])
            .build();
        swarm.dial(opts)?;
    } else {
        swarm.dial(dial_addr)?;
    }

    metrics::mark_swarm_initialized();

    use libp2p::futures::StreamExt;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished {
                peer_id: remote,
                endpoint,
                ..
            } => {
                if !peer_is_allowed(&remote).await {
                    metrics::record_handshake_failure();
                    warn!(
                        "event=peer_denied peer_id={} reason=not_allowlisted",
                        remote
                    );
                    let _ = swarm.disconnect_peer_id(remote);
                } else {
                    metrics::record_handshake_success();
                    clear_backoff_for(endpoint.get_remote_address());
                    println!("Connected to {remote} via {endpoint:?}");
                    // Don't break - keep connection alive for VPN mode
                    // The connection will stay active
                }
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
                metrics::record_handshake_attempt();
                println!("Dialing {peer_id:?} (connection {connection_id:?})");
            }
            SwarmEvent::ConnectionClosed { .. } => {
                // no active peer tracking for dialer since we exit on connect
            }
            other => {
                println!("Unhandled event: {other:?}");
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct BackoffConfig {
    base_ms: u64,
    max_ms: u64,
}

#[derive(Debug)]
struct BackoffState {
    failures: u32,
    next_allowed: Instant,
    last_warn: Option<Instant>,
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

fn allow_incoming(addr: &Multiaddr) -> bool {
    let key = addr.to_string();
    let now = Instant::now();
    let mut guard = match BACKOFF.lock() {
        Ok(g) => g,
        Err(e) => {
            warn!("event=backoff_lock_poisoned error=\"{}\"", e);
            return true; // Allow on lock poisoning (fail open)
        }
    };

    if let Some(state) = guard.get_mut(&key) {
        if now < state.next_allowed {
            let should_warn = state.last_warn.map_or(true, |last| {
                now.duration_since(last) > Duration::from_secs(60)
            });
            if should_warn {
                warn!(
                    "event=inbound_backoff addr=\"{key}\" wait_ms={}",
                    state
                        .next_allowed
                        .saturating_duration_since(now)
                        .as_millis()
                );
                state.last_warn = Some(now);
            }
            return false;
        }
        guard.remove(&key);
    }
    true
}

fn record_failure(addr: &Multiaddr) {
    let key = addr.to_string();
    let now = Instant::now();
    let mut guard = match BACKOFF.lock() {
        Ok(g) => g,
        Err(e) => {
            warn!("event=backoff_lock_poisoned error=\"{}\"", e);
            return; // Skip recording on lock poisoning
        }
    };
    let state = guard.entry(key).or_insert(BackoffState {
        failures: 0,
        next_allowed: now,
        last_warn: None,
    });
    state.failures = (state.failures + 1).min(16);
    let backoff = compute_backoff_duration(
        BACKOFF_CONFIG.base_ms,
        BACKOFF_CONFIG.max_ms,
        state.failures,
    );
    state.next_allowed = now + backoff;
    state.last_warn = None;
}

fn clear_backoff_for(addr: &Multiaddr) {
    let key = addr.to_string();
    let mut guard = match BACKOFF.lock() {
        Ok(g) => g,
        Err(e) => {
            warn!("event=backoff_lock_poisoned error=\"{}\"", e);
            return; // Skip clearing on lock poisoning
        }
    };
    guard.remove(&key);
}

fn connection_limits_config() -> ConnectionLimits {
    let max_inbound = read_env_u64("CRYPRQ_MAX_INBOUND")
        .unwrap_or(64)
        .min(u64::from(u32::MAX)) as u32;
    ConnectionLimits::default()
        .with_max_pending_incoming(Some(max_inbound))
        .with_max_established_incoming(Some(max_inbound))
}

fn connection_limits_behaviour() -> ConnectionLimitBehaviour {
    ConnectionLimitBehaviour::new(connection_limits_config())
}
