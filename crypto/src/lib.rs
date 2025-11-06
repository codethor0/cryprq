//! Post-quantum cryptographic primitives
//!
//! This crate provides Kyber768 key encapsulation mechanism (KEM) support
//! using the pqcrypto-kyber implementation.
//!
//! # Security
//!
//! Uses NIST-standardized Kyber768 (CRYSTALS-Kyber) for post-quantum
//! key encapsulation. Provides protection against quantum computer attacks.
//!
//! # Implementation
//!
//! Based on pqcrypto-kyber which implements the NIST PQC standard.
//! Key sizes:
//! - Public key: 1184 bytes
//! - Secret key: 2400 bytes

#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{PublicKey, SecretKey};

/// Generate Kyber768 key pair
///
/// # Returns
///
/// Tuple of (public_key, secret_key) where:
/// - public_key: 1184 bytes (NIST Kyber768 standard)
/// - secret_key: 2400 bytes (NIST Kyber768 standard)
///
/// # Security
///
/// Uses CRYSTALS-Kyber768, a NIST-standardized post-quantum KEM.
/// Provides IND-CCA2 security against quantum adversaries.
///
/// # Example
///
/// ```
/// use cryprq_crypto::make_kyber_keys;
///
/// let (pk, sk) = make_kyber_keys();
/// assert_eq!(pk.len(), 1184); // Kyber768 public key size
/// assert_eq!(sk.len(), 2400); // Kyber768 secret key size
/// ```
pub fn make_kyber_keys() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = kyber768::keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

#[cfg(test)]
mod tests;
