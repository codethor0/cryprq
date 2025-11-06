// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024 Thorsten Behrens
use std::fmt;

#[derive(Debug)]
pub enum P2PError {
    LockPoisoned(String),
    InvalidPeerId,
    DialFailed(String),
}

impl fmt::Display for P2PError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            P2PError::LockPoisoned(msg) => write!(f, "Lock poisoned: {}", msg),
            P2PError::InvalidPeerId => write!(f, "Invalid peer ID format"),
            P2PError::DialFailed(msg) => write!(f, "Failed to dial peer: {}", msg),
        }
    }
}

impl std::error::Error for P2PError {}

impl From<P2PError> for std::io::Error {
    fn from(err: P2PError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
    }
}
