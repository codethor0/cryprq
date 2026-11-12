// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use bytes::BytesMut;
use crossbeam::queue::ArrayQueue;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time;
use zeroize::{Zeroize, Zeroizing};

pub mod auth_handshake;
mod crypto_utils;
mod dns;
mod error;
mod establish;
mod file_transfer;
mod handshake;
pub mod identity;
mod padding;
mod record_layer;
mod tls;
mod traffic_shaping;
pub mod tun;

pub use crypto_utils::{make_nonce, Epoch};
pub use establish::{accept_tunnel, connect_tunnel, TunnelConfig};
pub use file_transfer::{FileMetadata, FileTransferManager};
pub use handshake::HandshakeRole;
pub use identity::{IdentityError, NodeIdentity, PeerPublicKey, TrustedPeers};
pub use record_layer::{alloc_stream_id, recv_record, send_record, DirectionKeys, VPN_STREAM_ID};

pub use cryprq_core::{RecordHeader, MSG_TYPE_VPN_PACKET};

pub use dns::{resolve_hostname, DnsConfig, DnsError};
pub use error::TunnelError;
pub use padding::{pad_packet, unpad_packet, PaddingConfig};
pub use tls::{TlsClient, TlsConfig, TlsError, TlsServer, TlsStream};
pub use traffic_shaping::TrafficShaper;
pub use tun::{TunConfig, TunInterface};

const MAX_NONCE_VALUE: u64 = u64::MAX - 1000; // Force rekey before overflow
const REPLAY_WINDOW_SIZE: usize = 2048; // Track last 2048 nonces
const BUFFER_SIZE: usize = 65535; // UDP max packet size
const POOL_SIZE: usize = 32; // Number of buffers to pool
/// Interval between automatic rotations of the sending traffic keys.
const KEY_ROTATION_INTERVAL: Duration = Duration::from_secs(300);
/// How long the previous receive epoch stays usable for reordered records.
const PREVIOUS_EPOCH_GRACE: Duration = Duration::from_secs(10);
/// How long a responder answers a retransmitted FINISH with its CONFIRM.
const CONFIRM_RESEND_WINDOW: Duration = Duration::from_secs(30);

/// Buffer pool for packet receive operations
///
/// Reuses 65KB buffers to prevent excessive allocations.
/// Uses lock-free queue for high-performance multi-threaded access.
pub(crate) struct BufferPool {
    pool: Arc<ArrayQueue<BytesMut>>,
}

impl BufferPool {
    fn new(capacity: usize) -> Self {
        let pool = Arc::new(ArrayQueue::new(capacity));

        // Pre-allocate buffers
        for _ in 0..capacity {
            let buf = BytesMut::with_capacity(BUFFER_SIZE);
            let _ = pool.push(buf);
        }

        Self { pool }
    }

    /// Get a buffer from the pool, or allocate new if pool is empty
    fn get(&self) -> BytesMut {
        self.pool
            .pop()
            .unwrap_or_else(|| BytesMut::with_capacity(BUFFER_SIZE))
    }

    /// Return a buffer to the pool
    fn put(&self, mut buf: BytesMut) {
        buf.clear();
        // Only return to pool if under capacity (prevents unbounded growth)
        let _ = self.pool.push(buf);
    }
}

impl Clone for BufferPool {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}

/// Token bucket rate limiter
///
/// Implements a token bucket algorithm to limit incoming packet rate.
/// Prevents DoS attacks by rejecting excessive traffic.
pub(crate) struct RateLimiter {
    /// Maximum tokens (burst capacity)
    capacity: u32,
    /// Current token count
    tokens: f64,
    /// Tokens added per second (steady rate)
    refill_rate: f64,
    /// Last refill timestamp
    last_refill: Instant,
}

impl RateLimiter {
    /// Create new rate limiter
    ///
    /// # Arguments
    /// * `packets_per_second` - Sustained rate limit
    /// * `burst_size` - Maximum burst capacity
    fn new(packets_per_second: u32, burst_size: u32) -> Self {
        Self {
            capacity: burst_size,
            tokens: burst_size as f64,
            refill_rate: packets_per_second as f64,
            last_refill: Instant::now(),
        }
    }

    /// Check if packet should be accepted
    /// Returns Ok(()) if within limit, Err if rate exceeded
    fn check_and_consume(&mut self) -> Result<(), TunnelError> {
        // Calculate tokens to add based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Refill tokens based on elapsed time
        self.tokens += elapsed * self.refill_rate;
        self.tokens = self.tokens.min(self.capacity as f64);
        self.last_refill = now;

        // Check if we have tokens available
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            Err(TunnelError::RateLimitExceeded)
        }
    }
}

