// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! HKDF key derivation functions for CrypRQ v1.0 protocol
//!
//! Implements the key schedule as specified in cryp-rq-protocol-v1.md Section 4.4

use alloc::vec::Vec;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

/// Handshake salt as specified in Section 4.4
pub const SALT_HS: &[u8] = b"cryp-rq v1.0 hs";

/// Label for handshake authentication key
pub const LABEL_HS_AUTH: &[u8] = b"cryp-rq hs auth";

/// Label for master secret
pub const LABEL_MASTER_SECRET: &[u8] = b"cryp-rq master secret";

/// Label for Initiator→Responder encryption key
pub const LABEL_IR_KEY: &[u8] = b"cryp-rq ir key";

/// Label for Initiator→Responder IV
pub const LABEL_IR_IV: &[u8] = b"cryp-rq ir iv";

/// Label for Responder→Initiator encryption key
pub const LABEL_RI_KEY: &[u8] = b"cryp-rq ri key";

/// Label for Responder→Initiator IV
pub const LABEL_RI_IV: &[u8] = b"cryp-rq ri iv";

/// Derives handshake authentication key and master secret from hybrid shared secrets
///
/// As specified in Section 4.4:
/// - Input: `ss_kem || ss_x` (concatenated shared secrets)
/// - Output: `(hs_auth_key, master_secret)`
///
/// # Arguments
///
/// * `ss_kem` - ML-KEM shared secret (32 bytes)
/// * `ss_x` - X25519 shared secret (32 bytes)
///
/// # Returns
///
/// * `hs_auth_key` - Handshake authentication key (32 bytes)
/// * `master_secret` - Master secret (32 bytes)
///
/// # Note
///
/// HKDF expand is guaranteed not to fail for these sizes; expect is acceptable here.
#[allow(clippy::expect_used)]
pub fn derive_handshake_keys(ss_kem: &[u8; 32], ss_x: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    // IKM = ss_kem || ss_x
    let mut ikm = [0u8; 64];
    ikm[..32].copy_from_slice(ss_kem);
    ikm[32..].copy_from_slice(ss_x);

    // Extract PRK
    let (_, hk) = Hkdf::<Sha256>::extract(Some(SALT_HS), &ikm);

    // Expand hs_auth_key
    let mut hs_auth_key = [0u8; 32];
    hk.expand(LABEL_HS_AUTH, &mut hs_auth_key)
        .expect("HKDF expand should not fail for 32 bytes");

    // Expand master_secret
    let mut master_secret = [0u8; 32];
    hk.expand(LABEL_MASTER_SECRET, &mut master_secret)
        .expect("HKDF expand should not fail for 32 bytes");

    // Zeroize IKM
    ikm.zeroize();

    (hs_auth_key, master_secret)
}

/// Derives application traffic keys from master secret
///
/// As specified in Section 4.4.2:
/// - Derives directional keys and IVs for both directions
///
/// # Arguments
///
/// * `master_secret` - Master secret (32 bytes)
/// * `key_len` - Length of encryption key (typically 32 for ChaCha20-Poly1305)
/// * `iv_len` - Length of IV (typically 12 for ChaCha20-Poly1305)
///
/// # Returns
///
/// * `(key_ir, iv_ir, key_ri, iv_ri)` - Traffic keys for both directions
///
/// # Note
///
/// HKDF expand is guaranteed not to fail for these sizes; expect is acceptable here.
#[allow(clippy::expect_used)]
pub fn derive_traffic_keys(
    master_secret: &[u8; 32],
    key_len: usize,
    iv_len: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let (_, hk) = Hkdf::<Sha256>::extract(None, master_secret);

    let mut key_ir = alloc::vec![0u8; key_len];
    hk.expand(LABEL_IR_KEY, &mut key_ir)
        .expect("HKDF expand should not fail");

    let mut iv_ir = alloc::vec![0u8; iv_len];
    hk.expand(LABEL_IR_IV, &mut iv_ir)
        .expect("HKDF expand should not fail");

    let mut key_ri = alloc::vec![0u8; key_len];
    hk.expand(LABEL_RI_KEY, &mut key_ri)
        .expect("HKDF expand should not fail");

    let mut iv_ri = alloc::vec![0u8; iv_len];
    hk.expand(LABEL_RI_IV, &mut iv_ri)
        .expect("HKDF expand should not fail");

    (key_ir, iv_ir, key_ri, iv_ri)
}

