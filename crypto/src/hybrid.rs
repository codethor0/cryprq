#![cfg_attr(not(feature = "std"), no_std)]

use blake3::Hasher;
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
use kyber768::{Keypair, PublicKey as KyberPublicKey, Ciphertext, SharedSecret as KyberSharedSecret};

pub struct HybridHandshake {
    pub x25519: StaticSecret,
    pub kyber: Keypair,
}

pub struct SharedSecret32([u8; 32]);

impl HybridHandshake {
    pub fn generate() -> Self {
        Self {
            x25519: StaticSecret::new(rand::rngs::OsRng),
            kyber: Keypair::generate(),
        }
    }

    pub fn encapsulate(&self, peer_x25519_pk: &X25519PublicKey, peer_kyber_pk: &KyberPublicKey) -> (Ciphertext, SharedSecret32) {
        let x25519_ss = self.x25519.diffie_hellman(peer_x25519_pk);
        let (ct, kyber_ss) = peer_kyber_pk.encapsulate();
        let mut hasher = Hasher::new();
        hasher.update(x25519_ss.as_bytes());
        hasher.update(kyber_ss.as_bytes());
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize().as_bytes()[..32]);
        (ct, SharedSecret32(out))
    }

    pub fn decapsulate(&self, ct: &Ciphertext, peer_x25519_pk: &X25519PublicKey) -> SharedSecret32 {
        let x25519_ss = self.x25519.diffie_hellman(peer_x25519_pk);
        let kyber_ss = self.kyber.decapsulate(ct);
        let mut hasher = Hasher::new();
        hasher.update(x25519_ss.as_bytes());
        hasher.update(kyber_ss.as_bytes());
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize().as_bytes()[..32]);
        SharedSecret32(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn hybrid_handshake_shared_secret_matches() {
        let alice = HybridHandshake::generate();
        let bob = HybridHandshake::generate();
        let alice_x_pk = X25519PublicKey::from(&alice.x25519);
        let bob_x_pk = X25519PublicKey::from(&bob.x25519);
        let alice_k_pk = &alice.kyber.public;
        let bob_k_pk = &bob.kyber.public;
        let (ct, ss_alice) = alice.encapsulate(&bob_x_pk, bob_k_pk);
        let ss_bob = bob.decapsulate(&ct, &alice_x_pk);
        assert_eq!(ss_alice.0, ss_bob.0);
    }
}