/// Anti-replay window using sliding bitmap
///
/// Tracks recently seen nonces to detect and reject replay attacks.
/// Uses a bitmap for memory efficiency (256 bytes for 2048-nonce window).
pub(crate) struct ReplayWindow {
    /// Highest nonce value seen so far
    max_nonce: u64,
    /// Bitmap tracking which nonces in the window have been seen
    /// Each bit represents one nonce: 1 = seen, 0 = not seen
    bitmap: [u64; REPLAY_WINDOW_SIZE / 64],
}

impl ReplayWindow {
    fn new() -> Self {
        Self {
            max_nonce: 0,
            bitmap: [0u64; REPLAY_WINDOW_SIZE / 64],
        }
    }

    /// Check if nonce is valid and mark it as seen
    /// Returns Ok(()) if nonce is fresh, Err if replay detected
    fn check_and_update(&mut self, nonce: u64) -> Result<(), TunnelError> {
        // Reject nonces that are too old (outside window)
        if nonce.saturating_add(REPLAY_WINDOW_SIZE as u64) < self.max_nonce {
            return Err(TunnelError::ReplayDetected);
        }

        // Check if nonce is within current window
        if nonce <= self.max_nonce {
            let diff = (self.max_nonce - nonce) as usize;
            if diff >= REPLAY_WINDOW_SIZE {
                return Err(TunnelError::ReplayDetected);
            }

            // Check if this nonce was already seen
            let index = diff / 64;
            let bit = diff % 64;
            if (self.bitmap[index] & (1u64 << bit)) != 0 {
                return Err(TunnelError::ReplayDetected);
            }

            // Mark nonce as seen
            self.bitmap[index] |= 1u64 << bit;
        } else {
            // New maximum nonce - slide the window forward
            let diff = (nonce - self.max_nonce) as usize;

            if diff < REPLAY_WINDOW_SIZE {
                // Shift bitmap by 'diff' positions
                let words_to_shift = diff / 64;
                let bits_to_shift = diff % 64;

                // Shift whole words
                if words_to_shift > 0 {
                    for i in (words_to_shift..self.bitmap.len()).rev() {
                        self.bitmap[i] = self.bitmap[i - words_to_shift];
                    }
                    for i in 0..words_to_shift {
                        self.bitmap[i] = 0;
                    }
                }

                // Shift remaining bits
                if bits_to_shift > 0 {
                    let mut carry = 0u64;
                    for word in self.bitmap.iter_mut() {
                        let new_carry = *word >> (64 - bits_to_shift);
                        *word = (*word << bits_to_shift) | carry;
                        carry = new_carry;
                    }
                }
            } else {
                // Gap is larger than window - clear bitmap
                self.bitmap = [0u64; REPLAY_WINDOW_SIZE / 64];
            }

            // Mark new nonce as seen (bit 0 of word 0)
            self.bitmap[0] |= 1u64;
            self.max_nonce = nonce;
        }

        Ok(())
    }
}

/// Name of the environment variable that enables the insecure fixed-key mode.
pub const TEST_MODE_ENV: &str = "CRYPRQ_TEST_MODE";

/// Whether a value of [`TEST_MODE_ENV`] asks for test mode. Only an explicit
/// affirmative does; `false`, `0`, an empty value and anything else do not.
fn test_mode_value_enables(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some("1") | Some("true") | Some("TRUE") | Some("True")
    )
}

/// Whether the insecure fixed-key test mode was explicitly requested through
/// the environment.
pub fn test_mode_enabled() -> bool {
    test_mode_value_enables(std::env::var(TEST_MODE_ENV).ok().as_deref())
}

/// The publicly known placeholder credentials used by `--test-mode`.
fn is_fixed_test_credentials(
    peer_pk: &[u8; 32],
    peer_identity_key: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    peer_pk.iter().all(|&b| b == peer_pk[0])
        && peer_identity_key.iter().all(|&b| b == peer_identity_key[0])
        && signature.iter().all(|&b| b == signature[0])
        && matches!(peer_pk[0], 0x01..=0x04)
}

/// Tunnel role (initiator or responder)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelRole {
    Initiator,
    Responder,
}

impl From<HandshakeRole> for TunnelRole {
    fn from(role: HandshakeRole) -> Self {
        match role {
            HandshakeRole::Initiator => TunnelRole::Initiator,
            HandshakeRole::Responder => TunnelRole::Responder,
        }
    }
}

