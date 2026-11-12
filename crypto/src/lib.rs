// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;

mod hybrid;
mod kdf;
pub mod mlkem;
mod ppk;
mod pqc_suite;
mod zkp;

pub use kdf::{
    confirmation_mac, derive_epoch_keys, derive_handshake_keys, derive_session_secrets,
    derive_traffic_keys, verify_confirmation_mac, SessionSecrets, TranscriptHash, LABEL_HS_AUTH,
    LABEL_IR_IV, LABEL_IR_KEY, LABEL_MASTER_SECRET, LABEL_RI_IV, LABEL_RI_KEY, SALT_HS, SALT_HS_V2,
};

#[cfg(test)]
mod kat_tests;

#[cfg(test)]
mod property_tests;

// Publicly export items needed by other crates
pub use crate::hybrid::{HybridHandshake, SharedSecret32};
pub use crate::ppk::{PPKStore, PostQuantumPSK};
pub use crate::pqc_suite::{PQCKeyExchange, PQCSignature, PQCSuite};
pub use crate::zkp::{generate_proof, verify_proof, ZkProof};
// Re-export Kyber types for use in other crates (may be used in future)
#[allow(unused_imports)]
pub use crate::mlkem::{
    keypair as kyber_keypair, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
};
