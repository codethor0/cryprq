// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use cryprq_crypto::HybridHandshake;
use pqcrypto_mlkem::mlkem768::{decapsulate, encapsulate, PublicKey as KyberPublicKey};
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SharedSecret};
use x25519_dalek::PublicKey as X25519PublicKey;
use zeroize::Zeroize;

/// Size of a hello message: X25519 public key followed by an ML-KEM-768 public key.
pub const HELLO_LEN: usize = 32 + 1184;
/// Size of CLIENT_FINISH: one ML-KEM-768 ciphertext.
pub const CLIENT_FINISH_LEN: usize = 1088;

/// Handshake role (initiator or responder)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeRole {
    Initiator,
    #[allow(dead_code)] // Reserved for future responder implementation
    Responder,
}

/// Handshake state for performing ML-KEM + X25519 hybrid key exchange
#[allow(dead_code)] // Handshake implementation in progress
pub struct HandshakeState {
    pub role: HandshakeRole,
    pub local_handshake: HybridHandshake,
    pub peer_x25519_pk: Option<X25519PublicKey>,
    pub peer_kyber_pk: Option<KyberPublicKey>,
    pub ss_kem: Option<[u8; 32]>,
    pub ss_x: Option<[u8; 32]>,
    /// ML-KEM ciphertext produced alongside `ss_kem`; sent as CLIENT_FINISH
    kem_ciphertext: Option<Vec<u8>>,
    /// Position in the message flow; every message is accepted exactly once
    step: Step,
}

/// Progress of the key exchange. Transitions only move forward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// No peer message processed yet
    Start,
    /// Responder has answered CLIENT_HELLO and awaits CLIENT_FINISH
    AwaitingFinish,
    /// Both shared secrets are derived
    Complete,
    /// A cryptographic check failed; the state is unusable
    Failed,
}

impl Drop for HandshakeState {
    fn drop(&mut self) {
        self.wipe_secrets();
    }
}

/// X25519 agreement that rejects low-order peer keys (all-zero shared secret)
fn x25519_agree(
    local: &x25519_dalek::StaticSecret,
    peer: &X25519PublicKey,
) -> Result<[u8; 32], String> {
    let shared = local.diffie_hellman(peer);
    if !shared.was_contributory() {
        return Err("Peer X25519 public key is low-order".to_string());
    }
    Ok(*shared.as_bytes())
}

impl HandshakeState {
    fn wipe_secrets(&mut self) {
        if let Some(secret) = self.ss_kem.as_mut() {
            secret.zeroize();
        }
        if let Some(secret) = self.ss_x.as_mut() {
            secret.zeroize();
        }
        self.ss_kem = None;
        self.ss_x = None;
    }

    /// Marks the exchange as failed and discards anything derived so far.
    fn fail<T>(&mut self, reason: String) -> Result<T, String> {
        self.wipe_secrets();
        self.kem_ciphertext = None;
        self.step = Step::Failed;
        Err(reason)
    }

    fn require(&self, role: HandshakeRole, step: Step, message: &str) -> Result<(), String> {
        if self.role != role {
            return Err(format!(
                "{message} is not valid for the {:?} role",
                self.role
            ));
        }
        if self.step != step {
            return Err(format!("{message} is not valid in state {:?}", self.step));
        }
        Ok(())
    }

    /// Create new handshake state as initiator
    #[allow(dead_code)] // Handshake implementation in progress
    pub fn new_initiator() -> Self {
        Self {
            role: HandshakeRole::Initiator,
            local_handshake: HybridHandshake::new(),
            peer_x25519_pk: None,
            peer_kyber_pk: None,
            ss_kem: None,
            ss_x: None,
            kem_ciphertext: None,
            step: Step::Start,
        }
    }

    /// Create new handshake state as responder
    #[allow(dead_code)] // Handshake implementation in progress
    pub fn new_responder() -> Self {
        Self {
            role: HandshakeRole::Responder,
            local_handshake: HybridHandshake::new(),
            peer_x25519_pk: None,
            peer_kyber_pk: None,
            ss_kem: None,
            ss_x: None,
            kem_ciphertext: None,
            step: Step::Start,
        }
    }

