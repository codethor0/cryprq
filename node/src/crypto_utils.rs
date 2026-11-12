// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use chacha20poly1305::Nonce;

/// Epoch type for key rotation (8-bit, modulo 256)
///
/// As specified in Section 5.3.1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Epoch(pub u8);

impl Epoch {
    /// Initial epoch (0)
    pub fn initial() -> Self {
        Epoch(0)
    }

    /// Increment epoch (wraps at 256)
    pub fn next(self) -> Self {
        Epoch(self.0.wrapping_add(1))
    }

    /// Get the u8 value
    pub fn value(self) -> u8 {
        self.0
    }
}

impl From<u8> for Epoch {
    fn from(value: u8) -> Self {
        Epoch(value)
    }
}

/// Constructs a nonce using TLS 1.3-style XOR as specified in Section 6.2.2
///
/// Nonce construction:
/// - `seq_be = 0x00000000 || seq_64_be` (32 zero bits + 64-bit big-endian seq)
/// - `nonce = static_iv XOR seq_be`
///
/// # Arguments
///
/// * `static_iv` - 96-bit IV derived from key schedule (12 bytes)
/// * `seq` - 64-bit sequence number
///
/// # Returns
///
/// 96-bit nonce (12 bytes)
pub fn make_nonce(static_iv: [u8; 12], seq: u64) -> Nonce {
    let mut nonce = static_iv;
    let seq_be = seq.to_be_bytes(); // 8 bytes

    // XOR the last 8 bytes of IV with seq (TLS 1.3 style)
    // nonce[0..4] stays as-is (first 32 bits of IV)
    // nonce[4..12] gets XORed with seq_be
    for i in 0..8 {
        nonce[4 + i] ^= seq_be[i];
    }

    Nonce::from(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_wrapping() {
        let epoch = Epoch::initial();
        assert_eq!(epoch.value(), 0);

        let epoch = epoch.next();
        assert_eq!(epoch.value(), 1);

        let epoch = Epoch(255);
        let epoch = epoch.next();
        assert_eq!(epoch.value(), 0); // Wraps
    }

    #[test]
    fn test_nonce_construction() {
        let static_iv = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
        ];
        let seq1 = 1u64;
        let seq2 = 2u64;

        let nonce1 = make_nonce(static_iv, seq1);
        let nonce2 = make_nonce(static_iv, seq2);

        // Same IV + same seq = same nonce
        let nonce1_again = make_nonce(static_iv, seq1);
        assert_eq!(&nonce1[..], &nonce1_again[..]);

        // Same IV + different seq = different nonce
        assert_ne!(&nonce1[..], &nonce2[..]);

        // First 4 bytes should be unchanged (not XORed)
        assert_eq!(&nonce1[0..4], &static_iv[0..4]);
        assert_eq!(&nonce2[0..4], &static_iv[0..4]);
    }

    #[test]
    fn test_nonce_deterministic() {
        let static_iv = [0xFF; 12];
        let seq = 0x1234567890ABCDEFu64;

        let nonce1 = make_nonce(static_iv, seq);
        let nonce2 = make_nonce(static_iv, seq);

        assert_eq!(&nonce1[..], &nonce2[..]);
    }
}
