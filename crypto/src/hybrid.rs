#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use x25519_dalek::StaticSecret;
use pqcrypto_kyber::kyber768::{
    keypair as kyber_keypair,
    PublicKey as KyberPublicKey,
    SecretKey as KyberSecretKey,
};
use rand::rngs::OsRng;

pub struct SharedSecret32([u8; 32]);

pub struct HybridHandshake {
    x25519: StaticSecret,
    kyber_pk: KyberPublicKey,
    kyber_sk: KyberSecretKey,
}

impl HybridHandshake {
    pub fn new() -> Self {
        let (pk, sk) = kyber_keypair();
        Self {
            x25519: StaticSecret::random_from_rng(&mut OsRng),
            kyber_pk: pk,
            kyber_sk: sk,
        }
    }
}

impl Default for HybridHandshake {
    fn default() -> Self {
        Self::new()
    }
}
