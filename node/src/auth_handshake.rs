// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! Authenticated hybrid handshake, wire version 2.
//!
//! Wraps the ML-KEM-768 + X25519 exchange of the `handshake` module in a mutually
//! authenticated, transcript-bound flow between two pinned Ed25519 identities:
//!
//! ```text
//! Initiator                                             Responder
//!   INIT    suite, random_i, id_i, share_i          ->
//!           <-  suite, random_r, id_r, share_r, sig_r   RESPONSE
//!   FINISH  kem_ct, sig_i, confirm_i                ->
//!           <-                           confirm_r      CONFIRM
//! ```
//!
//! * Each side signs the running transcript hash, which covers the protocol
//!   version, the suite, both identities and every message so far. Signatures
//!   are verified against the locally pinned key, never against a key taken
//!   on trust from the message.
//! * Session secrets are derived from both shared secrets and the transcript
//!   hash, and each side proves possession with a key-confirmation MAC before
//!   the session counts as established.
//! * The state machines do no I/O. A message that fails a check made before
//!   any state is committed is rejected without disturbing the handshake, so
//!   forged datagrams cannot abort it; failures after that point are terminal.
//!
//! See `docs/HANDSHAKE_V2.md` for the wire format.

use crate::handshake::{HandshakeRole, HandshakeState, CLIENT_FINISH_LEN, HELLO_LEN};
use crate::identity::{NodeIdentity, PeerPublicKey, TrustedPeers};
use cryprq_crypto::{
    confirmation_mac, derive_session_secrets, verify_confirmation_mac, SessionSecrets,
    TranscriptHash,
};
use rand_core::{OsRng, RngCore};
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

/// First four bytes of every handshake message. A record starts with the record
/// protocol version (`0x01`), so the two can never be confused.
pub const HANDSHAKE_MAGIC: [u8; 4] = *b"CRQH";
/// Handshake wire version implemented here. No other version is accepted.
pub const HANDSHAKE_VERSION: u8 = 2;
/// ML-KEM-768 + X25519, Ed25519 signatures, HKDF/HMAC-SHA256,
/// ChaCha20-Poly1305 records. The only supported suite.
pub const SUITE_MLKEM768_X25519_ED25519: u16 = 0x0001;

/// Message type: initiator's first message.
pub const MSG_INIT: u8 = cryprq_core::MSG_TYPE_CLIENT_HELLO;
/// Message type: responder's signed reply.
pub const MSG_RESPONSE: u8 = cryprq_core::MSG_TYPE_SERVER_HELLO;
/// Message type: initiator's ciphertext, signature and key confirmation.
pub const MSG_FINISH: u8 = cryprq_core::MSG_TYPE_CLIENT_FINISH;
/// Message type: responder's key confirmation.
pub const MSG_CONFIRM: u8 = 0x23;

/// Bytes before the body: magic, version, type, 16-bit body length.
pub const HEADER_LEN: usize = 8;
const SIG_LEN: usize = 64;
const MAC_LEN: usize = 32;
const INIT_BODY_LEN: usize = 2 + 32 + 32 + HELLO_LEN;
const RESPONSE_BODY_LEN: usize = 2 + 32 + 32 + HELLO_LEN + SIG_LEN;
const FINISH_BODY_LEN: usize = CLIENT_FINISH_LEN + SIG_LEN + MAC_LEN;
const CONFIRM_BODY_LEN: usize = MAC_LEN;
/// Largest handshake message; anything longer is rejected unread.
pub const MAX_MESSAGE_LEN: usize = HEADER_LEN + RESPONSE_BODY_LEN;

const TRANSCRIPT_DOMAIN: &[u8] = b"cryprq authenticated hybrid handshake";
const SIG_CONTEXT_RESPONDER: &[u8] = b"cryprq handshake v2 responder signature";
const SIG_CONTEXT_INITIATOR: &[u8] = b"cryprq handshake v2 initiator signature";
const CONFIRM_LABEL_INITIATOR: &[u8] = b"initiator key confirmation";
const CONFIRM_LABEL_RESPONDER: &[u8] = b"responder key confirmation";

/// Why a handshake message was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    /// Framing or length is wrong.
    Malformed(&'static str),
    /// The message declares a handshake version other than 2.
    UnsupportedVersion(u8),
    /// The message proposes a suite this implementation does not support.
    UnsupportedSuite(u16),
    /// A valid message type that is not expected in the current state.
    UnexpectedMessage,
    /// The peer's identity is not pinned.
    UntrustedPeer,
    /// The peer is not the identity this handshake was started with.
    IdentityMismatch,
    /// The transcript signature does not verify under the pinned key.
    BadSignature,
    /// The peer's key-confirmation MAC is wrong.
    BadKeyConfirmation,
    /// The hybrid key exchange failed (for example a low-order X25519 share).
    KeyExchange(String),
    /// The handshake already failed or completed.
    Finished,
}

impl HandshakeError {
    /// Whether the handshake is unusable after this error. Non-fatal errors
    /// leave the state untouched and the caller should keep waiting.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::BadKeyConfirmation | Self::KeyExchange(_) | Self::Finished
        )
    }
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(why) => write!(f, "malformed handshake message: {why}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported handshake version {v}"),
            Self::UnsupportedSuite(s) => write!(f, "unsupported suite {s:#06x}"),
            Self::UnexpectedMessage => write!(f, "unexpected handshake message"),
            Self::UntrustedPeer => write!(f, "peer identity is not pinned"),
            Self::IdentityMismatch => write!(f, "peer is not the expected identity"),
            Self::BadSignature => write!(f, "handshake signature verification failed"),
            Self::BadKeyConfirmation => write!(f, "key confirmation failed"),
            Self::KeyExchange(why) => write!(f, "key exchange failed: {why}"),
            Self::Finished => write!(f, "handshake is no longer active"),
        }
    }
}

