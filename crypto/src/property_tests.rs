// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

#[cfg(test)]
#[allow(clippy::module_inception)]
mod property_tests {
    use crate::hybrid::HybridHandshake;
    use alloc::vec::Vec;
    use pqcrypto_traits::kem::{PublicKey, SecretKey};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_hybrid_handshake_symmetry(
            _seed in any::<u64>()
        ) {
            // Property: Hybrid handshake should produce valid keys
            // Each handshake should produce consistent key sizes
            let h1 = HybridHandshake::new();
            let h2 = HybridHandshake::new();

            // Both should produce valid X25519 keys (32 bytes)
            prop_assert_eq!(h1.x25519_secret().as_bytes().len(), 32);
            prop_assert_eq!(h2.x25519_secret().as_bytes().len(), 32);

            // Kyber keys - full assertions with trait imports
            prop_assert_eq!(h1.kyber_public_key().as_bytes().len(), 1184, "Kyber768 PK must be 1184 bytes");
            prop_assert_eq!(h1.kyber_secret_key().as_bytes().len(), 2400, "Kyber768 SK must be 2400 bytes");
            prop_assert_eq!(h2.kyber_public_key().as_bytes().len(), 1184, "Kyber768 PK must be 1184 bytes");
            prop_assert_eq!(h2.kyber_secret_key().as_bytes().len(), 2400, "Kyber768 SK must be 2400 bytes");

            // Keys should be non-zero
            prop_assert!(h1.kyber_public_key().as_bytes().iter().any(|&b| b != 0));
            prop_assert!(h1.kyber_secret_key().as_bytes().iter().any(|&b| b != 0));
        }

        #[test]
        fn test_handshake_idempotence(
            iterations in 1..100u32
        ) {
            // Property: Multiple handshakes should be independent
            // Creating N handshakes should produce N unique keypairs
            let mut handshakes = Vec::new();
            for _ in 0..iterations {
                handshakes.push(HybridHandshake::new());
            }

            // All handshakes should be unique
            for i in 0..handshakes.len() {
                for j in (i+1)..handshakes.len() {
                    let h1 = &handshakes[i];
                    let h2 = &handshakes[j];

                    prop_assert_ne!(
                        h1.x25519_secret().as_bytes(),
                        h2.x25519_secret().as_bytes(),
                        "X25519 secrets must be unique"
                    );
                    prop_assert_ne!(
                        h1.kyber_public_key().as_bytes(),
                        h2.kyber_public_key().as_bytes(),
                        "Kyber public keys must be unique"
                    );
                }
            }
        }

        #[test]
        fn test_key_sizes_consistent(
            _dummy in any::<u8>()
        ) {
            // Property: Key sizes should be consistent across all handshakes
            let h = HybridHandshake::new();

            // X25519 secret is always 32 bytes
            prop_assert_eq!(h.x25519_secret().as_bytes().len(), 32);

            // Kyber key sizes - full assertions
            prop_assert_eq!(h.kyber_public_key().as_bytes().len(), 1184, "Kyber768 PK must be 1184 bytes");
            prop_assert_eq!(h.kyber_secret_key().as_bytes().len(), 2400, "Kyber768 SK must be 2400 bytes");
        }
    }
}
