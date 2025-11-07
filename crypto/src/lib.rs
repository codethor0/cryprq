#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::unwrap_used,
    clippy::expect_used
)]

extern crate alloc;

mod hybrid;

// Publicly export items needed by other crates
pub use crate::hybrid::{HybridHandshake, SharedSecret32};
pub use kyber768::kyber768::{
    keypair as kyber_keypair, 
    PublicKey as KyberPublicKey, 
    SecretKey as KyberSecretKey
};