impl std::error::Error for HandshakeError {}

/// Result of a completed handshake.
pub struct SessionKeys {
    /// Role this side played; selects the traffic-key direction.
    pub role: HandshakeRole,
    /// The authenticated, pinned identity of the peer.
    pub peer: PeerPublicKey,
    /// Public identifier unique to this session.
    pub session_id: [u8; 16],
    /// Transcript-bound master secret for epoch traffic keys.
    pub master_secret: Zeroizing<[u8; 32]>,
}

impl fmt::Debug for SessionKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionKeys")
            .field("role", &self.role)
            .field("peer", &self.peer)
            .field("session_id", &hex::encode(self.session_id))
            .finish_non_exhaustive()
    }
}

/// Returns the message type if `datagram` carries the handshake magic.
pub fn message_type(datagram: &[u8]) -> Option<u8> {
    (datagram.len() >= HEADER_LEN && datagram[..4] == HANDSHAKE_MAGIC).then(|| datagram[5])
}

fn frame(msg_type: u8, body_len: usize) -> Vec<u8> {
    let mut msg = Vec::with_capacity(HEADER_LEN + body_len);
    msg.extend_from_slice(&HANDSHAKE_MAGIC);
    msg.push(HANDSHAKE_VERSION);
    msg.push(msg_type);
    msg.extend_from_slice(&(body_len as u16).to_be_bytes());
    msg
}

/// Validates the frame and returns the body. The length must match both the
/// header field and the fixed size for `expected_type` exactly.
fn parse(msg: &[u8], expected_type: u8, body_len: usize) -> Result<&[u8], HandshakeError> {
    if msg.len() < HEADER_LEN {
        return Err(HandshakeError::Malformed("shorter than the header"));
    }
    if msg.len() > MAX_MESSAGE_LEN {
        return Err(HandshakeError::Malformed("oversized message"));
    }
    if msg[..4] != HANDSHAKE_MAGIC {
        return Err(HandshakeError::Malformed("bad magic"));
    }
    if msg[4] != HANDSHAKE_VERSION {
        return Err(HandshakeError::UnsupportedVersion(msg[4]));
    }
    let msg_type = msg[5];
    if !matches!(msg_type, MSG_INIT | MSG_RESPONSE | MSG_FINISH | MSG_CONFIRM) {
        return Err(HandshakeError::Malformed("unknown message type"));
    }
    if msg_type != expected_type {
        return Err(HandshakeError::UnexpectedMessage);
    }
    let declared = u16::from_be_bytes([msg[6], msg[7]]) as usize;
    let body = &msg[HEADER_LEN..];
    if declared != body_len || body.len() != body_len {
        return Err(HandshakeError::Malformed("wrong length for message type"));
    }
    Ok(body)
}

fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(bytes);
    out
}

fn check_suite(body: &[u8]) -> Result<(), HandshakeError> {
    let suite = u16::from_be_bytes([body[0], body[1]]);
    if suite != SUITE_MLKEM768_X25519_ED25519 {
        return Err(HandshakeError::UnsupportedSuite(suite));
    }
    Ok(())
}

/// Transcript up to and including the unsigned part of RESPONSE.
fn transcript_through_response(
    initiator: &PeerPublicKey,
    responder: &PeerPublicKey,
    init_msg: &[u8],
    response_unsigned: &[u8],
) -> TranscriptHash {
    let mut transcript = TranscriptHash::new(TRANSCRIPT_DOMAIN);
    transcript.absorb(b"version", &[HANDSHAKE_VERSION]);
    transcript.absorb(b"suite", &SUITE_MLKEM768_X25519_ED25519.to_be_bytes());
    transcript.absorb(b"initiator identity", initiator.as_bytes());
    transcript.absorb(b"responder identity", responder.as_bytes());
    transcript.absorb(b"init", init_msg);
    transcript.absorb(b"response", response_unsigned);
    transcript
}

fn signed_message(context: &[u8], transcript_hash: &[u8; 32]) -> Vec<u8> {
    let mut message = Vec::with_capacity(context.len() + 32);
    message.extend_from_slice(context);
    message.extend_from_slice(transcript_hash);
    message
}

fn derive(
    kx: &HandshakeState,
    transcript_hash: &[u8; 32],
) -> Result<SessionSecrets, HandshakeError> {
    let (mut ss_kem, mut ss_x) = kx.get_secrets().map_err(HandshakeError::KeyExchange)?;
    let secrets = derive_session_secrets(&ss_kem, &ss_x, transcript_hash);
    ss_kem.zeroize();
    ss_x.zeroize();
    Ok(secrets)
}

