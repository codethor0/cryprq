// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! Authenticated VPN link over libp2p.
//!
//! libp2p is used as a transport only. Its own handshake (QUIC/TLS or Noise)
//! is not what authenticates a VPN peer: every connection runs the CrypRQ
//! handshake of [`node::auth_handshake`] between two pinned identities, and IP
//! packets travel only as records of the resulting [`node::Session`]. A packet
//! that is not such a record is dropped, and without an established session
//! nothing read from the local device is sent at all.
//!
//! The module has two layers:
//!
//! * [`Endpoint`] holds the handshake and session state and does no I/O. It
//!   turns the messages of a link into replies, sessions and packets, so its
//!   security properties can be tested with an attacker in the middle.
//! * [`run_secure_vpn`] drives an `Endpoint` over a libp2p swarm using a
//!   request-response protocol, and handles dialing, reconnection, key
//!   rotation and renewal of the session.
//!
//! Handshake messages map onto requests and responses: INIT is answered by
//! RESPONSE and FINISH by CONFIRM. A record is a request with an empty
//! response.
//!
//! # Renewal
//!
//! A renewal is a new handshake on a link that already has a session. Each
//! side installs the new session when it has verified the other side's
//! signature and key confirmation: the listener on FINISH, the dialer one
//! message later on CONFIRM. Neither side ever holds a session the other side
//! has not confirmed. Between the two installs, and for records in flight:
//!
//! * The replaced session is kept for [`PREVIOUS_SESSION_GRACE`], for
//!   receiving only, with its replay window. A record is accepted in exactly
//!   one of the two sessions and at most once.
//! * The dialer sends in the new session as soon as it has it; the listener
//!   already has it by then.
//! * The listener keeps sending in the replaced session until a record of the
//!   new session arrives from the dialer, which shows that the dialer has it,
//!   and for at most half the grace period. Its sequence numbers continue, so
//!   no nonce is reused.
//! * If the handshake fails at any message, the existing session stays as it
//!   was on the side that rejected it.

use anyhow::{anyhow, bail, Context, Result};
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::futures::StreamExt;
use libp2p::request_response::{self, OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{dial_opts::DialOpts, ConnectionId, DialError, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm, SwarmBuilder};
use node::auth_handshake::{self, HandshakeError, Initiator, Responder, MSG_FINISH, MSG_INIT};
use node::{
    NodeIdentity, PeerPublicKey, Session, TrustedPeers, TunnelConfig, TunnelError, TunnelRole,
};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::hash::Hash;
use std::io;
use std::task::Poll;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Protocol carrying handshake messages and records.
pub const VPN_PROTOCOL: StreamProtocol = StreamProtocol::new("/cryprq/vpn/2.0.0");
/// Largest IP packet accepted from the local device or from the peer.
pub const MAX_PACKET_LEN: usize = 65535;
/// Largest request: a record around a maximum-size packet. Handshake messages
/// are far smaller.
const MAX_REQUEST_LEN: usize = MAX_PACKET_LEN + 64;
/// Largest response: a handshake RESPONSE.
const MAX_RESPONSE_LEN: usize = auth_handshake::MAX_MESSAGE_LEN;
/// Half-open handshakes a listener keeps at once.
const MAX_PENDING_HANDSHAKES: usize = 16;
/// Records in flight before further packets from the device are dropped.
const MAX_IN_FLIGHT: usize = 1024;
/// How long records of a session that was just renewed are still accepted.
const PREVIOUS_SESSION_GRACE: Duration = Duration::from_secs(10);
/// First and longest delay between two dial attempts.
const REDIAL_MIN: Duration = Duration::from_secs(1);
const REDIAL_MAX: Duration = Duration::from_secs(30);

/// Why a message of a link was not accepted.
#[derive(Debug)]
pub enum LinkError {
    /// No session is established on this link.
    NoSession,
    /// The handshake rejected the message.
    Handshake(HandshakeError),
    /// The record layer rejected the message.
    Record(TunnelError),
    /// The transport peer is not the identity it claims in the handshake.
    IdentityNotBound,
    /// A message that is not valid in the current state.
    Unexpected(&'static str),
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSession => write!(f, "no session is established"),
            Self::Handshake(e) => write!(f, "handshake: {e}"),
            Self::Record(e) => write!(f, "record: {e}"),
            Self::IdentityNotBound => {
                write!(f, "transport peer is not the claimed identity")
            }
            Self::Unexpected(why) => write!(f, "unexpected message: {why}"),
        }
    }
}

impl std::error::Error for LinkError {}

/// A session that was just established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Established<L> {
    /// The authenticated identity of the peer.
    pub peer: PeerPublicKey,
    /// Public identifier shared by both ends.
    pub session_id: [u8; 16],
    /// Role this side played.
    pub role: TunnelRole,
    /// The link of the session this one replaced, if it was another link.
    pub replaced: Option<L>,
}

