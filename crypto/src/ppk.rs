// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Post-Quantum Pre-Shared Keys (PPKs)
//!
//! This module implements PPK derivation using ML-KEM shared secrets.
//! PPKs provide enhanced security for peer authentication and can be
//! rotated independently of session keys.

use alloc::vec::Vec;
use blake3::Hasher;
use zeroize::ZeroizeOnDrop;

// For no_std compatibility, we'll use a timestamp-based approach
// In std environments, this can use SystemTime; in no_std, use a provided timestamp

/// Post-Quantum Pre-Shared Key
///
/// Derived from ML-KEM shared secret and peer identity.
/// Expires with key rotation to ensure forward secrecy.
#[derive(Clone, ZeroizeOnDrop)]
pub struct PostQuantumPSK {
    /// The PPK value (32 bytes)
    #[zeroize(on_drop)]
    key: [u8; 32],
    /// Peer ID this PPK is associated with
    peer_id: [u8; 32],
    /// Timestamp when this PPK was created
    created_at: u64,
    /// Timestamp when this PPK expires (typically rotation interval)
    expires_at: u64,
}

impl PostQuantumPSK {
    /// Derive a PPK from ML-KEM shared secret and peer identity
    ///
    /// # Arguments
    ///
    /// * `kyber_shared` - Shared secret from ML-KEM key exchange (32 bytes)
    /// * `peer_id` - Peer identity (32 bytes, typically Ed25519 public key)
    /// * `salt` - Random salt for this derivation (16 bytes)
    /// * `rotation_interval_secs` - Key rotation interval in seconds
    /// * `current_timestamp_secs` - Current Unix timestamp in seconds
    ///
    /// # Returns
    ///
    /// A new `PostQuantumPSK` that expires after `rotation_interval_secs`
    pub fn derive(
        kyber_shared: &[u8; 32],
        peer_id: &[u8; 32],
        salt: &[u8; 16],
        rotation_interval_secs: u64,
        current_timestamp_secs: u64,
    ) -> Self {
        // BLAKE3 KDF: hash(kyber_shared || peer_id || salt)
        let mut hasher = Hasher::new();
        hasher.update(kyber_shared);
        hasher.update(peer_id);
        hasher.update(salt);
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash.as_bytes()[..32]);

        Self {
            key,
            peer_id: *peer_id,
            created_at: current_timestamp_secs,
            expires_at: current_timestamp_secs + rotation_interval_secs,
        }
    }

    /// Get the PPK value (for use in authentication)
    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Get the peer ID this PPK is associated with
    pub fn peer_id(&self) -> &[u8; 32] {
        &self.peer_id
    }

    /// Check if this PPK has expired
    ///
    /// # Arguments
    ///
    /// * `current_timestamp_secs` - Current Unix timestamp in seconds
    pub fn is_expired_at(&self, current_timestamp_secs: u64) -> bool {
        current_timestamp_secs >= self.expires_at
    }

    /// Get time until expiration (in seconds)
    ///
    /// # Arguments
    ///
    /// * `current_timestamp_secs` - Current Unix timestamp in seconds
    pub fn expires_in_at(&self, current_timestamp_secs: u64) -> u64 {
        if current_timestamp_secs >= self.expires_at {
            0
        } else {
            self.expires_at - current_timestamp_secs
        }
    }
}

/// PPK Storage (in-memory for now; encrypted at-rest storage can be added later)
pub struct PPKStore {
    ppks: Vec<PostQuantumPSK>,
}

impl PPKStore {
    pub fn new() -> Self {
        Self { ppks: Vec::new() }
    }

    /// Store a PPK for a peer
    pub fn store(&mut self, ppk: PostQuantumPSK) {
        // Remove existing PPK for this peer if present
        self.ppks.retain(|p| p.peer_id() != ppk.peer_id());
        self.ppks.push(ppk);
    }

    /// Get the current PPK for a peer (if not expired)
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Peer identity bytes
    /// * `current_timestamp_secs` - Current Unix timestamp in seconds
    pub fn get(&self, peer_id: &[u8; 32], current_timestamp_secs: u64) -> Option<&PostQuantumPSK> {
        self.ppks
            .iter()
            .find(|p| p.peer_id() == peer_id && !p.is_expired_at(current_timestamp_secs))
    }

    /// Remove expired PPKs
    ///
    /// # Arguments
    ///
    /// * `current_timestamp_secs` - Current Unix timestamp in seconds
    pub fn cleanup_expired(&mut self, current_timestamp_secs: u64) {
        self.ppks
            .retain(|p| !p.is_expired_at(current_timestamp_secs));
    }

    /// Remove all PPKs for a peer
    pub fn remove_peer(&mut self, peer_id: &[u8; 32]) {
        self.ppks.retain(|p| p.peer_id() != peer_id);
    }
}

impl Default for PPKStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppk_derivation() {
        let kyber_shared = [1u8; 32];
        let peer_id = [2u8; 32];
        let salt = [3u8; 16];

        let ppk = PostQuantumPSK::derive(&kyber_shared, &peer_id, &salt, 300);

        assert_eq!(ppk.peer_id(), &peer_id);
        assert!(!ppk.is_expired());
        assert!(ppk.expires_in() <= 300);
    }

    #[test]
    fn test_ppk_expiration() {
        let kyber_shared = [1u8; 32];
        let peer_id = [2u8; 32];
        let salt = [3u8; 16];

        let ppk = PostQuantumPSK::derive(&kyber_shared, &peer_id, &salt, 1);

        // Should not be expired immediately
        assert!(!ppk.is_expired());

        // Wait 2 seconds
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should be expired now
        assert!(ppk.is_expired());
    }

    #[test]
    fn test_ppk_store() {
        let mut store = PPKStore::new();

        let kyber_shared = [1u8; 32];
        let peer_id = [2u8; 32];
        let salt = [3u8; 16];

        let ppk = PostQuantumPSK::derive(&kyber_shared, &peer_id, &salt, 300);
        store.store(ppk);

        // Should retrieve the PPK
        assert!(store.get(&peer_id).is_some());

        // Different peer ID should return None
        let other_peer = [4u8; 32];
        assert!(store.get(&other_peer).is_none());

        // Remove peer
        store.remove_peer(&peer_id);
        assert!(store.get(&peer_id, 1000).is_none());
    }
}
