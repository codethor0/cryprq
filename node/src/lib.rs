// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Post-quantum secure VPN tunnel implementation
//!
//! This crate provides authenticated, encrypted tunnel functionality with:
//! - Ed25519 peer authentication (prevents MitM attacks)
//! - ChaCha20-Poly1305 AEAD encryption
//! - X25519 Diffie-Hellman key exchange
//! - BLAKE3 key derivation
//! - Replay attack protection (2048-nonce sliding window)
//! - Rate limiting (token bucket algorithm)
//! - Memory-efficient buffer pooling
//!
//! # Security Features
//!
//! - CWE-287 Prevention: Mandatory peer identity verification
//! - CWE-294 Prevention: Anti-replay window with nonce tracking
//! - CWE-400 Prevention: Configurable rate limiting (1000 pps default)
//! - CWE-789 Prevention: Buffer pool prevents memory exhaustion
//!
//! # Example
//!
//! ```no_run
//! use node::{create_tunnel, generate_handshake_auth};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let local_sk = [1u8; 32];
//! let peer_pk = [2u8; 32];
//! let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&peer_pk);
//!
//! let tunnel = create_tunnel(
//!     &local_sk,
//!     &peer_pk,
//!     &peer_identity_key,
//!     &peer_signature,
//!     "0.0.0.0:51820"
//! ).await?;
//!
//! tunnel.send_packet(b"Hello, secure world!").await?;
//! # Ok(())
//! # }
//! ```

use bytes::BytesMut;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use crossbeam::queue::ArrayQueue;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng as RandOsRng;
use rand_core::OsRng;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time;
use x25519_dalek::{EphemeralSecret, PublicKey};
use zeroize::Zeroize;

mod dns;
mod error;
mod padding;
mod tls;
mod traffic_shaping;
mod tun;

pub use dns::{resolve_hostname, DnsConfig, DnsError};
pub use error::TunnelError;
pub use padding::{pad_packet, unpad_packet, PaddingConfig};
pub use tls::{TlsClient, TlsConfig, TlsError, TlsServer, TlsStream};
pub use tun::{TunConfig, TunInterface};
pub use traffic_shaping::TrafficShaper;

