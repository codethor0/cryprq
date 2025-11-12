// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

#![no_main]

use cryprq_crypto::{kyber_keypair, HybridHandshake};
use libfuzzer_sys::fuzz_target;
use pqcrypto_traits::kem::{PublicKey, SecretKey};

fuzz_target!(|data: &[u8]| {
    // Fuzz test hybrid handshake creation
    if data.len() >= 32 {
        let handshake = HybridHandshake::new();

        // Verify keys are non-zero
        let kyber_pk = handshake.kyber_public_key();
        let kyber_sk = handshake.kyber_secret_key();

        // Basic sanity checks
        assert!(kyber_pk.as_bytes().iter().any(|&b| b != 0));
        assert!(kyber_sk.as_bytes().iter().any(|&b| b != 0));
    }

    // Fuzz test keypair generation
    if data.len() >= 16 {
        let (pk, sk) = kyber_keypair();
        assert!(pk.as_bytes().iter().any(|&b| b != 0));
        assert!(sk.as_bytes().iter().any(|&b| b != 0));
    }
});
