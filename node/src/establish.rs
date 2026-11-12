// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! Runs the authenticated handshake over UDP and yields an established tunnel.

use crate::auth_handshake::{
    self, HandshakeError, Initiator, Responder, SessionKeys, MAX_MESSAGE_LEN, MSG_FINISH, MSG_INIT,
};
use crate::identity::{NodeIdentity, PeerPublicKey, TrustedPeers};
use crate::{RateLimiter, Session, Tunnel, TunnelError};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::timeout;

/// Default time allowed for one handshake attempt.
const DEFAULT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
/// Interval between retransmissions of an unanswered handshake message.
const RETRANSMIT_INTERVAL: Duration = Duration::from_millis(500);
/// Half-open handshakes a responder keeps at once.
const MAX_PENDING_HANDSHAKES: usize = 16;
/// Datagrams per second a listening responder processes before a session exists.
const ACCEPT_RATE: u32 = 200;

/// Identity and trust configuration for establishing tunnels.
#[derive(Clone)]
pub struct TunnelConfig {
    /// This node's persistent identity.
    pub identity: Arc<NodeIdentity>,
    /// The only peers this node will establish a session with.
    pub trusted_peers: TrustedPeers,
    /// Directory for received files.
    pub file_output_dir: Option<PathBuf>,
    /// Time allowed for a handshake attempt; also the lifetime of a half-open
    /// handshake on the responder.
    pub handshake_timeout: Duration,
}

impl TunnelConfig {
    /// Configuration with the default handshake timeout.
    pub fn new(identity: Arc<NodeIdentity>, trusted_peers: TrustedPeers) -> Self {
        Self {
            identity,
            trusted_peers,
            file_output_dir: None,
            handshake_timeout: DEFAULT_HANDSHAKE_TIMEOUT,
        }
    }
}

fn handshake_failed(err: HandshakeError) -> TunnelError {
    match err {
        HandshakeError::UntrustedPeer
        | HandshakeError::IdentityMismatch
        | HandshakeError::BadSignature => TunnelError::InvalidPeerIdentity,
        other => TunnelError::HandshakeFailed(other.to_string()),
    }
}

fn timed_out(stage: &str) -> TunnelError {
    TunnelError::HandshakeFailed(format!("timed out waiting for {stage}"))
}

fn into_tunnel(
    socket: UdpSocket,
    keys: SessionKeys,
    peer_addr: SocketAddr,
    config: &TunnelConfig,
    confirm_echo: Option<(Vec<u8>, Vec<u8>)>,
) -> Tunnel {
    log::info!(
        "event=handshake_complete role={:?} peer={} session_id={}",
        keys.role,
        keys.peer,
        hex::encode(keys.session_id)
    );
    let session = Session::from_handshake(keys);
    Tunnel::from_session(
        socket,
        session,
        Some(peer_addr),
        config.file_output_dir.clone(),
        confirm_echo,
    )
}

/// Establishes a tunnel as initiator with the pinned peer `expected_peer`
/// reachable at `peer_addr`.
///
/// Returns only after the responder's identity was verified against the pinned
/// key and both sides confirmed the session keys. Any failure, including a
/// timeout, yields an error and no tunnel.
pub async fn connect_tunnel(
    config: &TunnelConfig,
    socket: UdpSocket,
    peer_addr: SocketAddr,
    expected_peer: &PeerPublicKey,
) -> Result<Tunnel, TunnelError> {
    let mut handshake = Initiator::new(&config.identity, &config.trusted_peers, expected_peer)
        .map_err(handshake_failed)?;
    let deadline = Instant::now() + config.handshake_timeout;
    let mut buf = vec![0u8; MAX_MESSAGE_LEN + 1];
    let mut finish: Option<Vec<u8>> = None;

    loop {
        let outgoing = finish
            .as_deref()
            .unwrap_or_else(|| handshake.init_message());
        socket
            .send_to(outgoing, peer_addr)
            .await
            .map_err(|e| TunnelError::NetworkError(e.to_string()))?;

        let resend_at = Instant::now() + RETRANSMIT_INTERVAL;
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(timed_out(if finish.is_some() {
                    "key confirmation"
                } else {
                    "the peer's response"
                }));
            }
            let wait = resend_at.min(deadline).saturating_duration_since(now);
            let (len, from) = match timeout(wait, socket.recv_from(&mut buf)).await {
                Err(_) => break, // retransmit
                Ok(Err(e)) => return Err(TunnelError::NetworkError(e.to_string())),
                Ok(Ok(received)) => received,
            };
            if from != peer_addr {
                continue;
            }
            let result = if finish.is_none() {
                handshake.handle_response(&buf[..len]).map(|msg| {
                    finish = Some(msg);
                    None
                })
            } else {
                handshake.handle_confirm(&buf[..len]).map(Some)
            };
            match result {
                Ok(Some(keys)) => return Ok(into_tunnel(socket, keys, peer_addr, config, None)),
                Ok(None) => break, // send FINISH
                Err(e) if e.is_fatal() => return Err(handshake_failed(e)),
                Err(e) => log::debug!("ignoring handshake datagram from {from}: {e}"),
            }
        }
    }
}

