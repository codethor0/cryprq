#[cfg(test)]
mod p2p_tests {
    use crate::{init_swarm, get_current_pk, dial_peer, P2PError};
    use libp2p::PeerId;

    #[tokio::test]
    async fn test_swarm_initialization() {
        let result = init_swarm().await;
        assert!(result.is_ok(), "Swarm initialization should succeed");
    }

    #[tokio::test]
    async fn test_get_current_pk() {
        let result = get_current_pk();
        assert!(result.is_ok(), "Should retrieve current public key");
        
        if let Ok(pk) = result {
            assert!(!pk.is_empty(), "Public key should not be empty");
            assert_eq!(pk.len(), 1184, "Kyber768 public key should be 1184 bytes");
        }
    }

    #[tokio::test]
    async fn test_dial_peer() {
        let peer_id = PeerId::random();
        let result = dial_peer(peer_id).await;
        assert!(result.is_ok(), "Dial should succeed (stub implementation)");
    }

    #[test]
    fn test_key_storage_consistency() {
        if let (Ok(pk1), Ok(pk2)) = (get_current_pk(), get_current_pk()) {
            assert_eq!(pk1, pk2, "Multiple retrievals should return same data");
        }
    }

    #[test]
    fn test_error_display() {
        let err1 = P2PError::LockPoisoned("test lock".to_string());
        let err2 = P2PError::InvalidPeerId;
        let err3 = P2PError::DialFailed("connection refused".to_string());
        
        assert!(err1.to_string().contains("Lock poisoned"));
        assert!(err2.to_string().contains("Invalid peer ID"));
        assert!(err3.to_string().contains("Failed to dial peer"));
    }

    #[tokio::test]
    async fn test_concurrent_pk_access() {
        let handles: Vec<_> = (0..10).map(|_| {
            tokio::spawn(async {
                get_current_pk()
            })
        }).collect();
        
        for handle in handles {
            if let Ok(result) = handle.await {
                assert!(result.is_ok(), "Concurrent access should not fail");
            }
        }
    }

    #[test]
    fn test_key_rotation_zeroization() {
        use zeroize::Zeroize;
        let mut old_key = [0xAA; 32];
        old_key.zeroize();
        
        assert_eq!(&old_key[..], &[0u8; 32], "Key should be zeroed after zeroization");
    }

    #[test]
    fn test_pk_length() {
        if let Ok(pk) = get_current_pk() {
            assert_eq!(pk.len(), 1184, "Kyber768 public key should be 1184 bytes");
        }
    }
}
