// Copyright (c) 2026 Thor Thor
// License: MIT (see LICENSE file for details)

//! A bounded Circuit Relay v2 server for opt-in relayed direct-file transfers.
//!
//! The relay only forwards connections between peers. Each relayed connection
//! is authenticated end to end with Noise between the two peers' transport
//! keys, and the CrypRQ hybrid handshake and record encryption run inside it,
//! so the relay never holds a session key, a trust store or file plaintext.
//! It can still observe metadata (peer IDs, addresses, timing, sizes) and can
//! drop or delay traffic. Admission is bounded but not restricted to known
//! peers in this version; run it only where its operator controls who can
//! reach it.

use crate::direct_file::validate_address;
use crate::libp2p::{
    connection_limits::{self, ConnectionLimits},
    relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr,
};
use crate::secure_vpn::transport_keypair;
use anyhow::{anyhow, bail, Result};
use futures::StreamExt;
use node::NodeIdentity;
use std::convert::Infallible;
use std::time::Duration;

/// Relay resource limits. The library defaults (2 minutes, 128 KiB per
/// circuit) are far below a direct-file transfer, so every limit is explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayLimits {
    /// Bytes a single circuit may carry before the relay closes it.
    pub max_circuit_bytes: u64,
    /// How long a single circuit may stay open.
    pub max_circuit_duration: Duration,
    /// Concurrent circuits across all peers.
    pub max_circuits: usize,
    /// Concurrent reservations across all peers.
    pub max_reservations: usize,
}

impl RelayLimits {
    pub const MAX_CIRCUIT_BYTES: u64 = 2 << 30;
    pub const MAX_CIRCUIT_DURATION: Duration = Duration::from_secs(24 * 60 * 60);
    pub const MAX_SLOTS: usize = 1024;