    /// Initiator: Create CLIENT_HELLO message
    ///
    /// Returns: (message_bytes, x25519_public_key_bytes, kyber_public_key_bytes)
    #[allow(dead_code, clippy::type_complexity)] // Handshake implementation in progress
    pub fn create_client_hello(&self) -> Result<(Vec<u8>, [u8; 32], Vec<u8>), String> {
        if self.role != HandshakeRole::Initiator {
            return Err("CLIENT_HELLO can only be created by the initiator".to_string());
        }
        let x25519_pk = x25519_dalek::PublicKey::from(self.local_handshake.x25519_secret());
        let kyber_pk = self.local_handshake.kyber_public_key();

        // Encode: X25519 public key (32 bytes) + Kyber public key (1184 bytes)
        let mut msg = Vec::with_capacity(32 + 1184);
        msg.extend_from_slice(x25519_pk.as_bytes());

        // Kyber768 public key is 1184 bytes
        let kyber_pk_bytes = kyber_pk.as_bytes();
        msg.extend_from_slice(kyber_pk_bytes);

        Ok((msg, *x25519_pk.as_bytes(), kyber_pk_bytes.to_vec()))
    }

    /// Responder: Process CLIENT_HELLO and create SERVER_HELLO
    ///
    /// Returns: (server_hello_bytes, x25519_public_key_bytes, kyber_public_key_bytes)
    #[allow(dead_code, clippy::type_complexity)] // Handshake implementation in progress
    pub fn process_client_hello(
        &mut self,
        client_hello: &[u8],
    ) -> Result<(Vec<u8>, [u8; 32], Vec<u8>), String> {
        self.require(HandshakeRole::Responder, Step::Start, "CLIENT_HELLO")?;
        if client_hello.len() != HELLO_LEN {
            return Err(format!(
                "CLIENT_HELLO must be {HELLO_LEN} bytes, got {}",
                client_hello.len()
            ));
        }

        // Extract peer's public keys
        let peer_x25519_bytes: [u8; 32] = client_hello[0..32]
            .try_into()
            .map_err(|_| "Failed to extract X25519 public key")?;
        let peer_kyber_bytes = &client_hello[32..32 + 1184];

        // Parse X25519 public key
        self.peer_x25519_pk = Some(X25519PublicKey::from(peer_x25519_bytes));

        // Parse Kyber public key
        let kyber_pk_bytes: [u8; 1184] = peer_kyber_bytes
            .try_into()
            .map_err(|_| "Failed to extract Kyber public key")?;
        self.peer_kyber_pk = Some(
            <KyberPublicKey as PublicKey>::from_bytes(&kyber_pk_bytes)
                .map_err(|e| format!("Failed to parse Kyber public key: {e:?}"))?,
        );

        // Create SERVER_HELLO with our public keys
        let x25519_pk = x25519_dalek::PublicKey::from(self.local_handshake.x25519_secret());
        let kyber_pk = self.local_handshake.kyber_public_key();
        let kyber_pk_bytes = kyber_pk.as_bytes();

        let mut msg = Vec::with_capacity(32 + 1184);
        msg.extend_from_slice(x25519_pk.as_bytes());
        msg.extend_from_slice(kyber_pk_bytes);

        self.step = Step::AwaitingFinish;
        Ok((msg, *x25519_pk.as_bytes(), kyber_pk_bytes.to_vec()))
    }