/// Result of an inbound request.
#[derive(Debug)]
pub(crate) enum Inbound<L> {
    /// Send this handshake message as the response.
    Reply(Vec<u8>),
    /// Send the response; the session is now established.
    Established(Vec<u8>, Established<L>),
    /// An authentic IP packet for the local device.
    Packet(Vec<u8>),
    /// The request was dropped.
    Rejected(LinkError),
}

/// Result of a response to a handshake request.
#[derive(Debug)]
pub(crate) enum Progress<L> {
    /// Send this handshake message as the next request.
    Send(Vec<u8>),
    /// The session is now established.
    Established(Established<L>),
}

struct Outbound<'a, L> {
    link: L,
    handshake: Initiator<'a>,
    awaiting_confirm: bool,
}

struct Active<L> {
    link: L,
    session: Session,
}

type BindingCheck<'a, L> = Box<dyn Fn(&L, &PeerPublicKey) -> bool + Send + 'a>;

/// Handshake and session state of one VPN endpoint. Does no I/O.
///
/// `L` identifies a transport link (a libp2p peer). The endpoint has at most
/// one active session: a VPN device is point to point, and a newly established
/// session replaces the previous one.
pub(crate) struct Endpoint<'a, L> {
    identity: &'a NodeIdentity,
    trusted: &'a TrustedPeers,
    /// Whether INIT messages are answered (listener) or refused (dialer).
    accept_inbound: bool,
    /// Whether a link belongs to an identity; see [`link_is_bound`].
    bound: BindingCheck<'a, L>,
    handshake_timeout: Duration,
    outbound: Option<Outbound<'a, L>>,
    pending: HashMap<L, (Responder<'a>, Instant)>,
    active: Option<Active<L>>,
    /// The session replaced by a renewal on the same link, kept briefly so
    /// records already in flight are not lost.
    previous: Option<(Active<L>, Instant)>,
    /// After a renewal the responder has the new session one message before
    /// the initiator. Until this time, or until the initiator has used the new
    /// session, the responder keeps sending in the previous one, which the
    /// initiator still accepts.
    send_on_previous_until: Option<Instant>,
}

impl<'a, L: Copy + Eq + Hash> Endpoint<'a, L> {
    pub(crate) fn new(
        identity: &'a NodeIdentity,
        trusted: &'a TrustedPeers,
        accept_inbound: bool,
        handshake_timeout: Duration,
        bound: BindingCheck<'a, L>,
    ) -> Self {
        Self {
            identity,
            trusted,
            accept_inbound,
            bound,
            handshake_timeout,
            outbound: None,
            pending: HashMap::new(),
            active: None,
            previous: None,
            send_on_previous_until: None,
        }
    }

    /// Starts a handshake with the pinned identity `expected` over `link` and
    /// returns the INIT message. Replaces a handshake already in progress.
    pub(crate) fn begin(
        &mut self,
        link: L,
        expected: &PeerPublicKey,
    ) -> Result<Vec<u8>, LinkError> {
        if !(self.bound)(&link, expected) {
            return Err(LinkError::IdentityNotBound);
        }
        let handshake =
            Initiator::new(self.identity, self.trusted, expected).map_err(LinkError::Handshake)?;
        let init = handshake.init_message().to_vec();
        self.outbound = Some(Outbound {
            link,
            handshake,
            awaiting_confirm: false,
        });
        Ok(init)
    }

    /// Whether a handshake started with [`Endpoint::begin`] is in progress.
    pub(crate) fn handshake_in_progress(&self) -> bool {
        self.outbound.is_some()
    }

    /// Processes the response to the last handshake request sent over `link`.
    /// Any error ends the handshake: the transport delivers a response once
    /// and in order, so there is nothing to wait for.
    pub(crate) fn on_response(
        &mut self,
        link: L,
        response: &[u8],
    ) -> Result<Progress<L>, LinkError> {
        let Some(mut outbound) = self.outbound.take() else {
            return Err(LinkError::Unexpected("no handshake in progress"));
        };
        if outbound.link != link {
            self.outbound = Some(outbound);
            return Err(LinkError::Unexpected("response from another link"));
        }
        if outbound.awaiting_confirm {
            let keys = outbound
                .handshake
                .handle_confirm(response)
                .map_err(LinkError::Handshake)?;
            return Ok(Progress::Established(self.install(link, keys)));
        }
        let finish = outbound
            .handshake
            .handle_response(response)
            .map_err(LinkError::Handshake)?;
        outbound.awaiting_confirm = true;
        self.outbound = Some(outbound);
        Ok(Progress::Send(finish))
    }

    /// Processes a request received over `link`.
    pub(crate) fn on_request(&mut self, link: L, request: &[u8], now: Instant) -> Inbound<L> {
        match auth_handshake::message_type(request) {
            Some(MSG_INIT) => self.on_init(link, request, now),
            Some(MSG_FINISH) => self.on_finish(link, request, now),
            Some(_) => Inbound::Rejected(LinkError::Unexpected(
                "handshake message that is not a request",
            )),
            None => match self.open(link, request, now) {
                Ok(packet) => Inbound::Packet(packet),
                Err(e) => Inbound::Rejected(e),
            },
        }
    }

