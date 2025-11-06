#![no_std]
use rand_core::{OsRng, RngCore};

pub fn make_kyber_keys() -> ([u8; 32], [u8; 1024]) {
    let mut pk = [0u8; 32];
    let mut sk = [0u8; 1024];
    // TODO: real Kyber768 key-gen when rosenpass exposes it
    OsRng.fill_bytes(&mut pk);
    OsRng.fill_bytes(&mut sk);
    (pk, sk)
}
