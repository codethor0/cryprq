// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use alloc::vec::Vec;
use hkdf::hmac::{Hmac, Mac};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

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

/// Handshake salt for the authenticated v2 handshake
pub const SALT_HS_V2: &[u8] = b"cryp-rq v2 hs";

const LABEL_V2_CONFIRM_I: &[u8] = b"cryp-rq v2 confirm key initiator";
const LABEL_V2_CONFIRM_R: &[u8] = b"cryp-rq v2 confirm key responder";
const LABEL_V2_MASTER: &[u8] = b"cryp-rq v2 master secret";
const LABEL_V2_SESSION_ID: &[u8] = b"cryp-rq v2 session id";

/// Running SHA-256 hash over an unambiguously framed handshake transcript.
///
/// Every element is absorbed as `len(label) || label || len(data) || data` with
/// 32-bit big-endian lengths, so no two distinct element sequences hash alike.
#[derive(Clone)]
pub struct TranscriptHash(Sha256);

impl TranscriptHash {
    /// Starts a transcript under a protocol-specific domain separator.
    pub fn new(domain: &[u8]) -> Self {
        let mut transcript = Self(Sha256::new());
        transcript.absorb(b"domain", domain);
        transcript
    }

    /// Appends one labelled element to the transcript.
    pub fn absorb(&mut self, label: &[u8], data: &[u8]) {
        self.0.update((label.len() as u32).to_be_bytes());
        self.0.update(label);
        self.0.update((data.len() as u32).to_be_bytes());
        self.0.update(data);
    }

    /// Hash of everything absorbed so far; the transcript can keep growing.
    pub fn current(&self) -> [u8; 32] {
        self.0.clone().finalize().into()
    }
}

/// Secrets of one authenticated session, all bound to its handshake transcript.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionSecrets {
    /// Key for the initiator's key-confirmation MAC
    pub confirm_key_initiator: [u8; 32],
    /// Key for the responder's key-confirmation MAC
    pub confirm_key_responder: [u8; 32],
    /// Master secret from which epoch traffic keys are derived
    pub master_secret: [u8; 32],
    /// Public session identifier (safe to log)
    #[zeroize(skip)]
    pub session_id: [u8; 16],
}

/// Derives the v2 session secrets from the hybrid shared secrets and the
/// transcript hash.
///
/// `PRK = HKDF-Extract(SALT_HS_V2, ss_kem || ss_x)`; each output is
/// `HKDF-Expand(PRK, label || transcript_hash)`. Any difference in the
/// transcript (identities, roles, key shares, ciphertext, version, suite)
/// therefore yields unrelated keys.
#[allow(clippy::expect_used)] // HKDF-Expand cannot fail for outputs of <= 32 bytes
pub fn derive_session_secrets(
    ss_kem: &[u8; 32],
    ss_x: &[u8; 32],
    transcript_hash: &[u8; 32],
) -> SessionSecrets {
    let mut ikm = [0u8; 64];
    ikm[..32].copy_from_slice(ss_kem);
    ikm[32..].copy_from_slice(ss_x);
    let (_, hk) = Hkdf::<Sha256>::extract(Some(SALT_HS_V2), &ikm);
    ikm.zeroize();

    let mut secrets = SessionSecrets {
        confirm_key_initiator: [0u8; 32],
        confirm_key_responder: [0u8; 32],
        master_secret: [0u8; 32],
        session_id: [0u8; 16],
    };
    hk.expand_multi_info(
        &[LABEL_V2_CONFIRM_I, transcript_hash],
        &mut secrets.confirm_key_initiator,
    )
    .expect("HKDF expand should not fail for 32 bytes");
    hk.expand_multi_info(
        &[LABEL_V2_CONFIRM_R, transcript_hash],
        &mut secrets.confirm_key_responder,
    )
    .expect("HKDF expand should not fail for 32 bytes");
    hk.expand_multi_info(
        &[LABEL_V2_MASTER, transcript_hash],
        &mut secrets.master_secret,
    )
    .expect("HKDF expand should not fail for 32 bytes");
    hk.expand_multi_info(
        &[LABEL_V2_SESSION_ID, transcript_hash],
        &mut secrets.session_id,
    )
    .expect("HKDF expand should not fail for 16 bytes");
    secrets
}