/// Derives the (send, receive) traffic keys of `role` for `epoch`.
fn epoch_keys(
    master_secret: &[u8; 32],
    epoch: u8,
    role: TunnelRole,
) -> (DirectionKeys, DirectionKeys) {
    let (mut key_ir, mut iv_ir, mut key_ri, mut iv_ri) =
        cryprq_crypto::derive_epoch_keys(master_secret, epoch, 32, 12);
    let build = |key: &[u8], iv: &[u8]| {
        let mut keys = DirectionKeys {
            key: [0u8; 32],
            iv: [0u8; 12],
        };
        keys.key.copy_from_slice(key);
        keys.iv.copy_from_slice(iv);
        keys
    };
    let initiator_to_responder = build(&key_ir, &iv_ir);
    let responder_to_initiator = build(&key_ri, &iv_ri);
    key_ir.zeroize();
    iv_ir.zeroize();
    key_ri.zeroize();
    iv_ri.zeroize();
    match role {
        TunnelRole::Initiator => (initiator_to_responder, responder_to_initiator),
        TunnelRole::Responder => (responder_to_initiator, initiator_to_responder),
    }
}

/// Sending half of a session: one key and one sequence space per epoch.
struct SendState {
    epoch: u8,
    keys: DirectionKeys,
    /// Next sequence number. Shared by every message type, because the AEAD
    /// nonce is derived from the sequence number alone.
    next_seq: u64,
    /// Set when the last epoch has been used up; the session must be replaced.
    expired: bool,
}

/// Receiving keys and replay state of one epoch.
struct RecvEpoch {
    epoch: u8,
    keys: DirectionKeys,
    window: ReplayWindow,
}

struct RecvState {
    current: RecvEpoch,
    /// The epoch before `current`, kept briefly for reordered records.
    previous: Option<(RecvEpoch, Instant)>,
}

/// Cryptographic state of one established session.
///
/// Each direction has its own keys, epoch and sequence space. The sender of a
/// direction advances its epoch; the receiver follows when a record of a later
/// epoch authenticates. Epochs never wrap: after epoch 255 the session expires,
/// so a (key, nonce) pair is never used twice.
///
/// A session does no I/O. [`Tunnel`] carries its records over UDP; other
/// transports obtain one with [`Session::from_handshake`] after driving the
/// [`auth_handshake`] state machines themselves.
pub struct Session {
    role: TunnelRole,
    peer: Option<PeerPublicKey>,
    session_id: [u8; 16],
    master_secret: Zeroizing<[u8; 32]>,
    send: Mutex<SendState>,
    recv: Mutex<RecvState>,
}

impl Session {
    /// The session established by a completed authenticated handshake.
    pub fn from_handshake(keys: auth_handshake::SessionKeys) -> Self {
        Self::new(
            keys.role.into(),
            keys.master_secret,
            Some(keys.peer),
            keys.session_id,
        )
    }

    /// Role this side played in the handshake.
    pub fn role(&self) -> TunnelRole {
        self.role
    }

    /// The authenticated identity of the peer; `None` only for the insecure
    /// fixed-key test mode.
    pub fn peer_identity(&self) -> Option<PeerPublicKey> {
        self.peer
    }

    /// Public identifier shared by both ends of this session.
    pub fn session_id(&self) -> [u8; 16] {
        self.session_id
    }

    /// Encrypts one tunnelled IP packet.
    pub fn seal_vpn_packet(&self, packet: &[u8]) -> Result<Vec<u8>, TunnelError> {
        self.seal(VPN_STREAM_ID, MSG_TYPE_VPN_PACKET, 0, packet)
    }

    /// Authenticates and decrypts a record that must carry a tunnelled IP
    /// packet. Any other authentic record is refused.
    pub fn open_vpn_packet(&self, record: &[u8]) -> Result<Vec<u8>, TunnelError> {
        let (header, packet) = self.open(record)?;
        if header.message_type != MSG_TYPE_VPN_PACKET || header.stream_id != VPN_STREAM_ID {
            return Err(TunnelError::UnexpectedRecord);
        }
        Ok(packet)
    }