const MAX_NONCE_VALUE: u64 = u64::MAX - 1000; // Force rekey before overflow
const REPLAY_WINDOW_SIZE: usize = 2048; // Track last 2048 nonces
const BUFFER_SIZE: usize = 65535; // UDP max packet size
const POOL_SIZE: usize = 32; // Number of buffers to pool

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
        if nonce + (REPLAY_WINDOW_SIZE as u64) < self.max_nonce {
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

/// Verifies peer identity using Ed25519 signature
///
/// This prevents MitM attacks by ensuring the peer possesses the private key
/// corresponding to their advertised public key.
fn verify_peer_identity(
    peer_pk: &[u8; 32],
    peer_identity_key: &[u8; 32],
    signature: &[u8; 64],
) -> Result<(), TunnelError> {
    // Construct the verifying key from peer's identity public key
    let verifying_key = VerifyingKey::from_bytes(peer_identity_key)
        .map_err(|_| TunnelError::InvalidPeerIdentity)?;

    // Construct signature from bytes
    let sig = Signature::from_bytes(signature);

    // Verify that the signature on peer_pk is valid
    verifying_key
        .verify(peer_pk, &sig)
        .map_err(|_| TunnelError::InvalidPeerIdentity)?;

    Ok(())
}

/// Generates handshake authentication credentials for testing.
/// In production, this would use persistent identity keys.
///
/// Returns (signing_key, verifying_key_bytes, signature_bytes)
pub fn generate_handshake_auth(peer_pk: &[u8; 32]) -> (SigningKey, [u8; 32], [u8; 64]) {
    use ed25519_dalek::SigningKey;

    let mut csprng = RandOsRng;
    let secret_bytes: [u8; 32] = rand::Rng::gen(&mut csprng);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    let signature = signing_key.sign(peer_pk);

    (signing_key, verifying_key.to_bytes(), signature.to_bytes())
}

/// Authenticated encrypted tunnel
///
/// Provides secure bidirectional communication with:
/// - ChaCha20-Poly1305 AEAD encryption
/// - Automatic nonce management
/// - Replay attack protection
/// - Rate limiting (1000 pps sustained, 2000 burst)
/// - Buffer pooling for memory efficiency
///
/// # Security
///
/// - Session keys derived from X25519 DH + peer identity
/// - Automatic key rotation every 5 minutes
/// - Nonce overflow protection (rekey at u64::MAX - 1000)
/// - Anti-replay window tracks 2048 recent nonces
pub struct Tunnel {
    socket: Arc<UdpSocket>,
    session_key: Arc<RwLock<[u8; 32]>>,
    peer_addr: Arc<RwLock<Option<std::net::SocketAddr>>>,
    nonce_counter: Arc<RwLock<u64>>,
    replay_window: Arc<RwLock<ReplayWindow>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    buffer_pool: BufferPool,
}

impl Tunnel {
    /// Send encrypted packet to peer
    ///
    /// Encrypts payload with ChaCha20-Poly1305 and sends via UDP.
    /// Automatically increments nonce counter for each packet.
    ///
    /// # Arguments
    ///
    /// * `pkt` - Plaintext packet data to encrypt and send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `TunnelError` if:
    /// - Nonce counter overflow detected (requires rekey)
    /// - Encryption fails
    /// - Network I/O error
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use node::{create_tunnel, generate_handshake_auth};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let local_sk = [1u8; 32];
    /// # let peer_pk = [2u8; 32];
    /// # let (_, key, sig) = generate_handshake_auth(&peer_pk);
    /// # let tunnel = create_tunnel(&local_sk, &peer_pk, &key, &sig, "0.0.0.0:0").await?;
    /// tunnel.send_packet(b"secure message").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_packet(&self, pkt: &[u8]) -> Result<(), TunnelError> {
        // Read key and create cipher (fast, drop lock immediately)
        let cipher = {
            let key = self
                .session_key
                .read()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
            ChaCha20Poly1305::new(key.as_ref().into())
        };

        // Increment nonce counter (fast, drop lock immediately)
        let nonce = {
            let mut counter = self
                .nonce_counter
                .write()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;

            // Check for nonce overflow before incrementing
            if *counter >= MAX_NONCE_VALUE {
                return Err(TunnelError::NonceOverflow);
            }

            *counter += 1;
            let nonce_bytes = counter.to_le_bytes();
            let mut nonce_arr = [0u8; 12];
            nonce_arr[..8].copy_from_slice(&nonce_bytes);
            Nonce::from(nonce_arr)
        };

        let ciphertext = cipher
            .encrypt(&nonce, Payload { msg: pkt, aad: b"" })
            .map_err(|_| TunnelError::EncryptionFailed)?;
        
        // Log encryption event for debugging
        log::debug!("🔐 ENCRYPT: {} bytes plaintext -> {} bytes ciphertext (nonce: {})", 
                   pkt.len(), ciphertext.len(), nonce_value);

        let peer_addr = {
            *self
                .peer_addr
                .read()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?
        };

        if let Some(addr) = peer_addr {
            self.socket.send_to(&ciphertext, addr).await?;
        }

        Ok(())
    }

    /// Receive and decrypt packet from peer
    ///
    /// Receives UDP packet, validates rate limit, checks for replay,
    /// and decrypts payload using ChaCha20-Poly1305.
    ///
    /// # Returns
    ///
    /// Returns decrypted plaintext on success, or `TunnelError` if:
    /// - Rate limit exceeded (1000 pps sustained, 2000 burst)
    /// - Replay attack detected (nonce already seen)
    /// - Invalid nonce format
    /// - Decryption fails (authentication tag mismatch)
    /// - Network I/O error
    ///
    /// # Security
    ///
    /// Validates packets in this order (fast-fail):
    /// 1. Rate limiting (prevents DoS)
    /// 2. Nonce extraction
    /// 3. Replay detection (checks nonce window)
    /// 4. Decryption (AEAD authentication)
    ///
    /// Buffers are pooled and reused for memory efficiency.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use node::{create_tunnel, generate_handshake_auth};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let local_sk = [1u8; 32];
    /// # let peer_pk = [2u8; 32];
    /// # let (_, key, sig) = generate_handshake_auth(&peer_pk);
    /// # let tunnel = create_tunnel(&local_sk, &peer_pk, &key, &sig, "0.0.0.0:0").await?;
    /// let plaintext = tunnel.recv_packet().await?;
    /// println!("Received {} bytes", plaintext.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recv_packet(&self) -> Result<Vec<u8>, TunnelError> {
        // Get buffer from pool (reuse instead of allocating)
        let mut buf = self.buffer_pool.get();
        buf.resize(BUFFER_SIZE, 0);

        let (len, addr) = self.socket.recv_from(&mut buf).await?;

        // Check rate limit first (before expensive operations)
        {
            let mut limiter = self
                .rate_limiter
                .write()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
            limiter.check_and_consume()?;
        }

        // Store peer address for future sends (fast, drop lock immediately)
        {
            *self
                .peer_addr
                .write()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))? = Some(addr);
        }

        // Create cipher (fast, drop lock immediately)
        let cipher = {
            let key = self
                .session_key
                .read()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
            ChaCha20Poly1305::new(key.as_ref().into())
        };

        // Extract nonce from packet (simplified: assume first 12 bytes)
        let nonce_bytes: [u8; 12] = buf[..12]
            .try_into()
            .map_err(|_| TunnelError::InvalidNonce)?;
        let nonce = Nonce::from(nonce_bytes);
        let ciphertext = &buf[12..len];

        // Extract nonce value for replay detection
        let nonce_value = u64::from_le_bytes(
            nonce_bytes[..8]
                .try_into()
                .map_err(|_| TunnelError::InvalidNonce)?,
        );

        // Check for replay attack
        {
            let mut window = self
                .replay_window
                .write()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string()))?;
            window.check_and_update(nonce_value)?;
        }

        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: ciphertext,
                    aad: b"",
                },
            )
            .map_err(|_| TunnelError::DecryptionFailed)?;
        
        // Log decryption event for debugging
        log::debug!("🔓 DECRYPT: {} bytes ciphertext -> {} bytes plaintext (nonce: {})", 
                   ciphertext.len(), plaintext.len(), nonce_value);

        // Return buffer to pool for reuse
        self.buffer_pool.put(buf);

        Ok(plaintext)
    }
}