    pub fn validate(&self) -> Result<()> {
        if self.max_circuit_bytes == 0 || self.max_circuit_bytes > Self::MAX_CIRCUIT_BYTES {
            bail!("relay circuit byte limit must be between 1 byte and 2 GiB");
        }
        if self.max_circuit_duration.is_zero()
            || self.max_circuit_duration > Self::MAX_CIRCUIT_DURATION
        {
            bail!("relay circuit duration must be between 1 second and 24 hours");
        }
        if !(1..=Self::MAX_SLOTS).contains(&self.max_circuits)
            || !(1..=Self::MAX_SLOTS).contains(&self.max_reservations)
        {
            bail!("relay circuit and reservation counts must be between 1 and 1024");
        }
        Ok(())
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event", prelude = "libp2p_swarm::derive_prelude")]
struct Behaviour {
    relay: relay::Behaviour,
    limits: connection_limits::Behaviour,
}

#[derive(Debug)]
enum Event {
    Relay(relay::Event),
}

impl From<relay::Event> for Event {
    fn from(event: relay::Event) -> Self {
        Self::Relay(event)
    }
}
impl From<Infallible> for Event {
    fn from(event: Infallible) -> Self {
        match event {}
    }
}

/// Run the relay until the future is dropped or a listener fails.
///
/// `external` lists the addresses peers use to reach this relay; they are
/// returned in reservations. When it is empty, each concrete listen address is
/// advertised instead, which suits only a relay whose listen addresses are
/// themselves reachable. `ready` receives the advertised addresses.
pub async fn serve_relay(
    identity: &NodeIdentity,
    listen: Vec<Multiaddr>,
    external: Vec<Multiaddr>,
    limits: RelayLimits,
    ready: Option<tokio::sync::oneshot::Sender<Vec<Multiaddr>>>,
) -> Result<Infallible> {
    limits.validate()?;
    if listen.is_empty() {
        bail!("relay needs at least one explicit listen address");
    }
    for address in listen.iter().chain(&external) {
        validate_address(address)?;
    }
    let keypair = transport_keypair(identity)?;
    let local = keypair.public().to_peer_id();
    let config = relay::Config {
        max_reservations: limits.max_reservations,
        max_reservations_per_peer: 2,
        reservation_duration: Duration::from_secs(60 * 60),
        max_circuits: limits.max_circuits,
        max_circuits_per_peer: 2,
        max_circuit_duration: limits.max_circuit_duration,
        max_circuit_bytes: limits.max_circuit_bytes,
        ..Default::default()
    };
    let connection_limits = ConnectionLimits::default()
        .with_max_pending_incoming(Some(64))
        .with_max_established_per_peer(Some(4))
        .with_max_established(Some(
            (limits.max_reservations + 2 * limits.max_circuits).min(4096) as u32,
        ));
    let behaviour = Behaviour {
        relay: relay::Behaviour::new(local, config),
        limits: connection_limits::Behaviour::new(connection_limits),
    };
    let mut swarm = crate::libp2p::build_swarm(keypair, behaviour, |c| {
        c.with_idle_connection_timeout(Duration::from_secs(60))
    })
    .map_err(|_| anyhow!("cannot initialize relay transport"))?;
    let advertise_listen = external.is_empty();
    for address in external {
        swarm.add_external_address(address);
    }
    // Ready once every listener has reported an address. A wildcard listener
    // reports one address per interface, so count listeners, not addresses.
    let mut pending = std::collections::HashSet::new();
    for address in listen {
        pending.insert(swarm.listen_on(address)?);
    }
    let mut announced = false;
    let mut ready = ready;
    log::info!("event=relay_starting peer_id={local}");
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                use crate::libp2p::multiaddr::Protocol;
                if advertise_listen {
                    match address.iter().next() {
                        Some(Protocol::Ip4(ip)) if !ip.is_unspecified() => {
                            swarm.add_external_address(address.clone())
                        }
                        Some(Protocol::Ip6(ip)) if !ip.is_unspecified() => {
                            swarm.add_external_address(address.clone())
                        }
                        _ => {}
                    }
                }
                log::info!("event=relay_listening address={address}");
                pending.remove(&listener_id);
                if pending.is_empty() && !announced {
                    announced = true;
                    let advertised: Vec<Multiaddr> = swarm.external_addresses().cloned().collect();
                    if advertised.is_empty() {
                        bail!(
                            "relay has no reachable address to advertise; pass --external-address"
                        );
                    }
                    for address in &advertised {
                        log::info!("event=relay_advertised address={address}");
                    }
                    if let Some(ready) = ready.take() {
                        let _ = ready.send(advertised);
                    }
                }
            }
            SwarmEvent::Behaviour(Event::Relay(event)) => log_relay_event(&event),
            SwarmEvent::ListenerError { .. } | SwarmEvent::ListenerClosed { .. } => {
                bail!("relay listener failed")
            }
            _ => {}
        }
    }
}

/// Operational events only: no payload, and peer IDs at debug level.
fn log_relay_event(event: &relay::Event) {
    use relay::Event::*;
    match event {
        ReservationReqAccepted { src_peer_id, .. } => {
            log::info!("event=relay_reservation_accepted");
            log::debug!("relay reservation peer={src_peer_id}");
        }
        ReservationReqDenied { .. } => log::warn!("event=relay_reservation_denied"),
        ReservationTimedOut { .. } => log::info!("event=relay_reservation_expired"),
        CircuitReqAccepted {
            src_peer_id,
            dst_peer_id,
        } => {
            log::info!("event=relay_circuit_opened");
            log::debug!("relay circuit src={src_peer_id} dst={dst_peer_id}");
        }
        CircuitReqDenied { .. } => log::warn!("event=relay_circuit_denied"),
        CircuitClosed { error, .. } => {
            log::info!("event=relay_circuit_closed error={}", error.is_some())
        }
        // Includes failures to reach a reserved peer; carries peer IDs, so debug.
        other => log::debug!("relay event: {other:?}"),
    }
}