    fn on_init(&mut self, link: L, request: &[u8], now: Instant) -> Inbound<L> {
        if !self.accept_inbound {
            return Inbound::Rejected(LinkError::Unexpected(
                "this endpoint does not accept handshakes",
            ));
        }
        let mut responder = Responder::new(self.identity);
        let response = match responder.handle_init(request, self.trusted) {
            Ok(response) => response,
            Err(e) => return Inbound::Rejected(LinkError::Handshake(e)),
        };
        // The claimed identity is pinned; it must also be the transport peer.
        match responder.claimed_peer() {
            Some(claimed) if (self.bound)(&link, &claimed) => {}
            _ => return Inbound::Rejected(LinkError::IdentityNotBound),
        }
        self.expire_pending(now);
        if self.pending.len() >= MAX_PENDING_HANDSHAKES && !self.pending.contains_key(&link) {
            let oldest = self
                .pending
                .iter()
                .min_by_key(|(_, (_, started))| *started)
                .map(|(link, _)| *link);
            if let Some(oldest) = oldest {
                self.pending.remove(&oldest);
            }
        }
        self.pending.insert(link, (responder, now));
        Inbound::Reply(response)
    }

    fn on_finish(&mut self, link: L, request: &[u8], now: Instant) -> Inbound<L> {
        // A FINISH is tried once. Only the transport peer itself can send on
        // its link, so there is no forged FINISH to tolerate.
        let Some((mut responder, started)) = self.pending.remove(&link) else {
            return Inbound::Rejected(LinkError::Unexpected("FINISH without INIT"));
        };
        if now.saturating_duration_since(started) > self.handshake_timeout {
            return Inbound::Rejected(LinkError::Unexpected("handshake expired"));
        }
        match responder.handle_finish(request) {
            Ok((confirm, keys)) => Inbound::Established(confirm, self.install(link, keys)),
            Err(e) => Inbound::Rejected(LinkError::Handshake(e)),
        }
    }

    fn install(&mut self, link: L, keys: auth_handshake::SessionKeys) -> Established<L> {
        let established = Established {
            peer: keys.peer,
            session_id: keys.session_id,
            role: keys.role.into(),
            replaced: None,
        };
        let session = Session::from_handshake(keys);
        let replaced = self.active.replace(Active { link, session });
        self.send_on_previous_until = None;
        match replaced {
            Some(old) if old.link == link => {
                let now = Instant::now();
                self.previous = Some((old, now + PREVIOUS_SESSION_GRACE));
                if established.role == TunnelRole::Responder {
                    // Well inside the time the initiator accepts the previous
                    // session, so nothing sent in it arrives too late.
                    self.send_on_previous_until = Some(now + PREVIOUS_SESSION_GRACE / 2);
                }
                established
            }
            Some(old) => {
                self.previous = None;
                Established {
                    replaced: Some(old.link),
                    ..established
                }
            }
            None => established,
        }
    }

    fn open(&mut self, link: L, record: &[u8], now: Instant) -> Result<Vec<u8>, LinkError> {
        self.retire_previous(now);
        let active = match &self.active {
            Some(active) if active.link == link => active,
            _ => return Err(LinkError::NoSession),
        };
        match active.session.open_vpn_packet(record) {
            Ok(packet) => {
                // The peer has the active session.
                self.send_on_previous_until = None;
                Ok(packet)
            }
            Err(TunnelError::DecryptionFailed) => {
                // Possibly a record sealed just before the session was renewed.
                // A failed open changes nothing in the active session.
                match &self.previous {
                    Some((previous, _)) if previous.link == link => previous
                        .session
                        .open_vpn_packet(record)
                        .map_err(LinkError::Record),
                    _ => Err(LinkError::Record(TunnelError::DecryptionFailed)),
                }
            }
            Err(e) => Err(LinkError::Record(e)),
        }
    }

    /// Encrypts a packet from the local device for the active session and
    /// returns the link to send it on. Without a session there is no output:
    /// a packet is never sent unprotected.
    pub(crate) fn seal(&self, packet: &[u8]) -> Result<(L, Vec<u8>), LinkError> {
        let active = self.active.as_ref().ok_or(LinkError::NoSession)?;
        if let (Some(until), Some((previous, _))) = (self.send_on_previous_until, &self.previous) {
            if Instant::now() < until && previous.link == active.link {
                // If the previous session can no longer send, use the new one.
                if let Ok(record) = previous.session.seal_vpn_packet(packet) {
                    return Ok((previous.link, record));
                }
            }
        }
        let record = active
            .session
            .seal_vpn_packet(packet)
            .map_err(LinkError::Record)?;
        Ok((active.link, record))
    }

    /// Rotates the sending keys of the active session.
    pub(crate) fn rotate(&self) -> Option<Result<u8, TunnelError>> {
        self.active
            .as_ref()
            .map(|active| active.session.rotate_send_keys())
    }

    /// The link of the active session.
    pub(crate) fn active_link(&self) -> Option<L> {
        self.active.as_ref().map(|active| active.link)
    }