    pub(crate) fn new(
        role: TunnelRole,
        master_secret: Zeroizing<[u8; 32]>,
        peer: Option<PeerPublicKey>,
        session_id: [u8; 16],
    ) -> Self {
        let (send_keys, recv_keys) = epoch_keys(&master_secret, 0, role);
        Self {
            role,
            peer,
            session_id,
            master_secret,
            send: Mutex::new(SendState {
                epoch: 0,
                keys: send_keys,
                next_seq: 0,
                expired: false,
            }),
            recv: Mutex::new(RecvState {
                current: RecvEpoch {
                    epoch: 0,
                    keys: recv_keys,
                    window: ReplayWindow::new(),
                },
                previous: None,
            }),
        }
    }

    /// Encrypts one record under the current sending epoch.
    pub fn seal(
        &self,
        stream_id: u32,
        message_type: u8,
        flags: u8,
        payload: &[u8],
    ) -> Result<Vec<u8>, TunnelError> {
        let mut send = self
            .send
            .lock()
            .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
        if send.expired {
            return Err(TunnelError::SessionExpired);
        }
        if send.next_seq >= MAX_NONCE_VALUE {
            return Err(TunnelError::NonceOverflow);
        }
        let seq = send.next_seq;
        let record = send_record(
            Epoch(send.epoch),
            stream_id,
            seq,
            message_type,
            flags,
            payload,
            &send.keys,
        )
        .map_err(|_| TunnelError::EncryptionFailed)?;
        send.next_seq = seq + 1;
        Ok(record)
    }

    /// Authenticates and decrypts one record, then enforces replay protection.
    ///
    /// Nothing changes unless the record authenticates: a forged or corrupted
    /// datagram cannot advance the epoch or the replay window.
    pub fn open(&self, datagram: &[u8]) -> Result<(RecordHeader, Vec<u8>), TunnelError> {
        let header =
            RecordHeader::from_bytes(datagram).map_err(|_| TunnelError::DecryptionFailed)?;
        let mut recv = self
            .recv
            .lock()
            .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
        if matches!(&recv.previous, Some((_, expiry)) if Instant::now() >= *expiry) {
            recv.previous = None;
        }

        if header.epoch == recv.current.epoch {
            let (header, plaintext) = recv_record(datagram, &recv.current.keys)
                .map_err(|_| TunnelError::DecryptionFailed)?;
            recv.current
                .window
                .check_and_update(header.sequence_number)?;
            return Ok((header, plaintext));
        }
        if let Some((previous, _)) = recv.previous.as_mut() {
            if header.epoch == previous.epoch {
                let (header, plaintext) = recv_record(datagram, &previous.keys)
                    .map_err(|_| TunnelError::DecryptionFailed)?;
                previous.window.check_and_update(header.sequence_number)?;
                return Ok((header, plaintext));
            }
        }
        if header.epoch < recv.current.epoch {
            // An epoch that was already left behind is never accepted again.
            return Err(TunnelError::DecryptionFailed);
        }

        // The peer moved to a later epoch. Follow only if the record proves it.
        let (_, candidate) = epoch_keys(&self.master_secret, header.epoch, self.role);
        let (header, plaintext) =
            recv_record(datagram, &candidate).map_err(|_| TunnelError::DecryptionFailed)?;
        let mut next = RecvEpoch {
            epoch: header.epoch,
            keys: candidate,
            window: ReplayWindow::new(),
        };
        next.window.check_and_update(header.sequence_number)?;
        let superseded = std::mem::replace(&mut recv.current, next);
        recv.previous = Some((superseded, Instant::now() + PREVIOUS_EPOCH_GRACE));
        Ok((header, plaintext))
    }

    /// Moves the sending direction to the next epoch with fresh keys and a
    /// fresh sequence space. Fails with [`TunnelError::SessionExpired`] once
    /// all epochs are used; the session then refuses to send.
    pub fn rotate_send_keys(&self) -> Result<u8, TunnelError> {
        let mut send = self
            .send
            .lock()
            .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
        if send.expired || send.epoch == u8::MAX {
            send.expired = true;
            return Err(TunnelError::SessionExpired);
        }
        let epoch = send.epoch + 1;
        let (keys, _) = epoch_keys(&self.master_secret, epoch, self.role);
        send.keys = keys;
        send.epoch = epoch;
        send.next_seq = 0;
        Ok(epoch)
    }
}

/// CONFIRM kept by a responder so that a retransmitted FINISH can be answered
/// when the first CONFIRM was lost.
struct ConfirmEcho {
    finish: Vec<u8>,
    confirm: Vec<u8>,
    until: Instant,
}

