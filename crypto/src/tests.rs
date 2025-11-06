#[cfg(test)]
mod crypto_tests {
    use crate::make_kyber_keys;

    #[test]
    fn test_kyber_keys_non_zero() {
        let (pk, sk) = make_kyber_keys();
        
        // Keys should not be all zeros
        assert_ne!(&pk[..], &[0u8; 32]);
        assert_ne!(&sk[..], &[0u8; 1024]);
    }

    #[test]
    fn test_kyber_keys_unique() {
        let (pk1, sk1) = make_kyber_keys();
        let (pk2, sk2) = make_kyber_keys();
        
        // Different invocations should produce different keys
        assert_ne!(&pk1[..], &pk2[..]);
        assert_ne!(&sk1[..], &sk2[..]);
    }

    #[test]
    fn test_kyber_keys_length() {
        let (pk, sk) = make_kyber_keys();
        
        // Verify buffer sizes
        assert_eq!(pk.len(), 32);
        assert_eq!(sk.len(), 1024);
    }
}
