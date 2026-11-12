// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! ML-KEM-768 (FIPS 203) for the handshake.
//!
//! This module is the only place that names the ML-KEM implementation. The
//! rest of the workspace uses the functions and byte formats defined here:
//!
//! | Value | Bytes | Encoding |
//! |-------|-------|----------|
//! | public (encapsulation) key | 1184 | FIPS 203 `ek` |
//! | secret (decapsulation) key | 2400 | FIPS 203 `dk` (expanded form) |
//! | ciphertext | 1088 | FIPS 203 `c` |
//! | shared secret | 32 | FIPS 203 `K` |
//!
//! These are the encodings that go on the wire in the handshake, so they must
//! not change with the implementation behind them.
//!
//! Randomness comes from the operating system through `rand::rngs::OsRng`:
//! 64 bytes (`d || z`) for a key pair and 32 bytes (`m`) for an
//! encapsulation, exactly the inputs FIPS 203 defines for
//! `ML-KEM.KeyGen_internal` and `ML-KEM.Encaps_internal`.

use alloc::vec::Vec;
use core::fmt;

use ml_kem::kem::Decapsulate;
use ml_kem::ml_kem_768::{DecapsulationKey, EncapsulationKey};
#[allow(deprecated)] // the expanded `dk` encoding is this module's stable format
use ml_kem::ExpandedKeyEncoding;
use ml_kem::{array::Array, KeyExport, Seed, B32};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

/// Length of an encoded public key.
pub const PUBLIC_KEY_LEN: usize = 1184;
/// Length of an encoded secret key (expanded form).
pub const SECRET_KEY_LEN: usize = 2400;
/// Length of a ciphertext.
pub const CIPHERTEXT_LEN: usize = 1088;
/// Length of a shared secret.
pub const SHARED_SECRET_LEN: usize = 32;
/// Length of the seed (`d || z`) a key pair is derived from.
pub const KEYPAIR_SEED_LEN: usize = 64;
/// Length of the randomness (`m`) of one encapsulation.
pub const ENCAPS_RANDOMNESS_LEN: usize = 32;

/// Why a byte string was not accepted as an ML-KEM value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlKemError {
    /// The input does not have the length of the value it should encode.
    InvalidLength {
        /// Length the encoding must have.
        expected: usize,
        /// Length that was given.
        actual: usize,
    },
    /// The input has the right length but is not a valid key (FIPS 203,
    /// section 7.2 and 7.3 input checks).
    InvalidKey,
}

impl fmt::Display for MlKemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(f, "expected {expected} bytes, got {actual}")
            }
            Self::InvalidKey => f.write_str("not a valid ML-KEM-768 key"),
        }
    }
}

impl core::error::Error for MlKemError {}

fn check_len(bytes: &[u8], expected: usize) -> Result<(), MlKemError> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(MlKemError::InvalidLength {
            expected,
            actual: bytes.len(),
        })
    }
}

/// An ML-KEM-768 public (encapsulation) key.
#[derive(Clone)]
pub struct PublicKey {
    key: EncapsulationKey,
    bytes: [u8; PUBLIC_KEY_LEN],
}

impl PublicKey {
    fn from_key(key: EncapsulationKey) -> Self {
        let mut bytes = [0u8; PUBLIC_KEY_LEN];
        bytes.copy_from_slice(key.to_bytes().as_slice());
        Self { key, bytes }
    }

    /// Decodes and validates a public key. A key that fails the FIPS 203
    /// encapsulation-key check is refused.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlKemError> {
        check_len(bytes, PUBLIC_KEY_LEN)?;
        let encoded = Array::try_from(bytes).map_err(|_| MlKemError::InvalidKey)?;
        let key = EncapsulationKey::new(&encoded).map_err(|_| MlKemError::InvalidKey)?;
        Ok(Self::from_key(key))
    }

    /// The encoded key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for PublicKey {}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicKey")
            .field("len", &self.bytes.len())
            .finish()
    }
}

/// An ML-KEM-768 secret (decapsulation) key. Wiped when dropped.
pub struct SecretKey {
    key: DecapsulationKey,
    bytes: Zeroizing<Vec<u8>>,
}

impl SecretKey {
    #[allow(deprecated)] // see the import
    fn from_key(key: DecapsulationKey) -> Self {
        let bytes = Zeroizing::new(key.to_expanded_bytes().as_slice().to_vec());
        Self { key, bytes }
    }

    /// Decodes and validates a secret key in its expanded form.
    #[allow(deprecated)] // see the import
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlKemError> {
        check_len(bytes, SECRET_KEY_LEN)?;
        let encoded = Array::try_from(bytes).map_err(|_| MlKemError::InvalidKey)?;
        let key =
            DecapsulationKey::from_expanded_bytes(&encoded).map_err(|_| MlKemError::InvalidKey)?;
        Ok(Self {
            key,
            bytes: Zeroizing::new(bytes.to_vec()),
        })
    }