    /// Identifier of the active session.
    #[cfg(test)]
    pub(crate) fn session_id(&self) -> Option<[u8; 16]> {
        self.active
            .as_ref()
            .map(|active| active.session.session_id())
    }

    /// Forgets everything that belongs to `link`; the keys of its session are
    /// wiped. Returns whether an established session was closed.
    pub(crate) fn close(&mut self, link: L) -> bool {
        self.pending.remove(&link);
        if matches!(&self.outbound, Some(outbound) if outbound.link == link) {
            self.outbound = None;
        }
        if matches!(&self.previous, Some((previous, _)) if previous.link == link) {
            self.previous = None;
            self.send_on_previous_until = None;
        }
        if matches!(&self.active, Some(active) if active.link == link) {
            self.active = None;
            return true;
        }
        false
    }

    /// Drops the session replaced by a renewal once its grace period is over,
    /// which wipes its keys. Called on a timer as well as on every record, so
    /// the keys do not outlive the grace period on an idle link.
    pub(crate) fn retire_previous(&mut self, now: Instant) {
        if matches!(&self.previous, Some((_, expiry)) if now >= *expiry) {
            self.previous = None;
            self.send_on_previous_until = None;
        }
    }

    /// Drops half-open inbound handshakes older than the handshake timeout.
    pub(crate) fn expire_pending(&mut self, now: Instant) {
        let timeout = self.handshake_timeout;
        self.pending
            .retain(|_, (_, started)| now.saturating_duration_since(*started) <= timeout);
    }
}

/// The libp2p keypair of a node: the same Ed25519 key as its CrypRQ identity,
/// so the transport peer id is stable and derived from the pinned key.
pub fn transport_keypair(identity: &NodeIdentity) -> Result<identity::Keypair> {
    let mut seed = identity.secret_seed();
    identity::Keypair::ed25519_from_bytes(&mut *seed)
        .map_err(|e| anyhow!("identity is not usable as a transport key: {e}"))
}

/// The libp2p peer id that belongs to a pinned identity.
pub fn peer_id_of(key: &PeerPublicKey) -> Result<PeerId> {
    let public = identity::ed25519::PublicKey::try_from_bytes(key.as_bytes())
        .map_err(|e| anyhow!("pinned key is not usable as a transport key: {e}"))?;
    Ok(identity::PublicKey::from(public).to_peer_id())
}

/// Whether the transport peer `link` is the identity `peer`.
///
/// This binds the two layers together as defence in depth. It is not what
/// authenticates the peer: that is the signature and key confirmation of the
/// CrypRQ handshake, which do not depend on the transport.
fn link_is_bound(link: &PeerId, peer: &PeerPublicKey) -> bool {
    peer_id_of(peer).map(|id| id == *link).unwrap_or(false)
}

/// Length-prefixed codec with small, fixed limits.
#[derive(Clone, Default)]
pub struct VpnCodec;

async fn read_frame<T: AsyncRead + Unpin + Send>(io: &mut T, max: usize) -> io::Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    io.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn write_frame<T: AsyncWrite + Unpin + Send>(
    io: &mut T,
    frame: &[u8],
    max: usize,
) -> io::Result<()> {
    if frame.len() > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame too large",
        ));
    }
    io.write_all(&(frame.len() as u32).to_be_bytes()).await?;
    io.write_all(frame).await?;
    io.flush().await
}

#[async_trait::async_trait]
impl request_response::Codec for VpnCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_frame(io, MAX_REQUEST_LEN).await
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_frame(io, MAX_RESPONSE_LEN).await
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        request: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_frame(io, &request, MAX_REQUEST_LEN).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        response: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_frame(io, &response, MAX_RESPONSE_LEN).await
    }
}

type VpnBehaviour = request_response::Behaviour<VpnCodec>;

/// How this endpoint finds its peer.
#[derive(Debug, Clone)]
pub enum VpnMode {
    /// Wait on this address for a pinned peer.
    Listen(Multiaddr),
    /// Dial the pinned identity `peer` at `addr`, and dial again whenever the
    /// connection is lost.
    Dial {
        addr: Multiaddr,
        peer: PeerPublicKey,
    },
}

/// Something that happened on the link; also written to the log.
#[derive(Debug, Clone)]
pub enum VpnEvent {
    /// The listener is ready on this address.
    Listening(Multiaddr),
    /// A session with an authenticated peer is established.
    SessionEstablished {
        peer: PeerPublicKey,
        session_id: [u8; 16],
        role: TunnelRole,
    },
    /// The session ended.
    SessionClosed { reason: String },
    /// The sending keys moved to this epoch.
    KeysRotated(u8),
    /// A connection, handshake message or record was refused.
    Rejected(String),
}

/// Configuration of [`run_secure_vpn`].
pub struct VpnSettings {
    /// Identity, pinned peers and handshake timeout.
    pub tunnel: TunnelConfig,
    /// Interval between rotations of the sending keys within a session.
    pub rotate_interval: Duration,
    /// Age at which the dialer replaces the session by a new handshake, which
    /// renews the key agreement itself.
    pub rehandshake_interval: Duration,
    /// Receives a copy of every [`VpnEvent`].
    pub events: Option<mpsc::UnboundedSender<VpnEvent>>,
}

