// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;

mod hybrid;
mod ppk;
mod pqc_suite;
mod zkp;

#[cfg(test)]
mod kat_tests;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod property_expanded;

#[cfg(test)]
mod zeroization_tests;

#[cfg(feature = "kat")]
pub mod kat_loader;

// Publicly export items needed by other crates
pub use crate::hybrid::{HybridHandshake, SharedSecret32};
pub use crate::ppk::{PPKStore, PostQuantumPSK};
pub use crate::pqc_suite::{PQCKeyExchange, PQCSignature, PQCSuite};
pub use crate::zkp::{generate_proof, verify_proof, ZkProof};
// Re-export Kyber types for use in other crates (may be used in future)
#[allow(unused_imports)]
pub use pqcrypto_mlkem::mlkem768::{
    keypair as kyber_keypair, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
};
