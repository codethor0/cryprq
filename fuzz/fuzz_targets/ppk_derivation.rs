// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

#![no_main]

use cryprq_crypto::PostQuantumPSK;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz test PPK derivation
    // Input: 32 bytes kyber_shared + 32 bytes peer_id + 16 bytes salt + 8 bytes rotation_interval
    if data.len() >= 88 {
        let kyber_shared: [u8; 32] = data[0..32].try_into().expect("Invalid slice length");
        let peer_id: [u8; 32] = data[32..64].try_into().expect("Invalid slice length");
        let salt: [u8; 16] = data[64..80].try_into().expect("Invalid slice length");
        let rotation_bytes: [u8; 8] = data[80..88].try_into().expect("Invalid slice length");
        let rotation_interval = u64::from_le_bytes(rotation_bytes).clamp(1, 3600);

        let now = 1000u64; // Test timestamp
        let ppk = PostQuantumPSK::derive(&kyber_shared, &peer_id, &salt, rotation_interval, now);

        // Verify PPK properties
        assert_eq!(ppk.peer_id(), &peer_id);
        assert!(!ppk.is_expired_at(now)); // Should not be expired immediately
        assert!(ppk.expires_in_at(now) <= rotation_interval);

        // Verify key is non-zero
        assert!(ppk.key().iter().any(|&b| b != 0));
    }
});
