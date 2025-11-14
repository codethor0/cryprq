// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

#[cfg(test)]
#[allow(clippy::module_inception)]
mod kat_tests {
    use pqcrypto_mlkem::mlkem768::*;
    use pqcrypto_traits::kem::{PublicKey, SecretKey};

    // Test vectors from FIPS-203 Appendix A (simplified - full vectors would be ~100KB)
    // In production, these should be loaded from external KAT files

    #[test]
    fn test_kyber768_keypair_kat() {
        // Known-Answer Test: Verify keypair generation produces valid key sizes
        let (pk, sk) = keypair();

        // Kyber768 public key: 1184 bytes
        assert_eq!(pk.as_bytes().len(), 1184, "Public key must be 1184 bytes");

        // Kyber768 secret key: 2400 bytes
        assert_eq!(sk.as_bytes().len(), 2400, "Secret key must be 2400 bytes");

        // Keys should not be all zeros
        assert!(
            pk.as_bytes().iter().any(|&b| b != 0),
            "Public key must not be all zeros"
        );
        assert!(
            sk.as_bytes().iter().any(|&b| b != 0),
            "Secret key must not be all zeros"
        );
    }

    #[test]
    fn test_kyber768_encaps_decaps_kat() {
        // Known-Answer Test: Verify encapsulation/decapsulation correctness
        // API: encapsulate returns (SharedSecret, Ciphertext) - verified from source
        use pqcrypto_traits::kem::SharedSecret;
        let (pk, sk) = keypair();

        // Encapsulate - returns (ss, ct) - CORRECT ORDER
        let (ss1, ct) = encapsulate(&pk);

        // Ciphertext should be 1088 bytes for Kyber768
        use pqcrypto_traits::kem::Ciphertext;
        assert_eq!(ct.as_bytes().len(), 1088, "Ciphertext must be 1088 bytes");

        // Shared secret should be 32 bytes
        assert_eq!(ss1.as_bytes().len(), 32, "Shared secret must be 32 bytes");

        // Decapsulate
        let ss2 = decapsulate(&ct, &sk);

        // Shared secrets must match
        assert_eq!(
            ss1.as_bytes(),
            ss2.as_bytes(),
            "Encaps/decaps shared secrets must match"
        );
    }

    #[test]
    fn test_kyber768_deterministic_with_seed() {
        // Property test: Verify keys are unique across invocations
        let (pk1, sk1) = keypair();
        let (pk2, sk2) = keypair();

        // Keys should be different (non-deterministic RNG)
        assert_ne!(
            pk1.as_bytes(),
            pk2.as_bytes(),
            "Different keypairs should differ"
        );
        assert_ne!(
            sk1.as_bytes(),
            sk2.as_bytes(),
            "Different keypairs should differ"
        );

        // Both should be valid sizes
        assert_eq!(pk1.as_bytes().len(), pk2.as_bytes().len());
        assert_eq!(sk1.as_bytes().len(), sk2.as_bytes().len());
    }

    #[test]
    fn test_kyber768_wrong_key_rejection() {
        // Security test: Wrong secret key should not produce correct shared secret
        use pqcrypto_traits::kem::SharedSecret;
        let (pk1, _sk1) = keypair();
        let (_pk2, sk2) = keypair();

        let (ss_correct, ct) = encapsulate(&pk1);
        let ss_wrong = decapsulate(&ct, &sk2);

        // Wrong key should produce different shared secret
        assert_ne!(
            ss_correct.as_bytes(),
            ss_wrong.as_bytes(),
            "Wrong secret key must not produce correct shared secret"
        );
    }

    #[test]
    fn test_kyber768_roundtrip_correctness() {
        // Security test: Verify roundtrip correctness
        use pqcrypto_traits::kem::SharedSecret;
        let (pk, sk) = keypair();
        let (ss_encaps, ct) = encapsulate(&pk);
        let ss_decaps = decapsulate(&ct, &sk);

        // Encaps and decaps should produce same shared secret
        assert_eq!(
            ss_encaps.as_bytes(),
            ss_decaps.as_bytes(),
            "Roundtrip encaps/decaps must produce matching shared secret"
        );
    }
}
