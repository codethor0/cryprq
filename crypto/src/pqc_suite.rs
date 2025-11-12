// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Post-Quantum Cryptography Algorithm Suite
//!
//! This module provides a unified interface for selecting and using
//! different post-quantum algorithms. Currently supports ML-KEM (Kyber768),
//! with plans for Dilithium (signatures) and SPHINCS+ (hash-based signatures).

use pqcrypto_mlkem::mlkem768::{
    keypair as kyber_keypair, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
};

/// Post-quantum key exchange algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PQCKeyExchange {
    /// ML-KEM 768 (Kyber768-compatible) - Current default
    MLKEM768,
    /// ML-KEM 1024 (higher security, larger keys)
    MLKEM1024,
    /// X25519-only (classical, not post-quantum)
    X25519Only,
}

/// Post-quantum signature algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PQCSignature {
    /// Ed25519 (classical, not post-quantum)
    Ed25519,
    /// Dilithium3 (post-quantum, planned)
    Dilithium3,
    /// SPHINCS+ (hash-based, planned)
    SPHINCSPlus,
}

/// Post-quantum cryptography suite configuration
#[derive(Debug, Clone)]
pub struct PQCSuite {
    /// Key exchange algorithm
    pub kex: PQCKeyExchange,
    /// Signature algorithm
    pub sig: PQCSignature,
}

impl PQCSuite {
    /// Default suite: ML-KEM768 + X25519 hybrid + Ed25519 signatures
    pub fn default() -> Self {
        Self {
            kex: PQCKeyExchange::MLKEM768,
            sig: PQCSignature::Ed25519,
        }
    }

    /// High security suite: ML-KEM1024 + Dilithium3 (when available)
    pub fn high_security() -> Self {
        Self {
            kex: PQCKeyExchange::MLKEM1024,
            sig: PQCSignature::Dilithium3,
        }
    }

    /// Legacy suite: X25519-only + Ed25519 (not recommended)
    pub fn legacy() -> Self {
        Self {
            kex: PQCKeyExchange::X25519Only,
            sig: PQCSignature::Ed25519,
        }
    }

    /// Check if post-quantum encryption is enabled
    pub fn is_post_quantum(&self) -> bool {
        matches!(
            self.kex,
            PQCKeyExchange::MLKEM768 | PQCKeyExchange::MLKEM1024
        )
    }

    /// Get algorithm names for display
    pub fn algorithm_names(&self) -> (&'static str, &'static str) {
        let kex_name = match self.kex {
            PQCKeyExchange::MLKEM768 => "ML-KEM 768",
            PQCKeyExchange::MLKEM1024 => "ML-KEM 1024",
            PQCKeyExchange::X25519Only => "X25519",
        };

        let sig_name = match self.sig {
            PQCSignature::Ed25519 => "Ed25519",
            PQCSignature::Dilithium3 => "Dilithium3",
            PQCSignature::SPHINCSPlus => "SPHINCS+",
        };

        (kex_name, sig_name)
    }
}

impl Default for PQCSuite {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_suite() {
        let suite = PQCSuite::default();
        assert!(suite.is_post_quantum());
        assert_eq!(suite.kex, PQCKeyExchange::MLKEM768);
    }

    #[test]
    fn test_legacy_suite() {
        let suite = PQCSuite::legacy();
        assert!(!suite.is_post_quantum());
        assert_eq!(suite.kex, PQCKeyExchange::X25519Only);
    }

    #[test]
    fn test_algorithm_names() {
        let suite = PQCSuite::default();
        let (kex, sig) = suite.algorithm_names();
        assert_eq!(kex, "ML-KEM 768");
        assert_eq!(sig, "Ed25519");
    }
}