impl VpnSettings {
    /// Defaults: keys rotate every 5 minutes, the session is renewed hourly.
    pub fn new(tunnel: TunnelConfig) -> Self {
        Self {
            tunnel,
            rotate_interval: Duration::from_secs(300),
            rehandshake_interval: Duration::from_secs(3600),
            events: None,
        }
    }
}

/// The local side of the link: whole IP packets to and from a device.
pub struct PacketIo {
    /// Packets read from the local device, to be sent to the peer.
    pub from_device: mpsc::Receiver<Vec<u8>>,
    /// Authentic packets from the peer, to be written to the local device.
    pub to_device: mpsc::Sender<Vec<u8>>,
}

fn build_swarm(identity: &NodeIdentity) -> Result<Swarm<VpnBehaviour>> {
    let swarm = SwarmBuilder::with_existing_identity(transport_keypair(identity)?)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| anyhow!("failed to set up the TCP transport: {e}"))?
        .with_quic()
        .with_behaviour(|_| {
            request_response::Behaviour::with_codec(
                VpnCodec,
                [(VPN_PROTOCOL, ProtocolSupport::Full)],
                request_response::Config::default()
                    .with_request_timeout(Duration::from_secs(10))
                    .with_max_concurrent_streams(256),
            )
        })
        .map_err(|e| anyhow!("failed to set up the VPN protocol: {e}"))?
        // The link stays up while it is idle; a dead peer is noticed by the
        // transport's own keep-alive.
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(u32::MAX as u64)))
        .build();
    Ok(swarm)
}

struct DialTarget {
    addr: Multiaddr,
    peer_id: PeerId,
    peer: PeerPublicKey,
}

struct Engine<'a> {
    swarm: Swarm<VpnBehaviour>,
    endpoint: Endpoint<'a, PeerId>,
    /// Transport peer ids of the pinned identities.
    allowed: HashMap<PeerId, PeerPublicKey>,
    /// The connection in use with each pinned peer: the newest one.
    connections: HashMap<PeerId, ConnectionId>,
    target: Option<DialTarget>,
    to_device: mpsc::Sender<Vec<u8>>,
    events: Option<mpsc::UnboundedSender<VpnEvent>>,
    handshake_timeout: Duration,
    rehandshake_interval: Duration,
    /// The handshake request that is waiting for its response.
    handshake_request: Option<OutboundRequestId>,
    handshake_deadline: Option<Instant>,
    session_started: Option<Instant>,
    in_flight: usize,
    redial_at: Option<Instant>,
    redial_delay: Duration,
    dropped_without_session: u64,
    /// Set when the link cannot continue, for example when the listening
    /// socket is lost.
    fatal: Option<anyhow::Error>,
}