/// Authenticated encrypted tunnel
///
/// Provides secure bidirectional communication with:
/// - ChaCha20-Poly1305 AEAD records with the header as associated data
/// - Independent keys, epochs and sequence numbers per direction
/// - Replay protection (sliding window of 2048 sequence numbers per epoch)
/// - Rate limiting (1000 pps sustained, 2000 burst)
/// - Buffer pooling for memory efficiency
///
/// A tunnel is obtained from [`connect_tunnel`] or [`accept_tunnel`], which
/// complete the mutually authenticated hybrid handshake first. No tunnel value
/// exists before that, so application data cannot be exchanged with an
/// unauthenticated peer.
pub struct Tunnel {
    socket: Arc<UdpSocket>,
    session: Arc<Session>,
    peer_addr: Arc<RwLock<Option<std::net::SocketAddr>>>,
    rate_limiter: Mutex<RateLimiter>,
    buffer_pool: BufferPool,
    tun_write_tx: Arc<RwLock<Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>>>, // Channel to write VPN packets to TUN
    file_transfer: Arc<FileTransferManager>, // File transfer manager
    confirm_echo: Option<ConfirmEcho>,
    rotation_task: tokio::task::JoinHandle<()>,
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        // The task only holds a weak reference, so the session keys are wiped
        // as soon as the tunnel goes away; aborting just stops the timer.
        self.rotation_task.abort();
    }
}

impl Tunnel {
    /// Wraps an established session. Must be called within a Tokio runtime.
    pub(crate) fn from_session(
        socket: UdpSocket,
        session: Session,
        peer_addr: Option<std::net::SocketAddr>,
        file_output_dir: Option<std::path::PathBuf>,
        confirm_echo: Option<(Vec<u8>, Vec<u8>)>,
    ) -> Self {
        let session = Arc::new(session);
        let weak = Arc::downgrade(&session);
        let rotation_task = tokio::spawn(async move {
            let mut interval = time::interval(KEY_ROTATION_INTERVAL);
            interval.tick().await; // Skip first immediate tick
            loop {
                interval.tick().await;
                let Some(session) = weak.upgrade() else { break };
                match session.rotate_send_keys() {
                    Ok(epoch) => log::info!(
                        "event=key_rotation status=success epoch={} interval_secs={}",
                        epoch,
                        KEY_ROTATION_INTERVAL.as_secs()
                    ),
                    Err(e) => {
                        log::warn!("event=key_rotation status=stopped reason=\"{e}\"");
                        break;
                    }
                }
            }
        });
        Self {
            socket: Arc::new(socket),
            session,
            peer_addr: Arc::new(RwLock::new(peer_addr)),
            rate_limiter: Mutex::new(RateLimiter::new(1000, 2000)), // 1000 pps, 2000 burst
            buffer_pool: BufferPool::new(POOL_SIZE),
            tun_write_tx: Arc::new(RwLock::new(None)), // Will be set when TUN forwarding starts
            file_transfer: Arc::new(FileTransferManager::new(
                file_output_dir.unwrap_or_else(|| std::path::PathBuf::from("/tmp")),
            )),
            confirm_echo: confirm_echo.map(|(finish, confirm)| ConfirmEcho {
                finish,
                confirm,
                until: Instant::now() + CONFIRM_RESEND_WINDOW,
            }),
            rotation_task,
        }
    }

    /// Role this side played in the handshake.
    pub fn role(&self) -> TunnelRole {
        self.session.role
    }

    /// The authenticated identity of the peer. `None` only for the insecure
    /// fixed-key test mode, which authenticates nobody.
    pub fn peer_identity(&self) -> Option<PeerPublicKey> {
        self.session.peer
    }

    /// Public identifier shared by both ends of this session.
    pub fn session_id(&self) -> [u8; 16] {
        self.session.session_id
    }