fn session_keys(role: HandshakeRole, peer: PeerPublicKey, secrets: &SessionSecrets) -> SessionKeys {
    SessionKeys {
        role,
        peer,
        session_id: secrets.session_id,
        master_secret: Zeroizing::new(secrets.master_secret),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitiatorStep {
    AwaitingResponse,
    AwaitingConfirm,
    Established,
    Failed,
}

/// Initiator side of the authenticated handshake.
pub struct Initiator<'a> {
    identity: &'a NodeIdentity,
    expected_peer: PeerPublicKey,
    kx: HandshakeState,
    init_msg: Vec<u8>,
    step: InitiatorStep,
    /// Set once FINISH has been produced.
    pending: Option<(SessionSecrets, [u8; 32], Vec<u8>)>,
}

impl<'a> Initiator<'a> {
    /// Starts a handshake towards `expected_peer`, which must be pinned in
    /// `trusted` and must not be this node itself.
    pub fn new(
        identity: &'a NodeIdentity,
        trusted: &TrustedPeers,
        expected_peer: &PeerPublicKey,
    ) -> Result<Self, HandshakeError> {
        if !trusted.contains(expected_peer) {
            return Err(HandshakeError::UntrustedPeer);
        }
        if *expected_peer == identity.public_key() {
            return Err(HandshakeError::IdentityMismatch);
        }
        let kx = HandshakeState::new_initiator();
        let (share, _, _) = kx
            .create_client_hello()
            .map_err(HandshakeError::KeyExchange)?;
        let mut random = [0u8; 32];
        OsRng.fill_bytes(&mut random);

        let mut init_msg = frame(MSG_INIT, INIT_BODY_LEN);
        init_msg.extend_from_slice(&SUITE_MLKEM768_X25519_ED25519.to_be_bytes());
        init_msg.extend_from_slice(&random);
        init_msg.extend_from_slice(identity.public_key().as_bytes());
        init_msg.extend_from_slice(&share);
        Ok(Self {
            identity,
            expected_peer: *expected_peer,
            kx,
            init_msg,
            step: InitiatorStep::AwaitingResponse,
            pending: None,
        })
    }

    /// The INIT message. Stable across calls, so it can be retransmitted.
    pub fn init_message(&self) -> &[u8] {
        &self.init_msg
    }

    /// The FINISH message once RESPONSE was accepted, for retransmission.
    pub fn finish_message(&self) -> Option<&[u8]> {
        self.pending
            .as_ref()
            .map(|(_, _, finish)| finish.as_slice())
    }

    /// Processes RESPONSE and returns FINISH.
    pub fn handle_response(&mut self, msg: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        match self.step {
            InitiatorStep::AwaitingResponse => {}
            InitiatorStep::AwaitingConfirm => return Err(HandshakeError::UnexpectedMessage),
            InitiatorStep::Established | InitiatorStep::Failed => {
                return Err(HandshakeError::Finished)
            }
        }
        let body = parse(msg, MSG_RESPONSE, RESPONSE_BODY_LEN)?;
        check_suite(body)?;
        let responder_id: [u8; 32] = array(&body[34..66]);
        if responder_id != *self.expected_peer.as_bytes() {
            return Err(HandshakeError::IdentityMismatch);
        }
        let share = &body[66..66 + HELLO_LEN];
        let signature: [u8; SIG_LEN] = array(&body[66 + HELLO_LEN..]);

        // Authenticate against the pinned key before touching any state.
        let unsigned = &msg[..msg.len() - SIG_LEN];
        let mut transcript = transcript_through_response(
            &self.identity.public_key(),
            &self.expected_peer,
            &self.init_msg,
            unsigned,
        );
        if !self.expected_peer.verify(
            &signed_message(SIG_CONTEXT_RESPONDER, &transcript.current()),
            &signature,
        ) {
            return Err(HandshakeError::BadSignature);
        }

        // The peer is authentic from here on, so any failure is terminal.
        if let Err(reason) = self.kx.process_server_hello(share) {
            self.step = InitiatorStep::Failed;
            return Err(HandshakeError::KeyExchange(reason));
        }
        let kem_ct = match self.kx.create_client_finish() {
            Ok(ct) => ct,
            Err(reason) => {
                self.step = InitiatorStep::Failed;
                return Err(HandshakeError::KeyExchange(reason));
            }
        };

        transcript.absorb(b"responder signature", &signature);
        transcript.absorb(b"kem ciphertext", &kem_ct);
        let my_signature = self.identity.sign(&signed_message(
            SIG_CONTEXT_INITIATOR,
            &transcript.current(),
        ));
        transcript.absorb(b"initiator signature", &my_signature);
        let key_hash = transcript.current();
        let secrets = match derive(&self.kx, &key_hash) {
            Ok(secrets) => secrets,
            Err(err) => {
                self.step = InitiatorStep::Failed;
                return Err(err);
            }
        };
        let confirm = confirmation_mac(
            &secrets.confirm_key_initiator,
            CONFIRM_LABEL_INITIATOR,
            &key_hash,
        );
        transcript.absorb(b"initiator confirmation", &confirm);

        let mut finish = frame(MSG_FINISH, FINISH_BODY_LEN);
        finish.extend_from_slice(&kem_ct);
        finish.extend_from_slice(&my_signature);
        finish.extend_from_slice(&confirm);
        self.pending = Some((secrets, transcript.current(), finish.clone()));
        self.step = InitiatorStep::AwaitingConfirm;
        Ok(finish)
    }

    /// Processes CONFIRM. Only after this returns `Ok` is the session
    /// established on the initiator.
    pub fn handle_confirm(&mut self, msg: &[u8]) -> Result<SessionKeys, HandshakeError> {
        match self.step {
            InitiatorStep::AwaitingConfirm => {}
            InitiatorStep::AwaitingResponse => return Err(HandshakeError::UnexpectedMessage),
            InitiatorStep::Established | InitiatorStep::Failed => {
                return Err(HandshakeError::Finished)
            }
        }
        let body = parse(msg, MSG_CONFIRM, CONFIRM_BODY_LEN)?;
        let Some((secrets, confirm_hash, _)) = self.pending.as_ref() else {
            return Err(HandshakeError::Finished);
        };
        // A wrong MAC is ignored rather than fatal: CONFIRM is 40 unauthenticated
        // bytes, so an off-path forgery must not be able to kill the handshake.
        if !verify_confirmation_mac(
            &secrets.confirm_key_responder,
            CONFIRM_LABEL_RESPONDER,
            confirm_hash,
            body,
        ) {
            return Err(HandshakeError::BadSignature);
        }
        let keys = session_keys(HandshakeRole::Initiator, self.expected_peer, secrets);
        self.pending = None;
        self.step = InitiatorStep::Established;
        Ok(keys)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponderStep {
    AwaitingInit,
    AwaitingFinish,
    Established,
    Failed,
}

/// Responder side of the authenticated handshake.
pub struct Responder<'a> {
    identity: &'a NodeIdentity,
    kx: HandshakeState,
    step: ResponderStep,
    /// Peer identity and transcript, set once INIT was accepted.
    pending: Option<(PeerPublicKey, TranscriptHash, Vec<u8>)>,
}

impl<'a> Responder<'a> {
    /// Creates a responder for one handshake attempt.
    pub fn new(identity: &'a NodeIdentity) -> Self {
        Self {
            identity,
            kx: HandshakeState::new_responder(),
            step: ResponderStep::AwaitingInit,
            pending: None,
        }
    }

    /// The pinned identity claimed in INIT, before it has been authenticated.
    pub fn claimed_peer(&self) -> Option<PeerPublicKey> {
        self.pending.as_ref().map(|(peer, _, _)| *peer)
    }

    /// The RESPONSE message once INIT was accepted, for retransmission.
    pub fn response_message(&self) -> Option<&[u8]> {
        self.pending.as_ref().map(|(_, _, msg)| msg.as_slice())
    }

    /// Processes INIT and returns the signed RESPONSE. The claimed initiator
    /// identity must be pinned in `trusted`; it is authenticated by FINISH.
    pub fn handle_init(
        &mut self,
        msg: &[u8],
        trusted: &TrustedPeers,
    ) -> Result<Vec<u8>, HandshakeError> {
        match self.step {
            ResponderStep::AwaitingInit => {}
            ResponderStep::AwaitingFinish => return Err(HandshakeError::UnexpectedMessage),
            ResponderStep::Established | ResponderStep::Failed => {
                return Err(HandshakeError::Finished)
            }
        }
        let body = parse(msg, MSG_INIT, INIT_BODY_LEN)?;
        check_suite(body)?;
        let initiator = PeerPublicKey::from_bytes(&array(&body[34..66]))
            .map_err(|_| HandshakeError::UntrustedPeer)?;
        if !trusted.contains(&initiator) {
            return Err(HandshakeError::UntrustedPeer);
        }
        if initiator == self.identity.public_key() {
            return Err(HandshakeError::IdentityMismatch);
        }
        let (share, _, _) = match self.kx.process_client_hello(&body[66..]) {
            Ok(share) => share,
            Err(reason) => {
                self.step = ResponderStep::Failed;
                return Err(HandshakeError::KeyExchange(reason));
            }
        };
        let mut random = [0u8; 32];
        OsRng.fill_bytes(&mut random);

        let mut response = frame(MSG_RESPONSE, RESPONSE_BODY_LEN);
        response.extend_from_slice(&SUITE_MLKEM768_X25519_ED25519.to_be_bytes());
        response.extend_from_slice(&random);
        response.extend_from_slice(self.identity.public_key().as_bytes());
        response.extend_from_slice(&share);

        let mut transcript =
            transcript_through_response(&initiator, &self.identity.public_key(), msg, &response);
        let signature = self.identity.sign(&signed_message(
            SIG_CONTEXT_RESPONDER,
            &transcript.current(),
        ));
        response.extend_from_slice(&signature);
        transcript.absorb(b"responder signature", &signature);

        self.pending = Some((initiator, transcript, response.clone()));
        self.step = ResponderStep::AwaitingFinish;
        Ok(response)
    }

    /// Processes FINISH. On success returns CONFIRM and the session keys; the
    /// initiator is then authenticated and has proven possession of the keys.
    pub fn handle_finish(&mut self, msg: &[u8]) -> Result<(Vec<u8>, SessionKeys), HandshakeError> {
        match self.step {
            ResponderStep::AwaitingFinish => {}
            ResponderStep::AwaitingInit => return Err(HandshakeError::UnexpectedMessage),
            ResponderStep::Established | ResponderStep::Failed => {
                return Err(HandshakeError::Finished)
            }
        }
        let body = parse(msg, MSG_FINISH, FINISH_BODY_LEN)?;
        let kem_ct = &body[..CLIENT_FINISH_LEN];
        let signature: [u8; SIG_LEN] = array(&body[CLIENT_FINISH_LEN..CLIENT_FINISH_LEN + SIG_LEN]);
        let confirm = &body[CLIENT_FINISH_LEN + SIG_LEN..];
        let Some((initiator, base_transcript, _)) = self.pending.as_ref() else {
            return Err(HandshakeError::Finished);
        };
        let initiator = *initiator;

        // Authenticate against the pinned key before touching any state.
        let mut transcript = base_transcript.clone();
        transcript.absorb(b"kem ciphertext", kem_ct);
        if !initiator.verify(
            &signed_message(SIG_CONTEXT_INITIATOR, &transcript.current()),
            &signature,
        ) {
            return Err(HandshakeError::BadSignature);
        }
        transcript.absorb(b"initiator signature", &signature);
        let key_hash = transcript.current();

        // The peer is authentic from here on, so any failure is terminal.
        if let Err(reason) = self.kx.process_client_finish(kem_ct) {
            self.fail();
            return Err(HandshakeError::KeyExchange(reason));
        }
        let secrets = match derive(&self.kx, &key_hash) {
            Ok(secrets) => secrets,
            Err(err) => {
                self.fail();
                return Err(err);
            }
        };
        if !verify_confirmation_mac(
            &secrets.confirm_key_initiator,
            CONFIRM_LABEL_INITIATOR,
            &key_hash,
            confirm,
        ) {
            self.fail();
            return Err(HandshakeError::BadKeyConfirmation);
        }
        transcript.absorb(b"initiator confirmation", confirm);
        let my_confirm = confirmation_mac(
            &secrets.confirm_key_responder,
            CONFIRM_LABEL_RESPONDER,
            &transcript.current(),
        );
        let mut confirm_msg = frame(MSG_CONFIRM, CONFIRM_BODY_LEN);
        confirm_msg.extend_from_slice(&my_confirm);

        let keys = session_keys(HandshakeRole::Responder, initiator, &secrets);
        self.pending = None;
        self.step = ResponderStep::Established;
        Ok((confirm_msg, keys))
    }

    fn fail(&mut self) {
        self.pending = None;
        self.step = ResponderStep::Failed;
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::identity::test_support::ephemeral_identity;

    struct Pair {
        alice: NodeIdentity,
        bob: NodeIdentity,
        alice_trust: TrustedPeers,
        bob_trust: TrustedPeers,
    }

    fn pair() -> Pair {
        let alice = ephemeral_identity();
        let bob = ephemeral_identity();
        let alice_trust = TrustedPeers::from_keys([bob.public_key()]);
        let bob_trust = TrustedPeers::from_keys([alice.public_key()]);
        Pair {
            alice,
            bob,
            alice_trust,
            bob_trust,
        }
    }

    /// Runs a full handshake and returns every message with both key sets.
    fn complete(p: &Pair) -> ([Vec<u8>; 4], SessionKeys, SessionKeys) {
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let init = initiator.init_message().to_vec();
        let response = responder.handle_init(&init, &p.bob_trust).expect("init");
        let finish = initiator.handle_response(&response).expect("response");
        let (confirm, responder_keys) = responder.handle_finish(&finish).expect("finish");
        let initiator_keys = initiator.handle_confirm(&confirm).expect("confirm");
        (
            [init, response, finish, confirm],
            initiator_keys,
            responder_keys,
        )
    }

    #[test]
    fn pinned_peers_agree_on_transcript_bound_keys() {
        let p = pair();
        let (msgs, initiator_keys, responder_keys) = complete(&p);
        assert_eq!(*initiator_keys.master_secret, *responder_keys.master_secret);
        assert_eq!(initiator_keys.session_id, responder_keys.session_id);
        assert_ne!(*initiator_keys.master_secret, [0u8; 32]);
        assert_eq!(initiator_keys.role, HandshakeRole::Initiator);
        assert_eq!(responder_keys.role, HandshakeRole::Responder);
        assert_eq!(initiator_keys.peer, p.bob.public_key());
        assert_eq!(responder_keys.peer, p.alice.public_key());

        assert_eq!(msgs[0].len(), HEADER_LEN + INIT_BODY_LEN);
        assert_eq!(msgs[1].len(), MAX_MESSAGE_LEN);
        assert_eq!(msgs[2].len(), HEADER_LEN + FINISH_BODY_LEN);
        assert_eq!(msgs[3].len(), HEADER_LEN + CONFIRM_BODY_LEN);
        // Every handshake message fits a 1500-byte MTU over IPv6/UDP.
        const _: () = assert!(MAX_MESSAGE_LEN <= 1500 - 48);
    }

    #[test]
    fn master_secret_is_not_the_unauthenticated_v1_secret() {
        // The traffic secret must depend on the transcript, not only on the two
        // raw shared secrets as the v1 key schedule does.
        let p = pair();
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        let finish = initiator.handle_response(&response).expect("response");
        let (ss_kem, ss_x) = initiator.kx.get_secrets().expect("secrets");
        let (_, v1_master) = cryprq_crypto::derive_handshake_keys(&ss_kem, &ss_x);
        let (_, keys) = responder.handle_finish(&finish).expect("finish");
        assert_ne!(*keys.master_secret, v1_master);
    }

    #[test]
    fn sessions_between_the_same_peers_have_independent_keys() {
        let p = pair();
        let (_, first, _) = complete(&p);
        let (_, second, _) = complete(&p);
        assert_ne!(*first.master_secret, *second.master_secret);
        assert_ne!(first.session_id, second.session_id);
    }

    #[test]
    fn initiator_refuses_an_unpinned_or_self_target() {
        let p = pair();
        let stranger = ephemeral_identity();
        assert_eq!(
            Initiator::new(&p.alice, &p.alice_trust, &stranger.public_key()).err(),
            Some(HandshakeError::UntrustedPeer)
        );
        // An empty trust set authorises nobody, including the intended peer.
        assert_eq!(
            Initiator::new(&p.alice, &TrustedPeers::new(), &p.bob.public_key()).err(),
            Some(HandshakeError::UntrustedPeer)
        );
        let self_trust = TrustedPeers::from_keys([p.alice.public_key()]);
        assert_eq!(
            Initiator::new(&p.alice, &self_trust, &p.alice.public_key()).err(),
            Some(HandshakeError::IdentityMismatch)
        );
    }

    #[test]
    fn responder_rejects_unpinned_initiators() {
        let p = pair();
        let stranger = ephemeral_identity();
        let stranger_trust = TrustedPeers::from_keys([p.bob.public_key()]);
        let initiator =
            Initiator::new(&stranger, &stranger_trust, &p.bob.public_key()).expect("initiator");

        // Bob pins only Alice.
        let mut responder = Responder::new(&p.bob);
        assert_eq!(
            responder.handle_init(initiator.init_message(), &p.bob_trust),
            Err(HandshakeError::UntrustedPeer)
        );
        // Missing trust configuration denies everyone.
        assert_eq!(
            responder.handle_init(initiator.init_message(), &TrustedPeers::new()),
            Err(HandshakeError::UntrustedPeer)
        );
        // The rejection consumed nothing: a pinned peer can still connect.
        let alice = Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("alice");
        assert!(responder
            .handle_init(alice.init_message(), &p.bob_trust)
            .is_ok());
    }

    #[test]
    fn impersonating_a_pinned_initiator_fails_at_finish() {
        // Mallory claims Alice's public identity but signs with her own key.
        let p = pair();
        let mallory = ephemeral_identity();
        let mallory_trust = TrustedPeers::from_keys([p.bob.public_key()]);
        let mut attacker =
            Initiator::new(&mallory, &mallory_trust, &p.bob.public_key()).expect("attacker");
        let mut forged_init = attacker.init_message().to_vec();
        forged_init[HEADER_LEN + 34..HEADER_LEN + 66]
            .copy_from_slice(p.alice.public_key().as_bytes());
        attacker.init_msg = forged_init.clone();

        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(&forged_init, &p.bob_trust)
            .expect("claimed identity is pinned, so RESPONSE is sent");
        // Mallory's transcript differs from Bob's, so she cannot even validate
        // Bob's signature; force her through by forging FINISH directly.
        assert_eq!(
            attacker.handle_response(&response),
            Err(HandshakeError::BadSignature)
        );
        let mut finish = frame(MSG_FINISH, FINISH_BODY_LEN);
        finish.extend_from_slice(&[0x5a; CLIENT_FINISH_LEN]);
        finish.extend_from_slice(&mallory.sign(b"anything"));
        finish.extend_from_slice(&[0u8; MAC_LEN]);
        assert_eq!(
            responder.handle_finish(&finish).err(),
            Some(HandshakeError::BadSignature)
        );
    }

    #[test]
    fn responder_with_substituted_identity_key_is_rejected() {
        // Mallory answers in Bob's place with her own valid identity and a
        // correctly computed signature under her own key.
        let p = pair();
        let mallory = ephemeral_identity();
        let mallory_trust = TrustedPeers::from_keys([p.alice.public_key()]);
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut imposter = Responder::new(&mallory);
        let response = imposter
            .handle_init(initiator.init_message(), &mallory_trust)
            .expect("imposter response");
        assert_eq!(
            initiator.handle_response(&response),
            Err(HandshakeError::IdentityMismatch)
        );

        // Relabelling the message with Bob's identity does not help either: the
        // signature is checked against the pinned key, not the transmitted one.
        let mut relabelled = response.clone();
        relabelled[HEADER_LEN + 34..HEADER_LEN + 66].copy_from_slice(p.bob.public_key().as_bytes());
        assert_eq!(
            initiator.handle_response(&relabelled),
            Err(HandshakeError::BadSignature)
        );

        // Neither attempt disturbed the handshake; the real Bob still completes.
        let mut responder = Responder::new(&p.bob);
        let genuine = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        let finish = initiator.handle_response(&genuine).expect("response");
        let (confirm, _) = responder.handle_finish(&finish).expect("finish");
        assert!(initiator.handle_confirm(&confirm).is_ok());
    }

    #[test]
    fn man_in_the_middle_key_share_substitution_is_detected() {
        // An active attacker replaces the responder's key share with its own.
        let p = pair();
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let mut response = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        let attacker_kx = HandshakeState::new_initiator();
        let (attacker_share, _, _) = attacker_kx.create_client_hello().expect("share");
        response[HEADER_LEN + 66..HEADER_LEN + 66 + HELLO_LEN].copy_from_slice(&attacker_share);
        assert_eq!(
            initiator.handle_response(&response),
            Err(HandshakeError::BadSignature)
        );

        // And in the other direction: the initiator's share in INIT.
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut tampered_init = initiator.init_message().to_vec();
        tampered_init[HEADER_LEN + 66..].copy_from_slice(&attacker_share);
        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(&tampered_init, &p.bob_trust)
            .expect("init");
        assert_eq!(
            initiator.handle_response(&response),
            Err(HandshakeError::BadSignature)
        );
    }

    #[test]
    fn every_single_bit_flip_in_any_message_is_rejected() {
        let p = pair();
        // Sample positions across every field of every message.
        for target in 0..4usize {
            let (reference, _, _) = complete(&p);
            let len = reference[target].len();
            let step = (len / 97).max(1);
            for position in (0..len).step_by(step).chain([len - 1]) {
                let mut initiator = Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key())
                    .expect("initiator");
                let mut responder = Responder::new(&p.bob);
                let flip = |index: usize, mut msg: Vec<u8>| {
                    if index == target {
                        msg[position] ^= 0x01;
                    }
                    msg
                };
                let outcome = (|| -> Result<(SessionKeys, SessionKeys), HandshakeError> {
                    let init = flip(0, initiator.init_message().to_vec());
                    let response = flip(1, responder.handle_init(&init, &p.bob_trust)?);
                    let finish = flip(2, initiator.handle_response(&response)?);
                    let (confirm, responder_keys) = responder.handle_finish(&finish)?;
                    let confirm = flip(3, confirm);
                    Ok((initiator.handle_confirm(&confirm)?, responder_keys))
                })();
                assert!(
                    outcome.is_err(),
                    "bit flip at byte {position} of message {target} was accepted"
                );
            }
        }
    }

    #[test]
    fn truncated_oversized_and_misframed_messages_are_rejected() {
        let p = pair();
        let (msgs, _, _) = complete(&p);
        let init = &msgs[0];

        let fresh = || Responder::new(&p.bob);
        assert!(matches!(
            fresh().handle_init(&[], &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        assert!(matches!(
            fresh().handle_init(&init[..HEADER_LEN - 1], &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        assert!(matches!(
            fresh().handle_init(&init[..init.len() - 1], &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        let mut extended = init.clone();
        extended.push(0);
        assert!(matches!(
            fresh().handle_init(&extended, &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        // Trailing byte with a matching length field is still the wrong size.
        let mut relabelled = extended.clone();
        relabelled[6..8].copy_from_slice(&((INIT_BODY_LEN + 1) as u16).to_be_bytes());
        assert!(matches!(
            fresh().handle_init(&relabelled, &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        assert!(matches!(
            fresh().handle_init(&vec![0x43; 70_000], &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        let mut bad_magic = init.clone();
        bad_magic[0] = b'X';
        assert!(matches!(
            fresh().handle_init(&bad_magic, &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        let mut unknown_type = init.clone();
        unknown_type[5] = 0x7f;
        assert!(matches!(
            fresh().handle_init(&unknown_type, &p.bob_trust),
            Err(HandshakeError::Malformed(_))
        ));
        // A record-layer datagram is not a handshake message.
        let mut record_like = init.clone();
        record_like[0] = cryprq_core::PROTOCOL_VERSION;
        assert!(fresh().handle_init(&record_like, &p.bob_trust).is_err());
        assert_eq!(message_type(&record_like), None);
        assert_eq!(message_type(init), Some(MSG_INIT));
    }

    #[test]
    fn unknown_versions_and_suites_are_rejected_explicitly() {
        let p = pair();
        let (msgs, _, _) = complete(&p);
        for version in [0u8, 1, 3, 0xff] {
            let mut init = msgs[0].clone();
            init[4] = version;
            assert_eq!(
                Responder::new(&p.bob).handle_init(&init, &p.bob_trust),
                Err(HandshakeError::UnsupportedVersion(version))
            );
        }
        let mut init = msgs[0].clone();
        init[HEADER_LEN..HEADER_LEN + 2].copy_from_slice(&0x0000u16.to_be_bytes());
        assert_eq!(
            Responder::new(&p.bob).handle_init(&init, &p.bob_trust),
            Err(HandshakeError::UnsupportedSuite(0))
        );

        // A responder cannot steer the initiator to another version or suite.
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        let mut downgraded = response.clone();
        downgraded[4] = 1;
        assert_eq!(
            initiator.handle_response(&downgraded),
            Err(HandshakeError::UnsupportedVersion(1))
        );
        let mut other_suite = response.clone();
        other_suite[HEADER_LEN + 1] = 0x02;
        assert_eq!(
            initiator.handle_response(&other_suite),
            Err(HandshakeError::UnsupportedSuite(2))
        );
        assert!(initiator.handle_response(&response).is_ok());
    }

    #[test]
    fn messages_out_of_order_or_for_the_wrong_role_are_rejected() {
        let p = pair();
        let (msgs, _, _) = complete(&p);
        let [init, response, finish, confirm] = &msgs;

        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        // CONFIRM before RESPONSE, and messages meant for the responder.
        assert_eq!(
            initiator.handle_confirm(confirm).err(),
            Some(HandshakeError::UnexpectedMessage)
        );
        for wrong in [init, finish, confirm] {
            assert_eq!(
                initiator.handle_response(wrong),
                Err(HandshakeError::UnexpectedMessage)
            );
        }

        let mut responder = Responder::new(&p.bob);
        // FINISH before INIT, and messages meant for the initiator.
        assert_eq!(
            responder.handle_finish(finish).err(),
            Some(HandshakeError::UnexpectedMessage)
        );
        for wrong in [response, finish, confirm] {
            assert_eq!(
                responder.handle_init(wrong, &p.bob_trust),
                Err(HandshakeError::UnexpectedMessage)
            );
        }
        // Reflection: the initiator's own INIT sent back to it.
        assert_eq!(
            initiator.handle_response(initiator.init_message().to_vec().as_slice()),
            Err(HandshakeError::UnexpectedMessage)
        );
    }

    #[test]
    fn replayed_messages_from_an_earlier_session_are_rejected() {
        let p = pair();
        let (old, _, _) = complete(&p);
        let [old_init, old_response, old_finish, old_confirm] = &old;

        // A recorded RESPONSE does not authenticate a new INIT.
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        assert_eq!(
            initiator.handle_response(old_response),
            Err(HandshakeError::BadSignature)
        );

        // A recorded INIT gets a fresh RESPONSE, but the recorded FINISH is bound
        // to the old responder share and fails; no session results.
        let mut responder = Responder::new(&p.bob);
        let fresh_response = responder.handle_init(old_init, &p.bob_trust).expect("init");
        assert_ne!(&fresh_response, old_response);
        assert_eq!(
            responder.handle_finish(old_finish).err(),
            Some(HandshakeError::BadSignature)
        );

        // A recorded CONFIRM does not complete a new handshake.
        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        initiator.handle_response(&response).expect("response");
        assert_eq!(
            initiator.handle_confirm(old_confirm).err(),
            Some(HandshakeError::BadSignature)
        );
    }

    #[test]
    fn each_message_is_accepted_once_and_never_after_completion() {
        let p = pair();
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let init = initiator.init_message().to_vec();
        let response = responder.handle_init(&init, &p.bob_trust).expect("init");
        assert_eq!(
            responder.handle_init(&init, &p.bob_trust),
            Err(HandshakeError::UnexpectedMessage)
        );
        let finish = initiator.handle_response(&response).expect("response");
        assert_eq!(
            initiator.handle_response(&response),
            Err(HandshakeError::UnexpectedMessage)
        );
        assert_eq!(initiator.finish_message(), Some(finish.as_slice()));
        let (confirm, _) = responder.handle_finish(&finish).expect("finish");
        assert_eq!(
            responder.handle_finish(&finish).err(),
            Some(HandshakeError::Finished)
        );
        assert_eq!(
            responder.handle_init(&init, &p.bob_trust),
            Err(HandshakeError::Finished)
        );
        initiator.handle_confirm(&confirm).expect("confirm");
        assert_eq!(
            initiator.handle_confirm(&confirm).err(),
            Some(HandshakeError::Finished)
        );
        assert_eq!(
            initiator.handle_response(&response),
            Err(HandshakeError::Finished)
        );
    }

    #[test]
    fn wrong_key_confirmation_fails_and_yields_no_session() {
        let p = pair();
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        let mut finish = initiator.handle_response(&response).expect("response");
        let honest_finish = finish.clone();
        let last = finish.len() - 1;
        finish[last] ^= 0x80;
        // The signature is genuine, so this is a terminal failure.
        let err = responder
            .handle_finish(&finish)
            .expect_err("bad confirmation");
        assert_eq!(err, HandshakeError::BadKeyConfirmation);
        assert!(err.is_fatal());
        assert_eq!(
            responder.handle_finish(&honest_finish).err(),
            Some(HandshakeError::Finished)
        );

        // A forged CONFIRM leaves the initiator waiting, not established.
        let mut forged = frame(MSG_CONFIRM, CONFIRM_BODY_LEN);
        forged.extend_from_slice(&[0u8; MAC_LEN]);
        let err = initiator.handle_confirm(&forged).expect_err("forged");
        assert!(!err.is_fatal());
        assert_eq!(initiator.step, InitiatorStep::AwaitingConfirm);
    }

    #[test]
    fn low_order_key_share_from_an_authentic_peer_is_fatal() {
        // A pinned but misbehaving responder signs a low-order X25519 share.
        let p = pair();
        let mut initiator =
            Initiator::new(&p.alice, &p.alice_trust, &p.bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&p.bob);
        let response = responder
            .handle_init(initiator.init_message(), &p.bob_trust)
            .expect("init");
        let mut unsigned = response[..response.len() - SIG_LEN].to_vec();
        unsigned[HEADER_LEN + 66..HEADER_LEN + 98].copy_from_slice(&[0u8; 32]);
        let transcript = transcript_through_response(
            &p.alice.public_key(),
            &p.bob.public_key(),
            initiator.init_message(),
            &unsigned,
        );
        let signature = p.bob.sign(&signed_message(
            SIG_CONTEXT_RESPONDER,
            &transcript.current(),
        ));
        unsigned.extend_from_slice(&signature);
        let err = initiator.handle_response(&unsigned).expect_err("low order");
        assert!(matches!(err, HandshakeError::KeyExchange(_)));
        assert!(err.is_fatal());
        assert_eq!(
            initiator.handle_response(&response),
            Err(HandshakeError::Finished)
        );
    }

    #[test]
    fn signatures_are_domain_separated_by_role() {
        let hash = [7u8; 32];
        let id = ephemeral_identity();
        let as_responder = id.sign(&signed_message(SIG_CONTEXT_RESPONDER, &hash));
        assert!(!id
            .public_key()
            .verify(&signed_message(SIG_CONTEXT_INITIATOR, &hash), &as_responder));
        // A bare signature over the hash, as the v1 code produced, is not valid.
        assert!(!id.public_key().verify(&hash, &as_responder));
    }

    #[test]
    fn session_keys_debug_output_hides_the_master_secret() {
        let p = pair();
        let (_, keys, _) = complete(&p);
        let rendered = format!("{keys:?}");
        assert!(!rendered.contains(&hex::encode(*keys.master_secret)));
        assert!(rendered.contains(&hex::encode(keys.session_id)));
    }
}