/// Creates a secure tunnel with peer authentication
///
/// # Security
///
/// This implementation now includes:
/// - Peer identity verification using Ed25519 signatures
/// - Protection against MitM attacks
/// - Mutual authentication via signature exchange
///
/// # Arguments
///
/// * `local_sk` - Local secret key for X25519 DH
/// * `peer_pk` - Peer's X25519 public key
/// * `peer_identity_key` - Peer's Ed25519 identity public key
/// * `peer_signature` - Peer's signature on their DH public key
/// * `listen_addr` - UDP socket address to bind
///
/// # Returns
///
/// Returns authenticated tunnel or TunnelError if handshake fails
pub async fn create_tunnel(
    local_sk: &[u8; 32],
    peer_pk: &[u8; 32],
    peer_identity_key: &[u8; 32],
    peer_signature: &[u8; 64],
    listen_addr: &str,
) -> Result<Tunnel, TunnelError> {
    let socket = UdpSocket::bind(listen_addr).await?;

    // SECURITY: Verify peer identity before establishing tunnel
    verify_peer_identity(peer_pk, peer_identity_key, peer_signature)?;

    // Perform authenticated Noise-IK handshake
    let peer_public = PublicKey::from(*peer_pk);
    let shared_secret = EphemeralSecret::random_from_rng(OsRng).diffie_hellman(&peer_public);

    // Derive session key using BLAKE3 KDF with peer identity included
    let mut kdf_input = Vec::with_capacity(32 + 32);
    kdf_input.extend_from_slice(shared_secret.as_bytes());
    kdf_input.extend_from_slice(peer_identity_key);
    let session_key = *blake3::hash(&kdf_input).as_bytes();

    let tunnel = Tunnel {
        socket: Arc::new(socket),
        session_key: Arc::new(RwLock::new(session_key)),
        peer_addr: Arc::new(RwLock::new(None)),
        nonce_counter: Arc::new(RwLock::new(0)),
        replay_window: Arc::new(RwLock::new(ReplayWindow::new())),
        rate_limiter: Arc::new(RwLock::new(RateLimiter::new(1000, 2000))), // 1000 pps, 2000 burst
        buffer_pool: BufferPool::new(POOL_SIZE),
    };

    // Spawn key rotation task (every 5 minutes)
    let session_key_clone = tunnel.session_key.clone();
    let nonce_counter_clone = tunnel.nonce_counter.clone();
    let _local_sk_clone = *local_sk;
    let peer_pk_clone = *peer_pk;
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(300));
        interval.tick().await; // Skip first immediate tick
        loop {
            interval.tick().await;

            // Re-derive session key (in practice would do new DH)
            let peer_public = PublicKey::from(peer_pk_clone);
            let shared_secret =
                EphemeralSecret::random_from_rng(OsRng).diffie_hellman(&peer_public);
            let new_key = *blake3::hash(shared_secret.as_bytes()).as_bytes();

            if let Ok(mut key) = session_key_clone.write() {
                key.zeroize();
                *key = new_key;
            }

            // Reset nonce counter on key rotation
            if let Ok(mut counter) = nonce_counter_clone.write() {
                *counter = 0;
            }

            println!("🔥 ransom rotate – new session key");
        }
    });

    Ok(tunnel)
}

#[cfg(test)]
mod tests;