    /// Initiator: Process SERVER_HELLO and derive secrets
    #[allow(dead_code)] // Handshake implementation in progress
    pub fn process_server_hello(&mut self, server_hello: &[u8]) -> Result<(), String> {
        // Accepted once: a second SERVER_HELLO would re-encapsulate and silently
        // replace the secret behind an already issued CLIENT_FINISH.
        self.require(HandshakeRole::Initiator, Step::Start, "SERVER_HELLO")?;
        if server_hello.len() != HELLO_LEN {
            return Err(format!(
                "SERVER_HELLO must be {HELLO_LEN} bytes, got {}",
                server_hello.len()
            ));
        }

        // Extract peer's public keys
        let peer_x25519_bytes: [u8; 32] = server_hello[0..32]
            .try_into()
            .map_err(|_| "Failed to extract X25519 public key")?;
        let peer_kyber_bytes = &server_hello[32..32 + 1184];

        // Parse X25519 public key
        self.peer_x25519_pk = Some(X25519PublicKey::from(peer_x25519_bytes));

        // Parse Kyber public key
        let kyber_pk_bytes: [u8; 1184] = peer_kyber_bytes
            .try_into()
            .map_err(|_| "Failed to extract Kyber public key")?;
        self.peer_kyber_pk = Some(
            KyberPublicKey::from_bytes(&kyber_pk_bytes)
                .map_err(|_| "Failed to parse Kyber public key")?,
        );

        // Derive X25519 shared secret
        let peer_pk = self
            .peer_x25519_pk
            .as_ref()
            .ok_or("Peer X25519 public key not set")?;
        match x25519_agree(self.local_handshake.x25519_secret(), peer_pk) {
            Ok(ss_x) => self.ss_x = Some(ss_x),
            Err(reason) => return self.fail(reason),
        }

        // Encapsulate once to the peer's Kyber public key. The ciphertext is kept so
        // CLIENT_FINISH carries exactly the encapsulation behind `ss_kem`.
        let peer_kyber_pk = self
            .peer_kyber_pk
            .as_ref()
            .ok_or("Peer Kyber public key not set")?;
        let (ss_kem_bytes, ciphertext) = encapsulate(peer_kyber_pk);
        self.kem_ciphertext = Some(ciphertext.as_bytes().to_vec());

        // ML-KEM shared secret is 32 bytes
        let ss_kem: [u8; 32] = ss_kem_bytes.as_bytes()[..32]
            .try_into()
            .map_err(|_| "Invalid ML-KEM shared secret length")?;
        self.ss_kem = Some(ss_kem);

        self.step = Step::Complete;
        Ok(())
    }

    /// Responder: Process CLIENT_FINISH and derive secrets
    #[allow(dead_code)] // Handshake implementation in progress
    pub fn process_client_finish(&mut self, client_finish: &[u8]) -> Result<(), String> {
        // CLIENT_FINISH contains ML-KEM ciphertext (1088 bytes for Kyber768)
        self.require(
            HandshakeRole::Responder,
            Step::AwaitingFinish,
            "CLIENT_FINISH",
        )?;
        if client_finish.len() != CLIENT_FINISH_LEN {
            return Err(format!(
                "CLIENT_FINISH must be {CLIENT_FINISH_LEN} bytes, got {}",
                client_finish.len()
            ));
        }

        // Extract ciphertext (1088 bytes for Kyber768)
        let ciphertext_bytes: [u8; 1088] = client_finish[0..1088]
            .try_into()
            .map_err(|_| "Failed to extract ML-KEM ciphertext")?;

        // Parse ciphertext
        let ciphertext = Ciphertext::from_bytes(&ciphertext_bytes)
            .map_err(|_| "Failed to parse ML-KEM ciphertext")?;

        // Decapsulate to get shared secret
        // decapsulate returns SharedSecret
        let ss_kem_bytes = decapsulate(&ciphertext, self.local_handshake.kyber_secret_key());

        // ML-KEM shared secret is 32 bytes
        let ss_kem: [u8; 32] = ss_kem_bytes.as_bytes()[..32]
            .try_into()
            .map_err(|_| "Invalid ML-KEM shared secret length")?;
        self.ss_kem = Some(ss_kem);

        // Derive X25519 shared secret (already have peer's public key from CLIENT_HELLO)
        let peer_pk = self
            .peer_x25519_pk
            .as_ref()
            .ok_or("Peer X25519 public key not set")?;
        match x25519_agree(self.local_handshake.x25519_secret(), peer_pk) {
            Ok(ss_x) => self.ss_x = Some(ss_x),
            Err(reason) => return self.fail(reason),
        }

        self.step = Step::Complete;
        Ok(())
    }

