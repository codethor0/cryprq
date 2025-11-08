// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;

mod hybrid;

// Publicly export items needed by other crates
pub use crate::hybrid::{HybridHandshake, SharedSecret32};
pub use pqcrypto_mlkem::mlkem768::{
    keypair as kyber_keypair, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
};