/// Derives epoch-scoped traffic keys for key rotation
///
/// As specified in Section 5.3.2:
/// - Keys are scoped by epoch: `"cryp-rq ir key epoch=" || epoch`
///
/// # Arguments
///
/// * `master_secret` - Master secret (32 bytes)
/// * `epoch` - Epoch number (0-255)
/// * `key_len` - Length of encryption key
/// * `iv_len` - Length of IV
///
/// # Returns
///
/// * `(key_ir, iv_ir, key_ri, iv_ri)` - Epoch-scoped traffic keys
///
/// # Note
///
/// HKDF expand is guaranteed not to fail for these sizes; expect is acceptable here.
#[allow(clippy::expect_used)]
pub fn derive_epoch_keys(
    master_secret: &[u8; 32],
    epoch: u8,
    key_len: usize,
    iv_len: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let (_, hk) = Hkdf::<Sha256>::extract(None, master_secret);

    // Build epoch-scoped labels
    let mut label_ir_key = Vec::from(LABEL_IR_KEY);
    label_ir_key.extend_from_slice(b" epoch=");
    label_ir_key.push(epoch);

    let mut label_ir_iv = Vec::from(LABEL_IR_IV);
    label_ir_iv.extend_from_slice(b" epoch=");
    label_ir_iv.push(epoch);

    let mut label_ri_key = Vec::from(LABEL_RI_KEY);
    label_ri_key.extend_from_slice(b" epoch=");
    label_ri_key.push(epoch);

    let mut label_ri_iv = Vec::from(LABEL_RI_IV);
    label_ri_iv.extend_from_slice(b" epoch=");
    label_ri_iv.push(epoch);

    let mut key_ir = alloc::vec![0u8; key_len];
    hk.expand(&label_ir_key, &mut key_ir)
        .expect("HKDF expand should not fail");

    let mut iv_ir = alloc::vec![0u8; iv_len];
    hk.expand(&label_ir_iv, &mut iv_ir)
        .expect("HKDF expand should not fail");

    let mut key_ri = alloc::vec![0u8; key_len];
    hk.expand(&label_ri_key, &mut key_ri)
        .expect("HKDF expand should not fail");

    let mut iv_ri = alloc::vec![0u8; iv_len];
    hk.expand(&label_ri_iv, &mut iv_ri)
        .expect("HKDF expand should not fail");

    (key_ir, iv_ir, key_ri, iv_ri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_handshake_keys() {
        let ss_kem = [0x01u8; 32];
        let ss_x = [0x02u8; 32];

        let (hs_auth_key, master_secret) = derive_handshake_keys(&ss_kem, &ss_x);

        assert_eq!(hs_auth_key.len(), 32);
        assert_eq!(master_secret.len(), 32);
        assert_ne!(hs_auth_key, master_secret);
    }

    #[test]
    fn test_derive_traffic_keys() {
        let master_secret = [0x42u8; 32];
        let (key_ir, iv_ir, key_ri, iv_ri) = derive_traffic_keys(&master_secret, 32, 12);

        assert_eq!(key_ir.len(), 32);
        assert_eq!(iv_ir.len(), 12);
        assert_eq!(key_ri.len(), 32);
        assert_eq!(iv_ri.len(), 12);
        assert_ne!(key_ir, key_ri);
        assert_ne!(iv_ir, iv_ri);
    }

    #[test]
    fn test_derive_epoch_keys() {
        let master_secret = [0x42u8; 32];
        let (key_ir_0, iv_ir_0, key_ri_0, iv_ri_0) = derive_epoch_keys(&master_secret, 0, 32, 12);
        let (key_ir_1, iv_ir_1, key_ri_1, iv_ri_1) = derive_epoch_keys(&master_secret, 1, 32, 12);

        // Different epochs should produce different keys
        assert_ne!(key_ir_0, key_ir_1);
        assert_ne!(iv_ir_0, iv_ir_1);
        assert_ne!(key_ri_0, key_ri_1);
        assert_ne!(iv_ri_0, iv_ri_1);
    }
}
