#[cfg(test)]
mod crypto_tests {
    use crate::make_kyber_keys;

    #[test]
    fn test_kyber_keys_non_zero() {
        let (pk, sk) = make_kyber_keys();
        
        // Keys should not be all zeros
        assert!(pk.iter().any(|&b| b != 0), "Public key should not be all zeros");
        assert!(sk.iter().any(|&b| b != 0), "Secret key should not be all zeros");
    }

    #[test]
    fn test_kyber_keys_unique() {
        let (pk1, sk1) = make_kyber_keys();
        let (pk2, sk2) = make_kyber_keys();
        
        // Different invocations should produce different keys
        assert_ne!(pk1, pk2, "Public keys should be unique");
        assert_ne!(sk1, sk2, "Secret keys should be unique");
    }

    #[test]
    fn test_kyber_keys_length() {
        let (pk, sk) = make_kyber_keys();
        
        // Verify Kyber768 standard sizes
        assert_eq!(pk.len(), 1184, "Kyber768 public key should be 1184 bytes");
        assert_eq!(sk.len(), 2400, "Kyber768 secret key should be 2400 bytes");
    }
}