    /// Local address of the tunnel's UDP socket.
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.socket.local_addr()
    }

    /// Rotates the sending traffic keys now and returns the new epoch. The
    /// peer follows on the first record it receives under the new epoch.
    pub fn rotate_keys(&self) -> Result<u8, TunnelError> {
        self.session.rotate_send_keys()
    }

    /// Set TUN write channel for VPN packet forwarding
    pub fn set_tun_writer(&self, tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>) {
        if let Ok(mut guard) = self.tun_write_tx.write() {
            *guard = Some(tx);
        }
    }

    /// Get peer address (for setting peer address)
    pub fn peer_addr(&self) -> &Arc<RwLock<Option<std::net::SocketAddr>>> {
        &self.peer_addr
    }

    /// Get file transfer manager
    pub fn file_transfer(&self) -> &Arc<FileTransferManager> {
        &self.file_transfer
    }

    /// Send a CrypRQ record to peer
    ///
    /// Wraps payload in a CrypRQ record with proper header, encryption, and sequence numbering.
    pub async fn send_record(
        &self,
        stream_id: u32,
        message_type: u8,
        flags: u8,
        payload: &[u8],
    ) -> Result<(), TunnelError> {
        let record_bytes = self.session.seal(stream_id, message_type, flags, payload)?;

        // Send over UDP
        let peer_addr = *self
            .peer_addr
            .read()
            .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;

        if let Some(addr) = peer_addr {
            self.socket
                .send_to(&record_bytes, addr)
                .await
                .map_err(|e| TunnelError::NetworkError(e.to_string()))?;
        }

        Ok(())
    }

    /// Receive and decrypt a CrypRQ record from peer
    ///
    /// Returns (message_type, stream_id, payload). Datagrams are checked in
    /// this order: rate limit, AEAD authentication, replay window. The peer
    /// address is updated only by a record that passed all three.
    pub async fn recv_record(&self) -> Result<(u8, u32, Vec<u8>), TunnelError> {
        loop {
            // Get buffer from pool
            let mut buf = self.buffer_pool.get();
            buf.resize(BUFFER_SIZE, 0);

            let received = self.socket.recv_from(&mut buf).await;
            let (len, addr) = match received {
                Ok(pair) => pair,
                Err(e) => {
                    self.buffer_pool.put(buf);
                    return Err(TunnelError::NetworkError(e.to_string()));
                }
            };

            let limited = self
                .rate_limiter
                .lock()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))
                .and_then(|mut limiter| limiter.check_and_consume());
            if let Err(e) = limited {
                self.buffer_pool.put(buf);
                return Err(e);
            }

            // Handshake messages are never application data.
            if auth_handshake::message_type(&buf[..len]).is_some() {
                let reply = self.confirm_for_retransmitted_finish(&buf[..len]);
                self.buffer_pool.put(buf);
                if let Some(confirm) = reply {
                    let _ = self.socket.send_to(&confirm, addr).await;
                }
                continue;
            }

            let opened = self.session.open(&buf[..len]);
            self.buffer_pool.put(buf);
            let (header, payload) = match opened {
                Ok(record) => record,
                Err(e) => {
                    log::debug!("cryp-rq: dropped {len} bytes from {addr}: {e}");
                    return Err(e);
                }
            };

            *self
                .peer_addr
                .write()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))? = Some(addr);
            return Ok((header.message_type, header.stream_id, payload));
        }
    }

    /// The stored CONFIRM, if `datagram` is exactly the FINISH that completed
    /// this session and the resend window is still open.
    fn confirm_for_retransmitted_finish(&self, datagram: &[u8]) -> Option<Vec<u8>> {
        let echo = self.confirm_echo.as_ref()?;
        (Instant::now() < echo.until && echo.finish == datagram).then(|| echo.confirm.clone())
    }

    /// Send VPN packet through record layer
    ///
    /// Wraps TUN packet in a CrypRQ VPN_PACKET record and sends it.
    pub async fn send_vpn_packet(&self, packet: &[u8]) -> Result<(), TunnelError> {
        self.send_record(VPN_STREAM_ID, cryprq_core::MSG_TYPE_VPN_PACKET, 0, packet)
            .await
    }

    /// Send file metadata through record layer
    pub async fn send_file_meta(
        &self,
        stream_id: u32,
        meta: &file_transfer::FileMetadata,
    ) -> Result<(), TunnelError> {
        let payload = meta.serialize();
        self.send_record(stream_id, cryprq_core::MSG_TYPE_FILE_META, 0, &payload)
            .await
    }

    /// Send file chunk through record layer
    pub async fn send_file_chunk(&self, stream_id: u32, chunk: &[u8]) -> Result<(), TunnelError> {
        self.send_record(stream_id, cryprq_core::MSG_TYPE_FILE_CHUNK, 0, chunk)
            .await
    }

    /// Send file acknowledgment through record layer
    pub async fn send_file_ack(&self, stream_id: u32, ack_data: &[u8]) -> Result<(), TunnelError> {
        self.send_record(stream_id, cryprq_core::MSG_TYPE_FILE_ACK, 0, ack_data)
            .await
    }

    /// Handle incoming CrypRQ record
    ///
    /// Routes records by message type to appropriate handlers.
    pub async fn handle_incoming_record(
        &self,
        msg_type: u8,
        stream_id: u32,
        payload: Vec<u8>,
    ) -> Result<(), TunnelError> {
        use cryprq_core::{
            MSG_TYPE_CONTROL, MSG_TYPE_DATA, MSG_TYPE_FILE_ACK, MSG_TYPE_FILE_CHUNK,
            MSG_TYPE_FILE_META, MSG_TYPE_VPN_PACKET,
        };

        match msg_type {
            MSG_TYPE_VPN_PACKET => {
                // Write VPN packet to TUN interface via channel
                match self.tun_write_tx.read() {
                    Ok(guard) => {
                        if let Some(ref tx) = *guard {
                            if let Err(e) = tx.send(payload.clone()) {
                                log::error!("Failed to send VPN packet to TUN: {e}");
                            } else {
                                log::debug!(
                                    "Forwarded VPN packet: {} bytes on stream {}",
                                    payload.len(),
                                    stream_id
                                );
                            }
                        } else {
                            log::debug!(
                                "Received VPN packet but no TUN writer: {} bytes on stream {}",
                                payload.len(),
                                stream_id
                            );
                        }
                    }
                    Err(_) => {
                        log::warn!("Failed to acquire TUN write lock");
                    }
                }
                Ok(())
            }
            MSG_TYPE_FILE_META | MSG_TYPE_FILE_CHUNK | MSG_TYPE_FILE_ACK | MSG_TYPE_CONTROL => {
                // Route to file transfer handler
                self.handle_file_or_control(stream_id, msg_type, payload)
                    .await
            }
            MSG_TYPE_DATA => {
                // Generic data stream
                log::debug!(
                    "Received data message: stream={}, {} bytes",
                    stream_id,
                    payload.len()
                );
                Ok(())
            }
            _ => {
                log::warn!("Unknown message type: {msg_type}");
                Ok(())
            }
        }
    }

    /// Send an encrypted VPN packet to the peer through the record layer.
    ///
    /// Returns `TunnelError` if the sequence space of the current epoch is
    /// exhausted, the session expired, encryption fails, or on network errors.
    pub async fn send_packet(&self, pkt: &[u8]) -> Result<(), TunnelError> {
        // Use record layer for VPN packets
        self.send_record(VPN_STREAM_ID, cryprq_core::MSG_TYPE_VPN_PACKET, 0, pkt)
            .await
    }

    /// Handle file transfer or control message
    async fn handle_file_or_control(
        &self,
        stream_id: u32,
        msg_type: u8,
        payload: Vec<u8>,
    ) -> Result<(), TunnelError> {
        use cryprq_core::{
            MSG_TYPE_CONTROL, MSG_TYPE_FILE_ACK, MSG_TYPE_FILE_CHUNK, MSG_TYPE_FILE_META,
        };

        match msg_type {
            MSG_TYPE_FILE_META => {
                let meta = file_transfer::FileMetadata::deserialize(&payload).map_err(|e| {
                    TunnelError::HandshakeFailed(format!("Failed to decode file metadata: {e}"))
                })?;
                log::debug!(
                    "cryp-rq: FILE_META received: stream_id={} name={} size={} hash={}",
                    stream_id,
                    meta.filename,
                    meta.size,
                    hex::encode(&meta.hash[..8])
                );
                self.file_transfer
                    .start_incoming_transfer(stream_id, meta)
                    .map_err(|e| {
                        TunnelError::HandshakeFailed(format!("Failed to start transfer: {e}"))
                    })?;
            }
            MSG_TYPE_FILE_CHUNK => {
                log::debug!(
                    "cryp-rq: FILE_CHUNK received: stream_id={} len={}",
                    stream_id,
                    payload.len()
                );
                self.file_transfer
                    .write_chunk(stream_id, &payload)
                    .map_err(|e| {
                        TunnelError::HandshakeFailed(format!("Failed to write chunk: {e}"))
                    })?;
            }
            MSG_TYPE_FILE_ACK => {
                self.file_transfer
                    .handle_ack(stream_id, &payload)
                    .map_err(|e| {
                        TunnelError::HandshakeFailed(format!("Failed to handle ACK: {e}"))
                    })?;
            }
            MSG_TYPE_CONTROL => {
                self.file_transfer
                    .handle_control(stream_id, &payload)
                    .map_err(|e| {
                        TunnelError::HandshakeFailed(format!("Failed to handle control: {e}"))
                    })?;
            }
            _ => {
                log::warn!("Unknown file/control message type: {msg_type}");
            }
        }
        Ok(())
    }

    /// Receive and process incoming record
    ///
    /// Receives a record, decrypts it, and routes it to the appropriate handler.
    pub async fn recv_and_handle_record(&self) -> Result<(), TunnelError> {
        let (msg_type, stream_id, payload) = self.recv_record().await?;
        self.handle_incoming_record(msg_type, stream_id, payload)
            .await
    }

    /// Receive and decrypt packet from peer (legacy method - returns payload only)
    pub async fn recv_packet(&self) -> Result<Vec<u8>, TunnelError> {
        let (_msg_type, _stream_id, plaintext) = self.recv_record().await?;
        Ok(plaintext)
    }
}

