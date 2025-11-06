#[cfg(test)]
mod tunnel_tests {
    use crate::{create_tunnel, TunnelError, MAX_NONCE_VALUE, generate_handshake_auth};

    #[tokio::test]
    async fn test_tunnel_creation() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8001";
        
        // Generate authentication credentials
        let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&peer_pk);
        
        let result = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            listen_addr
        ).await;
        assert!(result.is_ok(), "Tunnel creation should succeed");
    }

    #[tokio::test]
    async fn test_tunnel_send_packet() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8003";
        
        let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&peer_pk);
        
        if let Ok(tunnel) = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            listen_addr
        ).await {
            let test_data = b"test packet data";
            let result = tunnel.send_packet(test_data).await;
            assert!(result.is_ok(), "Sending packet should succeed");
        }
    }

    #[tokio::test]
    async fn test_nonce_overflow_protection() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8005";
        
        let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&peer_pk);
        
        if let Ok(tunnel) = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            listen_addr
        ).await {
            // Set nonce counter to MAX_NONCE_VALUE (at the limit)
            if let Ok(mut nonce_guard) = tunnel.nonce_counter.write()
                .map_err(|e| TunnelError::LockPoisoned(e.to_string())) {
                *nonce_guard = MAX_NONCE_VALUE;
            }
            
            // This send should trigger nonce overflow error
            let result = tunnel.send_packet(b"test").await;
            assert!(matches!(result, Err(TunnelError::NonceOverflow)), 
                "Expected NonceOverflow, got {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_empty_packet() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8007";
        
        let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&peer_pk);
        
        if let Ok(tunnel) = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            listen_addr
        ).await {
            let result = tunnel.send_packet(&[]).await;
            assert!(result.is_ok(), "Empty packets should be allowed");
        }
    }

    #[tokio::test]
    async fn test_large_packet() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8009";
        
        let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&peer_pk);
        
        if let Ok(tunnel) = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            listen_addr
        ).await {
            let large_data = vec![0xAA; 60000]; // Just under MTU
            let result = tunnel.send_packet(&large_data).await;
            assert!(result.is_ok(), "Large packets should succeed");
        }
    }

    #[tokio::test]
    async fn test_key_uniqueness() {
        let local_sk1 = [1u8; 32];
        let peer_pk1 = [2u8; 32];
        let local_sk2 = [3u8; 32];
        let peer_pk2 = [4u8; 32];
        
        let (_, peer_identity_key1, peer_signature1) = generate_handshake_auth(&peer_pk1);
        let (_, peer_identity_key2, peer_signature2) = generate_handshake_auth(&peer_pk2);
        
        if let (Ok(tunnel1), Ok(tunnel2)) = (
            create_tunnel(&local_sk1, &peer_pk1, &peer_identity_key1, &peer_signature1, "127.0.0.1:8011").await,
            create_tunnel(&local_sk2, &peer_pk2, &peer_identity_key2, &peer_signature2, "127.0.0.1:8013").await
        ) {
            if let (Ok(key1_read), Ok(key2_read)) = (
                tunnel1.session_key.read().map_err(|e| TunnelError::LockPoisoned(e.to_string())),
                tunnel2.session_key.read().map_err(|e| TunnelError::LockPoisoned(e.to_string()))
            ) {
                assert_ne!(&key1_read[..], &key2_read[..], "Different tunnels should have different keys");
            }
        }
    }

    #[test]
    fn test_max_nonce_value_constant() {
        assert_eq!(MAX_NONCE_VALUE, u64::MAX - 1000);
    }
    
    #[tokio::test]
    async fn test_invalid_peer_signature_rejected() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8015";
        
        // Generate valid credentials
        let (_, peer_identity_key, _) = generate_handshake_auth(&peer_pk);
        
        // Use invalid signature (all zeros)
        let invalid_signature = [0u8; 64];
        
        let result = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &invalid_signature,
            listen_addr
        ).await;
        
        assert!(matches!(result, Err(TunnelError::InvalidPeerIdentity)), 
            "Should reject invalid signature");
    }
    
    #[tokio::test]
    async fn test_mismatched_identity_key_rejected() {
        let local_sk = [1u8; 32];
        let peer_pk = [2u8; 32];
        let listen_addr = "127.0.0.1:8017";
        
        // Generate credentials for different key
        let different_pk = [99u8; 32];
        let (_, peer_identity_key, peer_signature) = generate_handshake_auth(&different_pk);
        
        // Try to use signature for different_pk with peer_pk
        let result = create_tunnel(
            &local_sk,
            &peer_pk,
            &peer_identity_key,
            &peer_signature,
            listen_addr
        ).await;
        
        assert!(matches!(result, Err(TunnelError::InvalidPeerIdentity)), 
            "Should reject mismatched signature");
    }
    
    #[test]
    fn test_replay_window_sequential() {
        use crate::ReplayWindow;
        
        let mut window = ReplayWindow::new();
        
        // Accept sequential nonces
        assert!(window.check_and_update(1).is_ok());
        assert!(window.check_and_update(2).is_ok());
        assert!(window.check_and_update(3).is_ok());
        
        // Reject replay of nonce 2
        assert!(matches!(window.check_and_update(2), Err(TunnelError::ReplayDetected)));
    }
    
    #[test]
    fn test_replay_window_out_of_order() {
        use crate::ReplayWindow;
        
        let mut window = ReplayWindow::new();
        
        // Accept out-of-order within window
        assert!(window.check_and_update(100).is_ok());
        assert!(window.check_and_update(99).is_ok());
        assert!(window.check_and_update(50).is_ok());
        
        // Reject replays
        assert!(matches!(window.check_and_update(100), Err(TunnelError::ReplayDetected)));
        assert!(matches!(window.check_and_update(50), Err(TunnelError::ReplayDetected)));
    }
    
    #[test]
    fn test_replay_window_old_nonces() {
        use crate::ReplayWindow;
        
        let mut window = ReplayWindow::new();
        
        // Accept nonce 3000
        assert!(window.check_and_update(3000).is_ok());
        
        // Nonce 900 is outside window (3000 - 2048 = 952), should be rejected
        assert!(matches!(window.check_and_update(900), Err(TunnelError::ReplayDetected)));
        
        // Nonce 1000 is within window (3000 - 1000 = 2000 < 2048)
        assert!(window.check_and_update(1000).is_ok());
    }
    
    #[test]
    fn test_replay_window_large_gap() {
        use crate::ReplayWindow;
        
        let mut window = ReplayWindow::new();
        
        assert!(window.check_and_update(100).is_ok());
        
        // Large gap clears window
        assert!(window.check_and_update(5000).is_ok());
        
        // Old nonce rejected
        assert!(matches!(window.check_and_update(100), Err(TunnelError::ReplayDetected)));
    }
    
    #[test]
    fn test_rate_limiter_basic() {
        use crate::RateLimiter;
        
        let mut limiter = RateLimiter::new(10, 10); // 10 pps, 10 burst
        
        // Should accept up to burst size
        for _ in 0..10 {
            assert!(limiter.check_and_consume().is_ok());
        }
        
        // Next packet should be rejected (bucket empty)
        assert!(matches!(limiter.check_and_consume(), Err(TunnelError::RateLimitExceeded)));
    }
    
    #[test]
    fn test_rate_limiter_refill() {
        use crate::RateLimiter;
        use std::thread;
        use std::time::Duration;
        
        let mut limiter = RateLimiter::new(10, 5); // 10 pps, 5 burst
        
        // Consume all tokens
        for _ in 0..5 {
            assert!(limiter.check_and_consume().is_ok());
        }
        
        // Bucket empty
        assert!(matches!(limiter.check_and_consume(), Err(TunnelError::RateLimitExceeded)));
        
        // Wait for refill (200ms = 2 tokens at 10 pps)
        thread::sleep(Duration::from_millis(200));
        
        // Should accept 2 packets now
        assert!(limiter.check_and_consume().is_ok());
        assert!(limiter.check_and_consume().is_ok());
        
        // Third should fail
        assert!(matches!(limiter.check_and_consume(), Err(TunnelError::RateLimitExceeded)));
    }
    
    #[test]
    fn test_rate_limiter_burst_then_sustained() {
        use crate::RateLimiter;
        
        let mut limiter = RateLimiter::new(100, 200); // 100 pps sustained, 200 burst
        
        // Consume burst
        for _ in 0..200 {
            assert!(limiter.check_and_consume().is_ok());
        }
        
        // Exceeded
        assert!(matches!(limiter.check_and_consume(), Err(TunnelError::RateLimitExceeded)));
    }
    
    #[test]
    fn test_buffer_pool_basic() {
        use crate::BufferPool;
        
        let pool = BufferPool::new(4);
        
        // Get buffers
        let buf1 = pool.get();
        let buf2 = pool.get();
        
        assert_eq!(buf1.capacity(), 65535);
        assert_eq!(buf2.capacity(), 65535);
        
        // Return to pool
        pool.put(buf1);
        pool.put(buf2);
        
        // Get again (should reuse)
        let buf3 = pool.get();
        assert_eq!(buf3.capacity(), 65535);
    }
    
    #[test]
    fn test_buffer_pool_reuse() {
        use crate::BufferPool;
        
        let pool = BufferPool::new(2);
        
        // Exhaust pool
        let buf1 = pool.get();
        let buf2 = pool.get();
        
        // Pool empty - should allocate new
        let buf3 = pool.get();
        assert_eq!(buf3.capacity(), 65535);
        
        // Return buffers
        pool.put(buf1);
        pool.put(buf2);
        pool.put(buf3);
        
        // Should have 2 in pool (capacity limit)
        let buf4 = pool.get();
        let buf5 = pool.get();
        assert_eq!(buf4.capacity(), 65535);
        assert_eq!(buf5.capacity(), 65535);
    }
    
    #[test]
    fn test_buffer_pool_clear_on_return() {
        use crate::BufferPool;
        
        let pool = BufferPool::new(2);
        
        let mut buf = pool.get();
        buf.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(buf.len(), 5);
        
        // Return should clear
        pool.put(buf);
        
        let buf2 = pool.get();
        assert_eq!(buf2.len(), 0); // Should be empty
    }
}