#[allow(clippy::expect_used)] // HMAC accepts keys of any length
fn confirmation_hmac(key: &[u8; 32], label: &[u8], transcript_hash: &[u8; 32]) -> Hmac<Sha256> {
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC accepts keys of any length");
    mac.update(&(label.len() as u32).to_be_bytes());
    mac.update(label);
    mac.update(transcript_hash);
    mac
}

/// Key-confirmation tag: HMAC-SHA256 over a role label and the transcript hash.
pub fn confirmation_mac(key: &[u8; 32], label: &[u8], transcript_hash: &[u8; 32]) -> [u8; 32] {
    confirmation_hmac(key, label, transcript_hash)
        .finalize()
        .into_bytes()
        .into()
}

/// Verifies a key-confirmation tag in constant time.
pub fn verify_confirmation_mac(
    key: &[u8; 32],
    label: &[u8],
    transcript_hash: &[u8; 32],
    tag: &[u8],
) -> bool {
    confirmation_hmac(key, label, transcript_hash)
        .verify_slice(tag)
        .is_ok()
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

    #[test]
    fn test_transcript_framing_is_unambiguous() {
        let mut a = TranscriptHash::new(b"d");
        a.absorb(b"x", b"ab");
        a.absorb(b"x", b"c");
        let mut b = TranscriptHash::new(b"d");
        b.absorb(b"x", b"a");
        b.absorb(b"x", b"bc");
        assert_ne!(a.current(), b.current());
        assert_ne!(
            TranscriptHash::new(b"d").current(),
            TranscriptHash::new(b"e").current()
        );
    }

    #[test]
    fn test_session_secrets_bind_transcript_and_both_shares() {
        let base = derive_session_secrets(&[1; 32], &[2; 32], &[3; 32]);
        let same = derive_session_secrets(&[1; 32], &[2; 32], &[3; 32]);
        assert_eq!(base.master_secret, same.master_secret);
        for other in [
            derive_session_secrets(&[9; 32], &[2; 32], &[3; 32]),
            derive_session_secrets(&[1; 32], &[9; 32], &[3; 32]),
            derive_session_secrets(&[1; 32], &[2; 32], &[9; 32]),
        ] {
            assert_ne!(base.master_secret, other.master_secret);
            assert_ne!(base.confirm_key_initiator, other.confirm_key_initiator);
            assert_ne!(base.confirm_key_responder, other.confirm_key_responder);
            assert_ne!(base.session_id, other.session_id);
        }
        assert_ne!(base.confirm_key_initiator, base.confirm_key_responder);
        assert_ne!(base.confirm_key_initiator, base.master_secret);
        // v2 keys are unrelated to the unauthenticated v1 schedule.
        assert_ne!(
            base.master_secret,
            derive_handshake_keys(&[1; 32], &[2; 32]).1
        );
    }

    #[test]
    fn test_confirmation_mac_rejects_any_change() {
        let tag = confirmation_mac(&[7; 32], b"initiator", &[8; 32]);
        assert!(verify_confirmation_mac(
            &[7; 32],
            b"initiator",
            &[8; 32],
            &tag
        ));
        assert!(!verify_confirmation_mac(
            &[6; 32],
            b"initiator",
            &[8; 32],
            &tag
        ));
        assert!(!verify_confirmation_mac(
            &[7; 32],
            b"responder",
            &[8; 32],
            &tag
        ));
        assert!(!verify_confirmation_mac(
            &[7; 32],
            b"initiator",
            &[9; 32],
            &tag
        ));
        assert!(!verify_confirmation_mac(
            &[7; 32],
            b"initiator",
            &[8; 32],
            &tag[..31]
        ));
        let mut flipped = tag;
        flipped[31] ^= 1;
        assert!(!verify_confirmation_mac(
            &[7; 32],
            b"initiator",
            &[8; 32],
            &flipped
        ));
    }
}