impl Engine<'_> {
    fn emit(&self, event: VpnEvent) {
        match &event {
            VpnEvent::Listening(addr) => log::info!("event=vpn_listening addr={addr}"),
            VpnEvent::SessionEstablished {
                peer,
                session_id,
                role,
            } => log::info!(
                "event=vpn_session_established role={:?} peer={} session_id={}",
                role,
                peer,
                hex(session_id)
            ),
            VpnEvent::SessionClosed { reason } => {
                log::info!("event=vpn_session_closed reason=\"{reason}\"")
            }
            VpnEvent::KeysRotated(epoch) => log::info!("event=vpn_keys_rotated epoch={epoch}"),
            VpnEvent::Rejected(reason) => log::warn!("event=vpn_rejected reason=\"{reason}\""),
        }
        if let Some(events) = &self.events {
            let _ = events.send(event);
        }
    }

    fn established(&mut self, established: Established<PeerId>) {
        self.handshake_request = None;
        self.handshake_deadline = None;
        self.session_started = Some(Instant::now());
        self.redial_delay = REDIAL_MIN;
        if let Some(old_link) = established.replaced {
            let _ = self.swarm.disconnect_peer_id(old_link);
        }
        self.emit(VpnEvent::SessionEstablished {
            peer: established.peer,
            session_id: established.session_id,
            role: established.role,
        });
    }

    fn dial(&mut self) {
        let Some(target) = &self.target else { return };
        self.redial_at = None;
        let opts = DialOpts::peer_id(target.peer_id)
            .addresses(vec![target.addr.clone()])
            .build();
        if let Err(e) = self.swarm.dial(opts) {
            log::debug!("event=vpn_dial_failed reason=\"{e}\"");
            self.schedule_redial();
        }
    }

    fn schedule_redial(&mut self) {
        if self.target.is_some() && self.redial_at.is_none() {
            self.redial_at = Some(Instant::now() + self.redial_delay);
            self.redial_delay = (self.redial_delay * 2).min(REDIAL_MAX);
        }
    }

    /// Starts a handshake with the dial target over its connection.
    fn start_handshake(&mut self) {
        let Some(target) = &self.target else { return };
        let (peer_id, peer) = (target.peer_id, target.peer);
        match self.endpoint.begin(peer_id, &peer) {
            Ok(init) => {
                let request = self.swarm.behaviour_mut().send_request(&peer_id, init);
                self.handshake_request = Some(request);
                self.handshake_deadline = Some(Instant::now() + self.handshake_timeout);
            }
            Err(e) => {
                self.emit(VpnEvent::Rejected(format!(
                    "cannot start the handshake: {e}"
                )));
                let _ = self.swarm.disconnect_peer_id(peer_id);
            }
        }
    }

    /// The session can no longer send (its key epochs are used up) or is due
    /// for renewal. The dialer runs a new handshake on the same connection;
    /// a listener closes the connection, upon which the dialer comes back.
    fn renew_session(&mut self) {
        if self.target.is_some() {
            if !self.endpoint.handshake_in_progress() {
                log::info!("event=vpn_session_renewal");
                self.start_handshake();
            }
        } else if let Some(link) = self.endpoint.active_link() {
            let _ = self.swarm.disconnect_peer_id(link);
        }
    }

    fn on_connection_established(&mut self, peer_id: PeerId, connection: ConnectionId) {
        if !self.allowed.contains_key(&peer_id) {
            self.emit(VpnEvent::Rejected(format!(
                "transport peer {peer_id} is not a pinned identity"
            )));
            let _ = self.swarm.disconnect_peer_id(peer_id);
            return;
        }
        log::info!("event=vpn_transport_connected transport_peer={peer_id}");
        // A dialer opens a connection only when it has none, and by then it
        // has dropped its session. An older connection to that peer which
        // this side has not yet seen fail is therefore dead, and a session
        // still held for the peer is one the peer no longer has. Sending on
        // that connection or in that session, or treating the handshake that
        // follows as a renewal of it, would lose packets. Only the pinned
        // peer itself can open this connection.
        if let Some(stale) = self.connections.insert(peer_id, connection) {
            self.swarm.close_connection(stale);
        }
        if self.endpoint.close(peer_id) {
            self.session_started = None;
            self.emit(VpnEvent::SessionClosed {
                reason: "the peer reconnected".to_string(),
            });
        }
        let is_target = matches!(&self.target, Some(target) if target.peer_id == peer_id);
        if is_target && !self.endpoint.handshake_in_progress() {
            self.start_handshake();
        }
    }

    fn on_connection_closed(&mut self, peer_id: PeerId, connection: ConnectionId) {
        // Only the connection in use carries the session. A stale one that
        // was replaced, or one that was refused, changes nothing.
        if self.connections.get(&peer_id) != Some(&connection) {
            return;
        }
        self.connections.remove(&peer_id);
        if matches!(&self.target, Some(target) if target.peer_id == peer_id) {
            self.handshake_request = None;
            self.handshake_deadline = None;
            self.schedule_redial();
        }
        if self.endpoint.close(peer_id) {
            self.session_started = None;
            self.emit(VpnEvent::SessionClosed {
                reason: "connection closed".to_string(),
            });
        }
    }

    fn on_request(
        &mut self,
        peer: PeerId,
        request: Vec<u8>,
        channel: request_response::ResponseChannel<Vec<u8>>,
    ) {
        if !self.allowed.contains_key(&peer) {
            return;
        }
        let behaviour = self.swarm.behaviour_mut();
        match self.endpoint.on_request(peer, &request, Instant::now()) {
            Inbound::Reply(reply) => {
                let _ = behaviour.send_response(channel, reply);
            }
            Inbound::Established(reply, established) => {
                let _ = behaviour.send_response(channel, reply);
                self.established(established);
            }
            Inbound::Packet(packet) => {
                let _ = behaviour.send_response(channel, Vec::new());
                if packet.len() <= MAX_PACKET_LEN {
                    // A full device queue drops the packet, like a full link.
                    let _ = self.to_device.try_send(packet);
                }
            }
            Inbound::Rejected(error) => {
                let _ = behaviour.send_response(channel, Vec::new());
                let handshake_failure =
                    matches!(error, LinkError::Handshake(_) | LinkError::IdentityNotBound);
                self.emit(VpnEvent::Rejected(format!("request from {peer}: {error}")));
                if handshake_failure {
                    let _ = self.swarm.disconnect_peer_id(peer);
                }
            }
        }
    }

    fn on_response(&mut self, peer: PeerId, request_id: OutboundRequestId, response: Vec<u8>) {
        if self.handshake_request != Some(request_id) {
            self.in_flight = self.in_flight.saturating_sub(1);
            return;
        }
        self.handshake_request = None;
        match self.endpoint.on_response(peer, &response) {
            Ok(Progress::Send(finish)) => {
                let request = self.swarm.behaviour_mut().send_request(&peer, finish);
                self.handshake_request = Some(request);
            }
            Ok(Progress::Established(established)) => self.established(established),
            Err(error) => {
                self.emit(VpnEvent::Rejected(format!(
                    "handshake with {peer} failed: {error}"
                )));
                let _ = self.swarm.disconnect_peer_id(peer);
            }
        }
    }

    fn on_outbound_failure(&mut self, peer: PeerId, request_id: OutboundRequestId) {
        if self.handshake_request == Some(request_id) {
            self.handshake_request = None;
            self.emit(VpnEvent::Rejected(format!(
                "handshake with {peer} failed: request was not answered"
            )));
            let _ = self.swarm.disconnect_peer_id(peer);
        } else {
            self.in_flight = self.in_flight.saturating_sub(1);
        }
    }

    fn on_swarm_event(&mut self, event: SwarmEvent<request_response::Event<Vec<u8>, Vec<u8>>>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => self.emit(VpnEvent::Listening(address)),
            SwarmEvent::ListenerClosed { reason, .. } => {
                // A listener that no longer listens must not keep running as
                // if it did.
                self.fatal = Some(match reason {
                    Ok(()) => anyhow!("the listener was closed"),
                    Err(e) => anyhow!("the listener failed: {e}"),
                });
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                ..
            } => self.on_connection_established(peer_id, connection_id),
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                ..
            } => self.on_connection_closed(peer_id, connection_id),
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                if let DialError::WrongPeerId { obtained, .. } = &error {
                    // Something answered at the address, but with another
                    // transport identity than the pinned one.
                    self.emit(VpnEvent::Rejected(format!(
                        "the node at the dialed address is {obtained}, not the pinned peer"
                    )));
                } else {
                    log::debug!("event=vpn_dial_failed reason=\"{error}\"");
                }
                self.schedule_redial();
            }
            SwarmEvent::Behaviour(request_response::Event::Message { peer, message, .. }) => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => self.on_request(peer, request, channel),
                    request_response::Message::Response {
                        request_id,
                        response,
                    } => self.on_response(peer, request_id, response),
                }
            }
            SwarmEvent::Behaviour(request_response::Event::OutboundFailure {
                peer,
                request_id,
                ..
            }) => self.on_outbound_failure(peer, request_id),
            _ => {}
        }
    }

    /// Sends one packet from the local device, or drops it.
    fn on_device_packet(&mut self, packet: Vec<u8>) {
        if packet.is_empty() || packet.len() > MAX_PACKET_LEN || self.in_flight >= MAX_IN_FLIGHT {
            return;
        }
        match self.endpoint.seal(&packet) {
            Ok((link, record)) => {
                self.swarm.behaviour_mut().send_request(&link, record);
                self.in_flight += 1;
            }
            Err(LinkError::NoSession) => {
                // Fail closed: without a session the packet goes nowhere.
                self.dropped_without_session += 1;
                if self.dropped_without_session.is_power_of_two() {
                    log::warn!(
                        "event=vpn_packet_dropped reason=\"no session\" total={}",
                        self.dropped_without_session
                    );
                }
            }
            Err(LinkError::Record(TunnelError::SessionExpired | TunnelError::NonceOverflow)) => {
                self.renew_session()
            }
            Err(error) => log::debug!("event=vpn_packet_dropped reason=\"{error}\""),
        }
    }

    fn rotate_keys(&mut self) {
        match self.endpoint.rotate() {
            Some(Ok(epoch)) => self.emit(VpnEvent::KeysRotated(epoch)),
            Some(Err(_)) => self.renew_session(),
            None => {}
        }
    }

    fn housekeeping(&mut self) {
        let now = Instant::now();
        self.endpoint.expire_pending(now);
        self.endpoint.retire_previous(now);
        if matches!(self.redial_at, Some(at) if now >= at) {
            self.dial();
        }
        if matches!(self.handshake_deadline, Some(deadline) if now >= deadline) {
            self.handshake_deadline = None;
            self.handshake_request = None;
            if let Some(target) = &self.target {
                let peer_id = target.peer_id;
                self.emit(VpnEvent::Rejected("handshake timed out".to_string()));
                let _ = self.swarm.disconnect_peer_id(peer_id);
            }
        }
        let due = matches!(self.session_started, Some(started)
            if now.saturating_duration_since(started) >= self.rehandshake_interval);
        if due && self.target.is_some() && !self.endpoint.handshake_in_progress() {
            // Push the next attempt out so a slow renewal is not restarted.
            self.session_started = Some(now);
            self.renew_session();
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// Runs an authenticated VPN link until `shutdown` completes. If the local
/// device closes its channel, the link is shut down and an error is returned.
///
/// Packets from `io.from_device` are sent only inside an established session
/// with a pinned, authenticated peer, and only authentic packets of that
/// session are delivered to `io.to_device`. When the session or the connection
/// is lost, packets are dropped until a new session exists; the dialer
/// reconnects on its own.
pub async fn run_secure_vpn<F>(
    settings: VpnSettings,
    mode: VpnMode,
    mut io: PacketIo,
    shutdown: F,
) -> Result<()>
where
    F: Future<Output = ()>,
{
    let identity: &NodeIdentity = &settings.tunnel.identity;
    let trusted: &TrustedPeers = &settings.tunnel.trusted_peers;
    if trusted.is_empty() {
        bail!("no trusted peers are configured; refusing to start the VPN link");
    }
    let mut allowed = HashMap::new();
    for key in trusted.iter() {
        allowed.insert(peer_id_of(key)?, *key);
    }

    let mut swarm = build_swarm(identity)?;
    let target = match &mode {
        VpnMode::Listen(addr) => {
            swarm
                .listen_on(addr.clone())
                .with_context(|| format!("cannot listen on {addr}"))?;
            None
        }
        VpnMode::Dial { addr, peer } => {
            if !trusted.contains(peer) {
                bail!("the peer to dial is not a pinned identity");
            }
            if *peer == identity.public_key() {
                bail!("the peer to dial is this node's own identity");
            }
            Some(DialTarget {
                addr: addr.clone(),
                peer_id: peer_id_of(peer)?,
                peer: *peer,
            })
        }
    };

    let handshake_timeout = settings.tunnel.handshake_timeout;
    let mut engine = Engine {
        swarm,
        endpoint: Endpoint::new(
            identity,
            trusted,
            target.is_none(),
            handshake_timeout,
            Box::new(link_is_bound),
        ),
        allowed,
        connections: HashMap::new(),
        target,
        to_device: io.to_device,
        events: settings.events,
        handshake_timeout,
        rehandshake_interval: settings.rehandshake_interval,
        handshake_request: None,
        handshake_deadline: None,
        session_started: None,
        in_flight: 0,
        redial_at: None,
        redial_delay: REDIAL_MIN,
        dropped_without_session: 0,
        fatal: None,
    };
    engine.dial();

    let mut rotation = tokio::time::interval(settings.rotate_interval);
    rotation.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    rotation.tick().await; // the first tick is immediate
    let mut housekeeping = tokio::time::interval(Duration::from_millis(250));
    housekeeping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    tokio::pin!(shutdown);
    let mut device_lost = false;

    // What woke the loop. The sources are polled by hand instead of with
    // `tokio::select!`, whose expansion contains `panic!`.
    enum Wake {
        Shutdown,
        Swarm(SwarmEvent<request_response::Event<Vec<u8>, Vec<u8>>>),
        Device(Option<Vec<u8>>),
        Rotate,
        Housekeeping,
    }
    let mut turn = 0usize;

    loop {
        let wake = {
            let swarm_event = engine.swarm.select_next_some();
            let device = io.from_device.recv();
            let rotate = rotation.tick();
            let keep = housekeeping.tick();
            tokio::pin!(swarm_event, device, rotate, keep);
            // Shutdown is always looked at first. The other sources take
            // turns at being first, so a busy one cannot starve the rest.
            // The first source that is ready ends the wait, so none is
            // polled again after it has completed.
            let first = turn;
            std::future::poll_fn(|cx| {
                if shutdown.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(Wake::Shutdown);
                }
                for offset in 0..4 {
                    let ready = match (first + offset) % 4 {
                        0 => swarm_event.as_mut().poll(cx).map(Wake::Swarm),
                        1 => device.as_mut().poll(cx).map(Wake::Device),
                        2 => rotate.as_mut().poll(cx).map(|_| Wake::Rotate),
                        _ => keep.as_mut().poll(cx).map(|_| Wake::Housekeeping),
                    };
                    if ready.is_ready() {
                        return ready;
                    }
                }
                Poll::Pending
            })
            .await
        };
        turn = turn.wrapping_add(1);
        match wake {
            Wake::Shutdown => break,
            Wake::Swarm(event) => engine.on_swarm_event(event),
            Wake::Device(Some(packet)) => engine.on_device_packet(packet),
            Wake::Device(None) => {
                // Without a device there is nothing to protect. Stop, and
                // report it, instead of holding a session with no use.
                device_lost = true;
                break;
            }
            Wake::Rotate => engine.rotate_keys(),
            Wake::Housekeeping => engine.housekeeping(),
        }
        if let Some(error) = engine.fatal.take() {
            return Err(error);
        }
    }

    // Close the connections so the peer drops its session promptly.
    engine.target = None;
    let connected: Vec<PeerId> = engine.swarm.connected_peers().copied().collect();
    if !connected.is_empty() {
        for peer in connected {
            let _ = engine.swarm.disconnect_peer_id(peer);
        }
        let flush_until = tokio::time::Instant::now() + Duration::from_millis(500);
        while let Ok(event) =
            tokio::time::timeout_at(flush_until, engine.swarm.select_next_some()).await
        {
            engine.on_swarm_event(event);
            if engine.swarm.connected_peers().next().is_none() {
                break;
            }
        }
    }
    if device_lost {
        log::error!("event=vpn_stopped reason=\"the local device is gone\"");
        bail!("the local packet device closed; the VPN link was shut down");
    }
    log::info!("event=vpn_stopped");
    Ok(())
}

#[cfg(test)]
#[path = "secure_vpn_tests.rs"]
mod tests;
