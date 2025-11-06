//! Post-quantum cryptographic primitives
//!
//! This crate provides Kyber768 key encapsulation mechanism (KEM) support.
//!
//! # Status
//!
//! Currently using stub implementation with random bytes.
//! Production version will integrate real Kyber768 from rosenpass or pqcrypto.
//!
//! # Security Warning
//!
//! The current implementation is NOT cryptographically secure.
//! It generates random bytes instead of proper Kyber768 keys.
//! Do not use in production until real Kyber768 is integrated.

#![no_std]
use rand_core::{OsRng, RngCore};

/// Generate Kyber768 key pair
///
/// # Returns
///
/// Tuple of (public_key, secret_key) where:
/// - public_key: 32 bytes (stub - should be 1184 bytes for real Kyber768)
/// - secret_key: 1024 bytes (stub - should be 2400 bytes for real Kyber768)
///
/// # Security
///
/// STUB IMPLEMENTATION: Returns cryptographically random bytes,
/// not actual Kyber768 keys. Waiting for rosenpass library integration.
///
/// # Example
///
/// ```
/// use cryprq_crypto::make_kyber_keys;
///
/// let (pk, sk) = make_kyber_keys();
/// assert_eq!(pk.len(), 32);
/// assert_eq!(sk.len(), 1024);
/// ```
pub fn make_kyber_keys() -> ([u8; 32], [u8; 1024]) {
    let mut pk = [0u8; 32];
    let mut sk = [0u8; 1024];
    // TODO: real Kyber768 key-gen when rosenpass exposes it
    OsRng.fill_bytes(&mut pk);
    OsRng.fill_bytes(&mut sk);
    (pk, sk)
}

#[cfg(test)]
mod tests;