struct PendingHandshake<'a> {
    responder: Responder<'a>,
    init: Vec<u8>,
    expires: Instant,
}

/// Waits on `socket` until a pinned peer completes the handshake as initiator
/// and returns the established tunnel.
///
/// Unpinned, malformed and unauthenticated datagrams are dropped without a
/// reply, and anything that is not a handshake message is ignored, so no data
/// is accepted before a session exists. The number of half-open handshakes is
/// bounded. The future can be dropped (for example by `tokio::time::timeout`)
/// to stop listening.
pub async fn accept_tunnel(
    config: &TunnelConfig,
    socket: UdpSocket,
) -> Result<Tunnel, TunnelError> {
    if config.trusted_peers.is_empty() {
        return Err(TunnelError::HandshakeFailed(
            "no trusted peers are configured; refusing to listen".to_string(),
        ));
    }
    let identity: &NodeIdentity = &config.identity;
    let mut pending: HashMap<SocketAddr, PendingHandshake<'_>> = HashMap::new();
    let mut spare = Responder::new(identity);
    let mut limiter = RateLimiter::new(ACCEPT_RATE, ACCEPT_RATE * 2);
    let mut buf = vec![0u8; MAX_MESSAGE_LEN + 1];

    loop {
        let (len, from) = socket
            .recv_from(&mut buf)
            .await
            .map_err(|e| TunnelError::NetworkError(e.to_string()))?;
        if limiter.check_and_consume().is_err() {
            continue;
        }
        let datagram = &buf[..len];
        let now = Instant::now();
        pending.retain(|_, p| p.expires > now);

        match auth_handshake::message_type(datagram) {
            Some(MSG_INIT) => {
                if let Some(existing) = pending.get(&from) {
                    // Answer a retransmission; ignore a different INIT until
                    // the half-open handshake expires.
                    if existing.init == datagram {
                        if let Some(response) = existing.responder.response_message() {
                            let _ = socket.send_to(response, from).await;
                        }
                    }
                    continue;
                }
                match spare.handle_init(datagram, &config.trusted_peers) {
                    Ok(response) => {
                        if pending.len() >= MAX_PENDING_HANDSHAKES {
                            let oldest = pending
                                .iter()
                                .min_by_key(|(_, p)| p.expires)
                                .map(|(addr, _)| *addr);
                            if let Some(addr) = oldest {
                                pending.remove(&addr);
                            }
                        }
                        let responder = std::mem::replace(&mut spare, Responder::new(identity));
                        pending.insert(
                            from,
                            PendingHandshake {
                                responder,
                                init: datagram.to_vec(),
                                expires: now + config.handshake_timeout,
                            },
                        );
                        let _ = socket.send_to(&response, from).await;
                    }
                    Err(e) => {
                        log::debug!(
                            "event=handshake_rejected stage=init from={from} reason=\"{e}\""
                        );
                        if e.is_fatal() {
                            spare = Responder::new(identity);
                        }
                    }
                }
            }
            Some(MSG_FINISH) => {
                let Some(entry) = pending.get_mut(&from) else {
                    continue;
                };
                match entry.responder.handle_finish(datagram) {
                    Ok((confirm, keys)) => {
                        let finish = datagram.to_vec();
                        drop(pending);
                        socket
                            .send_to(&confirm, from)
                            .await
                            .map_err(|e| TunnelError::NetworkError(e.to_string()))?;
                        return Ok(into_tunnel(
                            socket,
                            keys,
                            from,
                            config,
                            Some((finish, confirm)),
                        ));
                    }
                    Err(e) => {
                        log::debug!(
                            "event=handshake_rejected stage=finish from={from} reason=\"{e}\""
                        );
                        if e.is_fatal() {
                            pending.remove(&from);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
