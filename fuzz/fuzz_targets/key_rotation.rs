// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

#![no_main]

use cryprq_crypto::{PPKStore, PostQuantumPSK};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz test key rotation and PPK cleanup
    if data.len() >= 80 {
        let mut store = PPKStore::new();

        // Create multiple PPKs with different expiration times
        for i in 0..(data.len() / 80).min(10) {
            let offset = i * 80;
            if offset + 80 <= data.len() {
                let kyber_shared: [u8; 32] =
                    data[offset..offset + 32].try_into().expect("Invalid slice");
                let peer_id: [u8; 32] = data[offset + 32..offset + 64]
                    .try_into()
                    .expect("Invalid slice");
                let salt: [u8; 16] = data[offset + 64..offset + 80]
                    .try_into()
                    .expect("Invalid slice");

                let rotation_interval = (i as u64 + 1) * 60; // 60, 120, 180, etc.
                let now = 1000u64 + (i as u64 * 60); // Vary timestamps
                let ppk =
                    PostQuantumPSK::derive(&kyber_shared, &peer_id, &salt, rotation_interval, now);
                store.store(ppk);
            }
        }

        // Test cleanup
        let now = 2000u64; // Future timestamp
        store.cleanup_expired(now);

        // Verify store operations don't panic
        if data.len() >= 32 {
            let test_peer: [u8; 32] = data[0..32].try_into().expect("Invalid slice");
            let _ = store.get(&test_peer, now);
            store.remove_peer(&test_peer);
        }
    }
});
