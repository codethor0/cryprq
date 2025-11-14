// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use std::fmt;

#[derive(Debug)]
pub enum TunnelError {
    LockPoisoned(String),
    EncryptionFailed,
    DecryptionFailed,
    InvalidNonce,
    NonceOverflow,
    ReplayDetected,
    RateLimitExceeded,
    InvalidPeerIdentity,
    HandshakeFailed(String),
    NetworkError(String),
    IoError(std::io::Error),
}

impl fmt::Display for TunnelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TunnelError::LockPoisoned(msg) => write!(f, "Lock poisoned: {}", msg),
            TunnelError::EncryptionFailed => write!(f, "Packet encryption failed"),
            TunnelError::DecryptionFailed => write!(f, "Packet decryption failed"),
            TunnelError::InvalidNonce => write!(f, "Invalid nonce in packet"),
            TunnelError::NonceOverflow => {
                write!(f, "Nonce counter overflow - key rotation required")
            }
            TunnelError::ReplayDetected => write!(f, "Replay attack detected - nonce already seen"),
            TunnelError::RateLimitExceeded => write!(f, "Rate limit exceeded - too many packets"),
            TunnelError::InvalidPeerIdentity => write!(f, "Peer identity verification failed"),
            TunnelError::HandshakeFailed(msg) => write!(f, "Handshake failed: {}", msg),
            TunnelError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TunnelError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for TunnelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TunnelError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TunnelError {
    fn from(err: std::io::Error) -> Self {
        TunnelError::IoError(err)
    }
}
