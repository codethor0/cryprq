use pqcrypto_kyber::kyber768::{
    keypair as kyber_keypair, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
};
use rand::rngs::OsRng;
use x25519_dalek::StaticSecret;

#[allow(dead_code)]
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
            x25519: StaticSecret::random_from_rng(OsRng),
            kyber_pk: pk,
            kyber_sk: sk,
        }
    }

    pub fn x25519_secret(&self) -> &StaticSecret {
        &self.x25519
    }

    pub fn kyber_public_key(&self) -> &KyberPublicKey {
        &self.kyber_pk
    }

    pub fn kyber_secret_key(&self) -> &KyberSecretKey {
        &self.kyber_sk
    }
}

impl Default for HybridHandshake {
    fn default() -> Self {
        Self::new()
    }
}