    /// The encoded key (expanded form). This is secret key material.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The public key that belongs to this secret key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_key(self.key.encapsulation_key().clone())
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(<redacted>)")
    }
}

/// An ML-KEM-768 ciphertext. Any byte string of the right length is one:
/// decapsulation never fails, it yields a pseudorandom secret for a
/// ciphertext that was not produced for the key (implicit rejection).
#[derive(Clone, PartialEq, Eq)]
pub struct Ciphertext {
    bytes: [u8; CIPHERTEXT_LEN],
}

impl Ciphertext {
    /// Takes a ciphertext from its encoding.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlKemError> {
        check_len(bytes, CIPHERTEXT_LEN)?;
        let mut owned = [0u8; CIPHERTEXT_LEN];
        owned.copy_from_slice(bytes);
        Ok(Self { bytes: owned })
    }

    /// The encoded ciphertext.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ciphertext")
            .field("len", &self.bytes.len())
            .finish()
    }
}

/// The 32-byte secret both sides derive. Wiped when dropped.
#[derive(Clone)]
pub struct SharedSecret {
    bytes: Zeroizing<[u8; SHARED_SECRET_LEN]>,
}

impl SharedSecret {
    fn from_array(mut key: B32) -> Self {
        let mut bytes = Zeroizing::new([0u8; SHARED_SECRET_LEN]);
        bytes.copy_from_slice(key.as_slice());
        key.as_mut_slice().zeroize();
        Self { bytes }
    }

    /// The secret bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl PartialEq for SharedSecret {
    fn eq(&self, other: &Self) -> bool {
        // Not constant time; equality of secrets is only compared in tests.
        self.bytes[..] == other.bytes[..]
    }
}

impl Eq for SharedSecret {}

impl fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SharedSecret(<redacted>)")
    }
}

/// Generates a key pair from operating-system randomness.
pub fn keypair() -> (PublicKey, SecretKey) {
    let mut seed = Zeroizing::new([0u8; KEYPAIR_SEED_LEN]);
    OsRng.fill_bytes(&mut seed[..]);
    keypair_from_seed(&seed)
}

/// `ML-KEM.KeyGen_internal(d, z)` with `seed = d || z`.
///
/// For known-answer tests. Outside of tests the seed must be 64 fresh,
/// uniformly random bytes, which is what [`keypair`] supplies.
#[doc(hidden)]
pub fn keypair_from_seed(seed: &[u8; KEYPAIR_SEED_LEN]) -> (PublicKey, SecretKey) {
    let mut encoded = Seed::default();
    encoded.as_mut_slice().copy_from_slice(seed);
    let key = DecapsulationKey::from_seed(encoded);
    encoded.as_mut_slice().zeroize();
    let secret = SecretKey::from_key(key);
    (secret.public_key(), secret)
}

/// Encapsulates to `public_key` with operating-system randomness and returns
/// the shared secret and the ciphertext to send.
pub fn encapsulate(public_key: &PublicKey) -> (SharedSecret, Ciphertext) {
    let mut randomness = Zeroizing::new([0u8; ENCAPS_RANDOMNESS_LEN]);
    OsRng.fill_bytes(&mut randomness[..]);
    encapsulate_with_randomness(public_key, &randomness)
}

/// `ML-KEM.Encaps_internal(ek, m)`.
///
/// For known-answer tests. Outside of tests `m` must be 32 fresh, uniformly
/// random bytes, which is what [`encapsulate`] supplies.
#[doc(hidden)]
pub fn encapsulate_with_randomness(
    public_key: &PublicKey,
    randomness: &[u8; ENCAPS_RANDOMNESS_LEN],
) -> (SharedSecret, Ciphertext) {
    let mut m = B32::default();
    m.as_mut_slice().copy_from_slice(randomness);
    let (ciphertext, key) = public_key.key.encapsulate_deterministic(&m);
    m.as_mut_slice().zeroize();
    let mut bytes = [0u8; CIPHERTEXT_LEN];
    bytes.copy_from_slice(ciphertext.as_slice());
    (SharedSecret::from_array(key), Ciphertext { bytes })
}

/// `ML-KEM.Decaps(dk, c)`. Always returns a secret: for a ciphertext that
/// does not belong to the key it is the implicit-rejection value, which
/// differs from the sender's secret and reveals nothing about the key.
pub fn decapsulate(ciphertext: &Ciphertext, secret_key: &SecretKey) -> SharedSecret {
    let mut encoded = ml_kem::ml_kem_768::Ciphertext::default();
    encoded.as_mut_slice().copy_from_slice(&ciphertext.bytes);
    SharedSecret::from_array(secret_key.key.decapsulate(&encoded))
}