/// Creates a tunnel with the insecure fixed test keys.
///
/// This entry point predates the authenticated handshake and never exchanged
/// key material with the peer. It now serves a single purpose: local testing
/// with the publicly known `--test-mode` placeholders, and only when
/// [`TEST_MODE_ENV`] is explicitly set to `1` or `true`. Every other call is
/// refused; real sessions are established with [`connect_tunnel`] and
/// [`accept_tunnel`].
///
/// # Security
///
/// A tunnel returned by this function provides NO confidentiality or
/// authentication: its keys are derived from constants in this source file.
pub async fn create_tunnel(
    local_sk: &[u8; 32],
    peer_pk: &[u8; 32],
    peer_identity_key: &[u8; 32],
    peer_signature: &[u8; 64],
    listen_addr: &str,
) -> Result<Tunnel, TunnelError> {
    create_tunnel_with_output_dir(
        local_sk,
        peer_pk,
        peer_identity_key,
        peer_signature,
        listen_addr,
        None,
    )
    .await
}

/// Fixed-key test tunnel with a file output directory; see [`create_tunnel`].
pub async fn create_tunnel_with_output_dir(
    _local_sk: &[u8; 32],
    peer_pk: &[u8; 32],
    peer_identity_key: &[u8; 32],
    peer_signature: &[u8; 64],
    listen_addr: &str,
    file_output_dir: Option<std::path::PathBuf>,
) -> Result<Tunnel, TunnelError> {
    if !is_fixed_test_credentials(peer_pk, peer_identity_key, peer_signature) {
        return Err(TunnelError::HandshakeFailed(
            "this entry point performs no key exchange; use connect_tunnel or accept_tunnel \
             with a pinned peer identity"
                .to_string(),
        ));
    }
    create_test_mode_tunnel(TunnelRole::Initiator, listen_addr, file_output_dir).await
}