    /// Initiator: Create CLIENT_FINISH message
    ///
    /// Returns: CLIENT_FINISH message bytes (ML-KEM ciphertext)
    #[allow(dead_code)] // Handshake implementation in progress
    pub fn create_client_finish(&self) -> Result<Vec<u8>, String> {
        // A fresh encapsulation here would carry a secret the initiator never kept,
        // so the responder would decapsulate a different ss_kem.
        self.kem_ciphertext
            .clone()
            .ok_or_else(|| "SERVER_HELLO not processed; no ML-KEM ciphertext".to_string())
    }

    /// Get derived secrets (after handshake completes)
    ///
    /// Returns: (ss_kem, ss_x) or error if handshake incomplete
    #[allow(dead_code)] // Handshake implementation in progress
    pub fn get_secrets(&self) -> Result<([u8; 32], [u8; 32]), String> {
        if self.step != Step::Complete {
            return Err(format!("Handshake not complete (state {:?})", self.step));
        }
        match (self.ss_kem, self.ss_x) {
            (Some(kem), Some(x)) => Ok((kem, x)),
            (None, _) => Err("ML-KEM shared secret not derived".to_string()),
            (_, None) => Err("X25519 shared secret not derived".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x25519_dalek::EphemeralSecret;

    #[test]
    fn test_x25519_shared_secret_matches() {
        // Generate two X25519 keypairs
        let sk1 = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
        let pk1 = x25519_dalek::PublicKey::from(&sk1);

        let sk2 = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
        let pk2 = x25519_dalek::PublicKey::from(&sk2);

        // Derive shared secrets (should match)
        let ss1 = sk1.diffie_hellman(&pk2);
        let ss2 = sk2.diffie_hellman(&pk1);

        assert_eq!(
            ss1.as_bytes(),
            ss2.as_bytes(),
            "X25519 shared secrets must match"
        );
    }

    #[test]
    fn test_mlkem_round_trip() {
        use pqcrypto_mlkem::mlkem768::{decapsulate, encapsulate, keypair};

        // Generate ML-KEM keypair
        let (pk, sk) = keypair();

        // Encapsulate
        let (ss1, ct) = encapsulate(&pk);

        // Decapsulate
        let ss2 = decapsulate(&ct, &sk);

        // Shared secrets must match
        assert_eq!(
            ss1.as_bytes(),
            ss2.as_bytes(),
            "ML-KEM shared secrets must match"
        );

        // Verify sizes
        assert_eq!(
            ss1.as_bytes().len(),
            32,
            "ML-KEM shared secret must be 32 bytes"
        );
        assert_eq!(
            ct.as_bytes().len(),
            1088,
            "ML-KEM ciphertext must be 1088 bytes"
        );
    }

    #[test]
    fn test_hybrid_kdf_is_deterministic() {
        use cryprq_crypto::derive_handshake_keys;

        let dummy_kem = [0xAA; 32];
        let dummy_x = [0xBB; 32];

        let (hs_auth1, master1) = derive_handshake_keys(&dummy_kem, &dummy_x);
        let (hs_auth2, master2) = derive_handshake_keys(&dummy_kem, &dummy_x);

        assert_eq!(
            hs_auth1, hs_auth2,
            "Handshake auth keys must be deterministic"
        );
        assert_eq!(master1, master2, "Master secrets must be deterministic");
    }

    #[test]
    fn test_handshake_state_initiator() {
        let hs = HandshakeState::new_initiator();
        assert_eq!(hs.role, HandshakeRole::Initiator);
        assert!(hs.peer_x25519_pk.is_none());
        assert!(hs.peer_kyber_pk.is_none());
        assert!(hs.ss_kem.is_none());
        assert!(hs.ss_x.is_none());
    }

    #[test]
    fn test_handshake_state_responder() {
        let hs = HandshakeState::new_responder();
        assert_eq!(hs.role, HandshakeRole::Responder);
        assert!(hs.peer_x25519_pk.is_none());
        assert!(hs.peer_kyber_pk.is_none());
        assert!(hs.ss_kem.is_none());
        assert!(hs.ss_x.is_none());
    }

    #[test]
    fn test_client_hello_creation() {
        let hs = HandshakeState::new_initiator();
        let result = hs.create_client_hello();

        assert!(result.is_ok(), "CLIENT_HELLO creation should succeed");
        #[allow(clippy::disallowed_methods)] // Test code can use unwrap
        let (msg_bytes, x25519_bytes, kyber_bytes) = result.unwrap();

        assert_eq!(
            msg_bytes.len(),
            32 + 1184,
            "CLIENT_HELLO must be 32 + 1184 bytes"
        );
        assert_eq!(x25519_bytes.len(), 32, "X25519 public key must be 32 bytes");
        assert_eq!(
            kyber_bytes.len(),
            1184,
            "Kyber public key must be 1184 bytes"
        );
    }

    #[test]
    fn test_client_hello_server_hello_round_trip() {
        let mut initiator = HandshakeState::new_initiator();
        let mut responder = HandshakeState::new_responder();

        // Initiator creates CLIENT_HELLO
        let (client_hello, _, _) = initiator
            .create_client_hello()
            .expect("CLIENT_HELLO creation should succeed");

        // Responder processes CLIENT_HELLO and creates SERVER_HELLO
        let (server_hello, _, _) = responder
            .process_client_hello(&client_hello)
            .expect("SERVER_HELLO creation should succeed");

        // Initiator processes SERVER_HELLO
        initiator
            .process_server_hello(&server_hello)
            .expect("SERVER_HELLO processing should succeed");

        // Both should have derived secrets
        let (ss_kem, ss_x) = initiator
            .get_secrets()
            .expect("Initiator should have derived secrets");
        assert_ne!(ss_kem, [0u8; 32], "ML-KEM secret should not be zero");
        assert_ne!(ss_x, [0u8; 32], "X25519 secret should not be zero");
    }

    /// Runs CLIENT_HELLO -> SERVER_HELLO -> CLIENT_FINISH between two fresh peers.
    fn run_handshake() -> (HandshakeState, HandshakeState, Vec<u8>) {
        let mut initiator = HandshakeState::new_initiator();
        let mut responder = HandshakeState::new_responder();
        let (client_hello, _, _) = initiator.create_client_hello().expect("CLIENT_HELLO");
        let (server_hello, _, _) = responder
            .process_client_hello(&client_hello)
            .expect("SERVER_HELLO");
        initiator
            .process_server_hello(&server_hello)
            .expect("process SERVER_HELLO");
        let client_finish = initiator.create_client_finish().expect("CLIENT_FINISH");
        (initiator, responder, client_finish)
    }

    #[test]
    fn test_full_handshake_both_sides_derive_identical_secrets() {
        let (initiator, mut responder, client_finish) = run_handshake();
        responder
            .process_client_finish(&client_finish)
            .expect("process CLIENT_FINISH");

        let initiator_secrets = initiator.get_secrets().expect("initiator secrets");
        let responder_secrets = responder.get_secrets().expect("responder secrets");
        assert_eq!(
            initiator_secrets.0, responder_secrets.0,
            "ML-KEM secrets differ"
        );
        assert_eq!(
            initiator_secrets.1, responder_secrets.1,
            "X25519 secrets differ"
        );
        assert_eq!(
            cryprq_crypto::derive_handshake_keys(&initiator_secrets.0, &initiator_secrets.1),
            cryprq_crypto::derive_handshake_keys(&responder_secrets.0, &responder_secrets.1),
        );
    }

    #[test]
    fn test_client_finish_is_the_encapsulation_behind_ss_kem() {
        let (initiator, _, client_finish) = run_handshake();
        // Repeated calls must not re-encapsulate.
        assert_eq!(
            client_finish,
            initiator.create_client_finish().expect("handshake step")
        );
        assert_eq!(client_finish.len(), 1088);
    }

    #[test]
    fn test_independent_sessions_derive_different_secrets() {
        let (a, _, _) = run_handshake();
        let (b, _, _) = run_handshake();
        assert_ne!(
            a.get_secrets().expect("handshake step"),
            b.get_secrets().expect("handshake step")
        );
    }

    #[test]
    fn test_tampered_client_finish_breaks_kem_agreement() {
        let (initiator, mut responder, mut client_finish) = run_handshake();
        client_finish[0] ^= 0x01;
        responder
            .process_client_finish(&client_finish)
            .expect("ML-KEM uses implicit rejection, not an error");
        let (init_kem, init_x) = initiator.get_secrets().expect("handshake step");
        let (resp_kem, resp_x) = responder.get_secrets().expect("handshake step");
        assert_ne!(
            init_kem, resp_kem,
            "tampered ciphertext must not yield the same secret"
        );
        assert_eq!(init_x, resp_x);
    }

    #[test]
    fn test_kem_secret_is_bound_to_server_hello_key() {
        // Only the holder of the secret key matching the SERVER_HELLO Kyber key can
        // recover ss_kem. SERVER_HELLO is not yet authenticated, so this binding alone
        // does not prevent an active man-in-the-middle.
        let mut initiator = HandshakeState::new_initiator();
        let mut honest = HandshakeState::new_responder();
        let mut substitute = HandshakeState::new_responder();
        let (client_hello, _, _) = initiator.create_client_hello().expect("handshake step");
        honest
            .process_client_hello(&client_hello)
            .expect("handshake step");
        let (forged_hello, _, _) = substitute
            .process_client_hello(&client_hello)
            .expect("handshake step");
        initiator
            .process_server_hello(&forged_hello)
            .expect("handshake step");
        let client_finish = initiator.create_client_finish().expect("handshake step");

        honest
            .process_client_finish(&client_finish)
            .expect("handshake step");
        substitute
            .process_client_finish(&client_finish)
            .expect("handshake step");
        let init_kem = initiator.get_secrets().expect("handshake step").0;
        assert_ne!(init_kem, honest.get_secrets().expect("handshake step").0);
        assert_eq!(
            init_kem,
            substitute.get_secrets().expect("handshake step").0
        );
    }

    #[test]
    fn test_low_order_x25519_keys_are_rejected() {
        let mut initiator = HandshakeState::new_initiator();
        let mut responder = HandshakeState::new_responder();
        let (client_hello, _, _) = initiator.create_client_hello().expect("handshake step");
        let (mut server_hello, _, _) = responder
            .process_client_hello(&client_hello)
            .expect("handshake step");
        server_hello[..32].copy_from_slice(&[0u8; 32]);
        assert!(initiator.process_server_hello(&server_hello).is_err());

        let (_, mut responder, client_finish) = {
            let mut initiator = HandshakeState::new_initiator();
            let mut responder = HandshakeState::new_responder();
            let (mut client_hello, _, _) = initiator.create_client_hello().expect("handshake step");
            let (server_hello, _, _) = responder
                .process_client_hello(&client_hello)
                .expect("handshake step");
            initiator
                .process_server_hello(&server_hello)
                .expect("handshake step");
            let client_finish = initiator.create_client_finish().expect("handshake step");
            client_hello[..32].copy_from_slice(&[0u8; 32]);
            let mut low_order_responder = HandshakeState::new_responder();
            low_order_responder
                .process_client_hello(&client_hello)
                .expect("handshake step");
            (initiator, low_order_responder, client_finish)
        };
        assert!(responder.process_client_finish(&client_finish).is_err());
    }

    #[test]
    fn test_out_of_order_and_malformed_messages_are_rejected() {
        let initiator = HandshakeState::new_initiator();
        assert!(initiator.create_client_finish().is_err());

        let (_, mut responder, client_finish) = run_handshake();
        assert!(responder
            .process_client_finish(&client_finish[..1087])
            .is_err());
        let mut extended = client_finish.clone();
        extended.push(0);
        assert!(responder.process_client_finish(&extended).is_err());
    }

    #[test]
    fn test_messages_are_bound_to_role_and_accepted_once() {
        let mut initiator = HandshakeState::new_initiator();
        let mut responder = HandshakeState::new_responder();
        let (client_hello, _, _) = initiator.create_client_hello().expect("handshake step");

        // Wrong role for each message.
        assert!(responder.create_client_hello().is_err());
        assert!(initiator.process_client_hello(&client_hello).is_err());
        assert!(responder.process_server_hello(&client_hello).is_err());
        assert!(initiator
            .process_client_finish(&[0u8; CLIENT_FINISH_LEN])
            .is_err());
        // CLIENT_FINISH before CLIENT_HELLO.
        assert!(responder
            .process_client_finish(&[0u8; CLIENT_FINISH_LEN])
            .is_err());
        assert!(responder.get_secrets().is_err());

        let (server_hello, _, _) = responder
            .process_client_hello(&client_hello)
            .expect("handshake step");
        // Replayed CLIENT_HELLO.
        assert!(responder.process_client_hello(&client_hello).is_err());
        assert!(responder.get_secrets().is_err());

        initiator
            .process_server_hello(&server_hello)
            .expect("handshake step");
        let client_finish = initiator.create_client_finish().expect("handshake step");
        let secrets = initiator.get_secrets().expect("handshake step");
        // Replayed SERVER_HELLO must not re-encapsulate.
        assert!(initiator.process_server_hello(&server_hello).is_err());
        assert_eq!(
            client_finish,
            initiator.create_client_finish().expect("handshake step")
        );
        assert_eq!(secrets, initiator.get_secrets().expect("handshake step"));

        responder
            .process_client_finish(&client_finish)
            .expect("handshake step");
        // Replayed CLIENT_FINISH.
        assert!(responder.process_client_finish(&client_finish).is_err());
        assert_eq!(secrets, responder.get_secrets().expect("handshake step"));
    }

    #[test]
    fn test_hello_length_must_be_exact() {
        let initiator = HandshakeState::new_initiator();
        let (client_hello, _, _) = initiator.create_client_hello().expect("handshake step");
        let mut long = client_hello.clone();
        long.push(0);
        for bad in [&client_hello[..HELLO_LEN - 1], &long[..], &[][..]] {
            assert!(HandshakeState::new_responder()
                .process_client_hello(bad)
                .is_err());
            assert!(HandshakeState::new_initiator()
                .process_server_hello(bad)
                .is_err());
        }
        // A malformed length does not consume the single accepted message.
        let mut responder = HandshakeState::new_responder();
        assert!(responder.process_client_hello(&long).is_err());
        assert!(responder.process_client_hello(&client_hello).is_ok());
    }

    #[test]
    fn test_failed_exchange_exposes_no_secrets() {
        let mut initiator = HandshakeState::new_initiator();
        let mut responder = HandshakeState::new_responder();
        let (client_hello, _, _) = initiator.create_client_hello().expect("handshake step");
        let (mut server_hello, _, _) = responder
            .process_client_hello(&client_hello)
            .expect("handshake step");
        let honest = server_hello.clone();
        server_hello[..32].copy_from_slice(&[0u8; 32]);
        assert!(initiator.process_server_hello(&server_hello).is_err());
        assert!(initiator.get_secrets().is_err());
        assert!(initiator.create_client_finish().is_err());
        // The failure is terminal; a later honest message is not accepted.
        assert!(initiator.process_server_hello(&honest).is_err());
    }
}
