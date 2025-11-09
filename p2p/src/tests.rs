// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod p2p_tests {
    use crate::{get_current_pk, init_swarm, P2PError};

    #[tokio::test]
    async fn test_swarm_initialization() {
        let result = init_swarm().await;
        assert!(result.is_ok(), "Swarm initialization should succeed");
    }

    #[tokio::test]
    async fn test_get_current_pk() {
        let result = get_current_pk().await;
        assert!(
            matches!(result, Err(P2PError::NotInitialized)),
            "Uninitialized key store should return NotInitialized"
        );
    }
}