/// Creates a fixed-key test tunnel for the given role.
///
/// Two test-mode peers can only exchange records when one of them is the
/// initiator and the other the responder, because each direction has its own
/// keys. Like [`create_tunnel`], this is refused unless [`TEST_MODE_ENV`] is
/// explicitly set to `1` or `true`.
///
/// # Security
///
/// The keys are derived from constants in this source file. The tunnel
/// provides NO confidentiality and authenticates nobody.
pub async fn create_test_mode_tunnel(
    role: TunnelRole,
    listen_addr: &str,
    file_output_dir: Option<std::path::PathBuf>,
) -> Result<Tunnel, TunnelError> {
    if !test_mode_enabled() {
        log::error!("Rejecting fixed test keys: {TEST_MODE_ENV} is not set to 1 or true");
        return Err(TunnelError::InvalidPeerIdentity);
    }
    log::warn!(
        "TEST MODE: tunnel uses publicly known fixed keys and authenticates nobody. \
         This is INSECURE and must only be used for local testing."
    );

    let socket = UdpSocket::bind(listen_addr).await?;
    let (_, master_secret) = cryprq_crypto::derive_handshake_keys(&[0u8; 32], &[0xAA; 32]);
    let session = Session::new(role, Zeroizing::new(master_secret), None, [0u8; 16]);
    Ok(Tunnel::from_session(
        socket,
        session,
        None,
        file_output_dir,
        None,
    ))
}

// Implement PacketForwarder for Tunnel to work with TUN interface
#[async_trait::async_trait]
impl tun::PacketForwarder for Tunnel {
    async fn send_packet(&self, packet: &[u8]) -> anyhow::Result<()> {
        self.send_vpn_packet(packet)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send VPN packet: {e}"))
    }

    async fn recv_packet(&mut self) -> anyhow::Result<Vec<u8>> {
        // This is called by TUN write loop - receive records and extract VPN packets
        // The TUN write loop should actually use a separate receive loop that calls recv_and_handle_record
        // For now, this returns any packet payload (legacy behavior)
        self.recv_packet()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to receive packet: {e}"))
    }
}

#[cfg(test)]
mod tests;
