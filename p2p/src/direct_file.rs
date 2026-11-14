// Copyright (c) 2026 Thor Thor
// License: MIT (see LICENSE file for details)

//! Explicit, one-file direct transport. libp2p authenticates the transport key;
//! the unchanged CrypRQ hybrid handshake authenticates and confirms the
//! application session before any file record is accepted.
//!
//! Each bounded request has one encrypted, sequence-bound reply. There is no
//! discovery, retry, resume, or implicit listener. A cancelled receiver
//! drops its sole file manager, removing an unpublished partial file. The output
//! directory and its ancestors must remain under the operator's control.
//!
//! An explicit Circuit Relay v2 route is opt-in ([`Route::Relay`]). The relay's
//! own key is pinned separately from the peer's, and the peer's transport key
//! and the hybrid application session are authenticated end to end through it,
//! so the relay forwards only ciphertext. There is no NAT traversal or
//! direct-connection upgrade in this version.

use crate::libp2p::{
    connection_limits::{self, ConnectionLimits},
    multiaddr::Protocol,
    relay,
    request_response::{self, Message, ProtocolSupport, ResponseChannel},
    swarm::{
        behaviour::toggle::Toggle, dial_opts::DialOpts, ConnectionId, NetworkBehaviour, SwarmEvent,
    },
    Multiaddr, PeerId, StreamProtocol, Swarm,
};
use crate::secure_vpn::{peer_id_of, transport_keypair};
use anyhow::{anyhow, bail, Context, Result};
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p_core::ConnectedPoint;
use node::auth_handshake::{Initiator, Responder, MAX_MESSAGE_LEN};
use node::{FileMetadata, FileTransferManager, PeerPublicKey, Session, TunnelConfig};
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncReadExt as _, AsyncSeekExt};
use tokio::time::timeout;

const PROTOCOL: StreamProtocol = StreamProtocol::new("/cryprq/direct-file/1.0.0");
const CHUNK_LEN: usize = 16 * 1024;
const MAX_FRAME: usize = CHUNK_LEN + 128;
const IO_TIMEOUT: Duration = Duration::from_secs(15);
const LISTEN_TIMEOUT: Duration = Duration::from_secs(300);
// Match the existing receiver's maximum; its own quota checks remain authoritative.
const MAX_FILE_BYTES: u64 = 1 << 30;
// Existing generic DATA record type (core::MSG_TYPE_DATA), with a separate
// versioned payload domain. No changes to the cryptographic record format.
const RECORD_TYPE: u8 = 0x01;
const STREAM_ID: u32 = 2;
const MAGIC: &[u8; 5] = b"CRQF1";
const META: u8 = 1;
const CHUNK: u8 = 2;
const ACCEPTED: u8 = 3;
const COMPLETE: u8 = 4;

/// Verified receiver result, also checked in the sender's encrypted completion reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferReceipt {
    pub bytes: u64,
    pub sha256: [u8; 32],
    pub peer: PeerPublicKey,
    /// Whether the authenticated connection ran through the configured relay.
    pub relayed: bool,
}

/// How the peer is reached. Neither variant discovers addresses or trusts a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// An explicit IP multiaddr of the peer itself.
    Direct(Multiaddr),
    /// An explicit IP multiaddr of a Circuit Relay v2 server, and that relay's
    /// independently pinned key. The peer is still identified only by its own
    /// pinned key; the relay never becomes a trusted application peer.
    Relay {
        address: Multiaddr,
        key: PeerPublicKey,
    },
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // Tests can use unwrap and panic
mod tests {
    use super::*;
    use node::{NodeIdentity, TrustedPeers};
    use std::sync::Arc;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "cryprq-direct-unit-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn configs() -> (TunnelConfig, TunnelConfig) {
        let alice = Arc::new(NodeIdentity::from_secret_seed(&[1; 32]));
        let bob = Arc::new(NodeIdentity::from_secret_seed(&[2; 32]));
        (
            TunnelConfig::new(alice.clone(), TrustedPeers::from_keys([bob.public_key()])),
            TunnelConfig::new(bob, TrustedPeers::from_keys([alice.public_key()])),
        )
    }

    fn sessions() -> (Session, Session) {
        let (alice, bob) = configs();
        let mut i = Initiator::new(
            &alice.identity,
            &alice.trusted_peers,
            &bob.identity.public_key(),
        )
        .unwrap();
        let mut r = Responder::new(&bob.identity);
        let response = r.handle_init(i.init_message(), &bob.trusted_peers).unwrap();
        let finish = i.handle_response(&response).unwrap();
        let (confirm, receiver) = r.handle_finish(&finish).unwrap();
        (
            Session::from_handshake(i.handle_confirm(&confirm).unwrap()),
            Session::from_handshake(receiver),
        )
    }

    fn meta(name: &str, bytes: &[u8]) -> FileMetadata {
        FileMetadata::new(
            name.into(),
            bytes.len() as u64,
            Sha256::digest(bytes).into(),
        )
    }

    fn seal(session: &Session, opcode: u8, bytes: &[u8]) -> Vec<u8> {
        session
            .seal(STREAM_ID, RECORD_TYPE, 0, &payload(opcode, bytes))
            .unwrap()
    }

    fn chunk(offset: u64, bytes: &[u8]) -> Vec<u8> {
        let mut body = offset.to_be_bytes().to_vec();
        body.extend_from_slice(bytes);
        body
    }

    fn assert_empty(dir: &TestDir) {
        assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 0);
    }

    #[test]
    fn receiver_publishes_only_after_verified_completion_and_binds_reply() {
        let dir = TestDir::new();
        let (sender, receiver) = sessions();
        let mut state = Receiver::new(dir.0.clone());
        let metadata = meta("verified.bin", b"abcdef");
        let reply = state
            .record(&receiver, &seal(&sender, META, &metadata.serialize()))
            .unwrap();
        assert!(!check_reply(&sender, &reply, 0, 0, &metadata).unwrap());
        assert!(!dir.0.join("verified.bin").exists());
        let reply = state
            .record(&receiver, &seal(&sender, CHUNK, &chunk(0, b"abc")))
            .unwrap();
        assert!(!check_reply(&sender, &reply, 1, 3, &metadata).unwrap());
        assert!(!dir.0.join("verified.bin").exists());
        let reply = state
            .record(&receiver, &seal(&sender, CHUNK, &chunk(3, b"def")))
            .unwrap();
        assert!(check_reply(&sender, &reply, 2, 6, &metadata).unwrap());
        assert_eq!(
            std::fs::read(dir.0.join("verified.bin")).unwrap(),
            b"abcdef"
        );
        assert!(state.complete);
        assert!(state
            .record(&receiver, &seal(&sender, CHUNK, &chunk(6, b"extra")))
            .is_err());
    }

    #[test]
    fn empty_file_requires_valid_hash_before_completion_reply() {
        for valid in [true, false] {
            let dir = TestDir::new();
            let (sender, receiver) = sessions();
            let mut state = Receiver::new(dir.0.clone());
            let mut metadata = meta("empty.bin", b"");
            if !valid {
                metadata.hash = [0; 32];
            }
            let result = state.record(&receiver, &seal(&sender, META, &metadata.serialize()));
            if valid {
                assert!(check_reply(&sender, &result.unwrap(), 0, 0, &metadata).unwrap());
                assert_eq!(std::fs::metadata(dir.0.join("empty.bin")).unwrap().len(), 0);
            } else {
                assert!(result.is_err());
                drop(state);
                assert_empty(&dir);
            }
        }
    }

    #[test]
    fn tampered_replayed_reflected_and_forged_replies_are_rejected() {
        let dir = TestDir::new();
        let (sender, receiver) = sessions();
        let mut state = Receiver::new(dir.0.clone());
        let metadata = meta("partial.bin", b"abc");
        let request = seal(&sender, META, &metadata.serialize());
        let mut tampered = request.clone();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(state.record(&receiver, &tampered).is_err());
        assert_empty(&dir);
        let reply = state.record(&receiver, &request).unwrap();
        assert!(state.record(&receiver, &request).is_err());
        assert!(check_reply(&sender, &request, 0, 0, &metadata).is_err());
        let mut forged = reply.clone();
        *forged.last_mut().unwrap() ^= 1;
        assert!(check_reply(&sender, &forged, 0, 0, &metadata).is_err());
        assert!(!check_reply(&sender, &reply, 0, 0, &metadata).unwrap());
        assert!(check_reply(&sender, &reply, 0, 0, &metadata).is_err());
        drop(state);
        assert_empty(&dir);
    }

    #[test]
    fn early_wrong_type_stream_flags_version_and_out_of_order_records_are_rejected() {
        for case in 0..8 {
            let dir = TestDir::new();
            let (sender, receiver) = sessions();
            let mut state = Receiver::new(dir.0.clone());
            let metadata = meta("bad.bin", b"abc");
            let body = payload(META, &metadata.serialize());
            let record = match case {
                0 => seal(&sender, CHUNK, &chunk(0, b"abc")),
                1 => sender.seal(9, RECORD_TYPE, 0, &body).unwrap(),
                2 => sender
                    .seal(STREAM_ID, node::MSG_TYPE_VPN_PACKET, 0, &body)
                    .unwrap(),
                3 => sender.seal(STREAM_ID, RECORD_TYPE, 1, &body).unwrap(),
                4 => {
                    let mut body = body.clone();
                    body[4] = b'2';
                    sender.seal(STREAM_ID, RECORD_TYPE, 0, &body).unwrap()
                }
                5 => seal(&sender, COMPLETE, b""),
                6 => {
                    let _ = seal(&sender, META, &metadata.serialize());
                    seal(&sender, META, &metadata.serialize())
                }
                _ => b"plaintext file bytes".to_vec(),
            };
            assert!(state.record(&receiver, &record).is_err(), "case {case}");
            drop(state);
            assert_empty(&dir);
        }
    }

    #[test]
    fn unsafe_noncanonical_duplicate_and_quota_metadata_are_rejected() {
        for case in 0..8 {
            let dir = TestDir::new();
            let (sender, receiver) = sessions();
            let mut state = Receiver::new(dir.0.clone());
            let mut metadata = meta("file.bin", b"abc");
            match case {
                0 => metadata.filename = "../escape.bin".into(),
                1 => metadata.filename = "\\escape.bin".into(),
                2 => metadata.filename = "x".repeat(256),
                3 => metadata.size = MAX_FILE_BYTES + 1,
                4 => metadata.filename = ".".into(),
                7 => metadata.filename = "log\ninjection.bin".into(),
                _ => {}
            }
            let mut body = metadata.serialize();
            if case == 5 {
                body.push(0);
            }
            if case == 6 {
                state
                    .record(&receiver, &seal(&sender, META, &body))
                    .unwrap();
            }
            assert!(
                state
                    .record(&receiver, &seal(&sender, META, &body))
                    .is_err(),
                "case {case}"
            );
            drop(state);
            assert_empty(&dir);
        }
    }

    #[test]
    fn offset_truncation_hash_and_publication_race_never_publish_bad_data() {
        for case in 0..5 {
            let dir = TestDir::new();
            let (sender, receiver) = sessions();
            let mut state = Receiver::new(dir.0.clone());
            let metadata = meta("received.bin", b"good");
            state
                .record(&receiver, &seal(&sender, META, &metadata.serialize()))
                .unwrap();
            if case == 4 {
                std::fs::write(dir.0.join("received.bin"), b"local").unwrap();
            }
            let bytes = match case {
                0 => chunk(1, b"good"),
                1 => chunk(0, b"excess"),
                2 => chunk(0, b"evil"),
                3 => chunk(0, b"go"),
                _ => chunk(0, b"good"),
            };
            let result = state.record(&receiver, &seal(&sender, CHUNK, &bytes));
            if case == 3 {
                assert!(result.is_ok());
                assert!(!state.complete);
            } else {
                assert!(result.is_err());
            }
            drop(state);
            if case == 4 {
                assert_eq!(std::fs::read(dir.0.join("received.bin")).unwrap(), b"local");
            } else {
                assert_empty(&dir);
            }
        }
    }

    #[tokio::test]
    async fn codec_rejects_oversize_zero_and_truncated_frames() {
        use futures::io::Cursor;
        for bytes in [
            ((MAX_FRAME + 1) as u32).to_be_bytes().to_vec(),
            vec![0; 4],
            vec![0, 0, 0, 3, 1],
            vec![0, 0, 0, 1, 1, 2],
        ] {
            assert!(read_frame(&mut Cursor::new(bytes)).await.is_err());
        }
        assert!(
            write_frame(&mut Cursor::new(Vec::new()), &vec![0; MAX_FRAME + 1])
                .await
                .is_err()
        );
    }

    #[test]
    fn only_explicit_direct_ip_transport_addresses_are_accepted() {
        for address in ["/ip4/127.0.0.1/tcp/1", "/ip6/::1/udp/1/quic-v1"] {
            validate_address(&address.parse().unwrap()).unwrap();
        }
        for address in [
            "/ip4/127.0.0.1/udp/1",
            "/dns/example.com/tcp/1",
            "/ip4/127.0.0.1/tcp/1/p2p-circuit",
        ] {
            assert!(validate_address(&address.parse().unwrap()).is_err());
        }
    }

    async fn network_transfer(address: &str, data: Vec<u8>) {
        let dir = TestDir::new();
        let output = dir.0.join("output");
        std::fs::create_dir(&output).unwrap();
        let path = dir.0.join("network.bin");
        std::fs::write(&path, &data).unwrap();
        let (alice, bob) = configs();
        let alice_key = alice.identity.public_key();
        let bob_key = bob.identity.public_key();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let receiving = receive_inner(
            &bob,
            address.parse().unwrap(),
            alice_key,
            output.clone(),
            Some(ready_tx),
        );
        let sending = async {
            let address = ready_rx.await.unwrap();
            send_file(&alice, address, bob_key, path).await.unwrap()
        };
        let (received, sent) = timeout(Duration::from_secs(30), async {
            tokio::join!(receiving, sending)
        })
        .await
        .unwrap();
        let received = received.unwrap();
        assert_eq!(received.bytes, data.len() as u64);
        assert_eq!(sent.bytes, received.bytes);
        assert_eq!(sent.sha256, received.sha256);
        assert_eq!(std::fs::read(output.join("network.bin")).unwrap(), data);
        assert_eq!(std::fs::read_dir(output).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn tcp_transport_completes_hybrid_auth_and_multiple_chunks() {
        network_transfer(
            "/ip4/127.0.0.1/tcp/0",
            (0..CHUNK_LEN * 2 + 17).map(|n| n as u8).collect(),
        )
        .await;
    }

    #[tokio::test]
    async fn quic_transport_completes_hybrid_auth_and_empty_file() {
        network_transfer("/ip4/127.0.0.1/udp/0/quic-v1", Vec::new()).await;
    }

    #[tokio::test]
    async fn rejects_unpinned_peer_before_listener_or_file_creation() {
        let dir = TestDir::new();
        let (mut alice, bob) = configs();
        alice.trusted_peers = TrustedPeers::from_keys([]);
        assert!(receive_file(
            &alice,
            "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            bob.identity.public_key(),
            dir.0.clone()
        )
        .await
        .is_err());
        assert_empty(&dir);
    }

    #[tokio::test]
    async fn receiver_rejects_early_data_downgrade_and_transport_application_identity_mismatch() {
        for case in 0..4 {
            let dir = TestDir::new();
            let (alice, mut bob) = configs();
            let mallory = NodeIdentity::from_secret_seed(&[3; 32]);
            bob.trusted_peers =
                TrustedPeers::from_keys([alice.identity.public_key(), mallory.public_key()]);
            let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
            let receiving = receive_inner(
                &bob,
                "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
                alice.identity.public_key(),
                dir.0.clone(),
                Some(ready_tx),
            );
            let attacking = async {
                let address = ready_rx.await.unwrap();
                let peer = peer_id_of(&bob.identity.public_key()).unwrap();
                // Transport really belongs to Alice, but application INIT may
                // claim Mallory, who is also pinned. The expected key must win.
                let mut transport = swarm(&alice, false, false).unwrap();
                transport
                    .dial(DialOpts::peer_id(peer).addresses(vec![address]).build())
                    .unwrap();
                let initiator = Initiator::new(
                    if case == 1 { &mallory } else { &alice.identity },
                    &alice.trusted_peers,
                    &bob.identity.public_key(),
                )
                .unwrap();
                let mut request = initiator.init_message().to_vec();
                match case {
                    0 => request = payload(CHUNK, b"early plaintext file bytes"),
                    2 => request[4] = 1,
                    3 => request[5] = node::auth_handshake::MSG_FINISH,
                    _ => {}
                }
                assert!(exchange(&mut transport, peer, request).await.is_err());
            };
            let (result, ()) = timeout(Duration::from_secs(5), async {
                tokio::join!(receiving, attacking)
            })
            .await
            .unwrap();
            assert!(result.is_err(), "case {case}");
            assert_empty(&dir);
        }
    }

    async fn admitted_partial(
        config: &TunnelConfig,
        address: Multiaddr,
        expected: PeerPublicKey,
    ) -> Swarm<Behaviour> {
        let peer = peer_id_of(&expected).unwrap();
        let mut transport = swarm(config, false, false).unwrap();
        transport
            .dial(DialOpts::peer_id(peer).addresses(vec![address]).build())
            .unwrap();
        let mut initiator =
            Initiator::new(&config.identity, &config.trusted_peers, &expected).unwrap();
        let response = exchange(&mut transport, peer, initiator.init_message().to_vec())
            .await
            .unwrap();
        let finish = initiator.handle_response(&response).unwrap();
        let confirm = exchange(&mut transport, peer, finish).await.unwrap();
        let session = Session::from_handshake(initiator.handle_confirm(&confirm).unwrap());
        let metadata = meta("partial.bin", b"abcdef");
        let response = exchange(
            &mut transport,
            peer,
            seal(&session, META, &metadata.serialize()),
        )
        .await
        .unwrap();
        assert!(!check_reply(&session, &response, 0, 0, &metadata).unwrap());
        let response = exchange(
            &mut transport,
            peer,
            seal(&session, CHUNK, &chunk(0, b"abc")),
        )
        .await
        .unwrap();
        assert!(!check_reply(&session, &response, 1, 3, &metadata).unwrap());
        transport
    }

    #[tokio::test]
    async fn disconnect_after_admission_removes_unpublished_partial() {
        let dir = TestDir::new();
        let (alice, bob) = configs();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let receiving = receive_inner(
            &bob,
            "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            alice.identity.public_key(),
            dir.0.clone(),
            Some(ready_tx),
        );
        let sending = async {
            let address = ready_rx.await.unwrap();
            let peer = peer_id_of(&bob.identity.public_key()).unwrap();
            let mut transport = admitted_partial(&alice, address, bob.identity.public_key()).await;
            assert!(!dir.0.join("partial.bin").exists());
            assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
            transport.disconnect_peer_id(peer).unwrap();
            loop {
                if matches!(transport.select_next_some().await, SwarmEvent::ConnectionClosed { peer_id, .. } if peer_id == peer)
                {
                    break;
                }
            }
        };
        let (received, ()) = timeout(Duration::from_secs(5), async {
            tokio::join!(receiving, sending)
        })
        .await
        .unwrap();
        assert!(received.is_err());
        assert_empty(&dir);
    }

    #[tokio::test]
    async fn cancellation_after_admission_removes_unpublished_partial() {
        let dir = TestDir::new();
        let (alice, bob) = configs();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let (partial_tx, partial_rx) = tokio::sync::oneshot::channel();
        let mut receiving = Box::pin(receive_inner(
            &bob,
            "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            alice.identity.public_key(),
            dir.0.clone(),
            Some(ready_tx),
        ));
        let mut sending = Box::pin(async {
            let address = ready_rx.await.unwrap();
            let _transport = admitted_partial(&alice, address, bob.identity.public_key()).await;
            partial_tx.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        timeout(Duration::from_secs(5), async {
            tokio::select! {
                result = &mut receiving => panic!("receiver ended early: {result:?}"),
                () = &mut sending => panic!("sender ended early"),
                result = partial_rx => result.unwrap(),
            }
        })
        .await
        .unwrap();
        assert!(!dir.0.join("partial.bin").exists());
        assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
        drop(receiving);
        drop(sending);
        assert_empty(&dir);
    }

    #[test]
    fn records_and_replies_from_another_session_are_rejected() {
        let dir = TestDir::new();
        let (old_sender, old_receiver) = sessions();
        let (sender, receiver) = sessions();
        let metadata = meta("replay.bin", b"abc");
        let old_request = seal(&old_sender, META, &metadata.serialize());
        let mut old_state = Receiver::new(dir.0.clone());
        let old_reply = old_state.record(&old_receiver, &old_request).unwrap();
        drop(old_state);
        assert_empty(&dir);
        let mut state = Receiver::new(dir.0.clone());
        assert!(state.record(&receiver, &old_request).is_err());
        assert!(check_reply(&sender, &old_reply, 0, 0, &metadata).is_err());
        // A rejected replay leaves the fresh session's own sequence usable.
        let reply = state
            .record(&receiver, &seal(&sender, META, &metadata.serialize()))
            .unwrap();
        assert!(!check_reply(&sender, &reply, 0, 0, &metadata).unwrap());
        drop(state);
        assert_empty(&dir);
    }

    #[test]
    fn modified_handshake_response_fails_initiator_verification() {
        let (alice, bob) = configs();
        let mut initiator = Initiator::new(
            &alice.identity,
            &alice.trusted_peers,
            &bob.identity.public_key(),
        )
        .unwrap();
        let mut responder = Responder::new(&bob.identity);
        let mut response = responder
            .handle_init(initiator.init_message(), &bob.trusted_peers)
            .unwrap();
        *response.last_mut().unwrap() ^= 1;
        assert!(initiator.handle_response(&response).is_err());
    }

    #[tokio::test]
    async fn receiver_rejects_failed_key_confirmation_before_any_file_record() {
        let dir = TestDir::new();
        let (alice, bob) = configs();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let receiving = receive_inner(
            &bob,
            "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            alice.identity.public_key(),
            dir.0.clone(),
            Some(ready_tx),
        );
        let attacking = async {
            let address = ready_rx.await.unwrap();
            let peer = peer_id_of(&bob.identity.public_key()).unwrap();
            let mut transport = swarm(&alice, false, false).unwrap();
            transport
                .dial(DialOpts::peer_id(peer).addresses(vec![address]).build())
                .unwrap();
            let mut initiator = Initiator::new(
                &alice.identity,
                &alice.trusted_peers,
                &bob.identity.public_key(),
            )
            .unwrap();
            let response = exchange(&mut transport, peer, initiator.init_message().to_vec())
                .await
                .unwrap();
            let mut finish = initiator.handle_response(&response).unwrap();
            let middle = finish.len() / 2;
            finish[middle] ^= 1;
            assert!(exchange(&mut transport, peer, finish).await.is_err());
        };
        let (result, ()) = timeout(Duration::from_secs(5), async {
            tokio::join!(receiving, attacking)
        })
        .await
        .unwrap();
        assert!(result.is_err());
        assert_empty(&dir);
    }

    #[tokio::test]
    async fn sender_rejects_listener_whose_transport_key_is_not_the_expected_pin() {
        let dir = TestDir::new();
        let source = dir.0.join("source.bin");
        std::fs::write(&source, b"abc").unwrap();
        let output = dir.0.join("output");
        std::fs::create_dir(&output).unwrap();
        let (mut alice, bob) = configs();
        let mallory = NodeIdentity::from_secret_seed(&[3; 32]).public_key();
        alice.trusted_peers = TrustedPeers::from_keys([bob.identity.public_key(), mallory]);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let mut receiving = Box::pin(receive_inner(
            &bob,
            "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            alice.identity.public_key(),
            output.clone(),
            Some(ready_tx),
        ));
        let mut sending = Box::pin(async {
            let address = ready_rx.await.unwrap();
            // Bob listens there, but the sender independently expects Mallory.
            send_file(&alice, address, mallory, source.clone()).await
        });
        let sent = timeout(Duration::from_secs(10), async {
            tokio::select! {
                sent = &mut sending => sent,
                received = &mut receiving => {
                    assert!(received.is_err());
                    sending.await
                }
            }
        })
        .await
        .unwrap();
        assert!(sent.is_err());
        drop(receiving);
        assert_eq!(std::fs::read_dir(&output).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn receiver_requires_existing_non_symlink_output_directory() {
        let dir = TestDir::new();
        let real = dir.0.join("real");
        std::fs::create_dir(&real).unwrap();
        let link = dir.0.join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let file = dir.0.join("file");
        std::fs::write(&file, b"").unwrap();
        let (alice, bob) = configs();
        for output in [link, file, dir.0.join("missing")] {
            let result = timeout(
                Duration::from_secs(5),
                receive_file(
                    &bob,
                    "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
                    alice.identity.public_key(),
                    output.clone(),
                ),
            )
            .await
            .expect("output validation must precede listening");
            assert!(result.is_err(), "{output:?}");
        }
        assert_eq!(std::fs::read_dir(&real).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn non_regular_sources_are_rejected_without_blocking() {
        let dir = TestDir::new();
        let regular = dir.0.join("regular.bin");
        std::fs::write(&regular, b"abc").unwrap();
        let link = dir.0.join("link.bin");
        std::os::unix::fs::symlink(&regular, &link).unwrap();
        let fifo = dir.0.join("fifo.bin");
        assert!(std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success());
        let (alice, bob) = configs();
        for source in [
            dir.0.clone(),
            link.clone(),
            fifo.clone(),
            dir.0.join("missing.bin"),
        ] {
            let result = timeout(
                Duration::from_secs(5),
                send_file(
                    &alice,
                    "/ip4/127.0.0.1/tcp/9".parse().unwrap(),
                    bob.identity.public_key(),
                    source.clone(),
                ),
            )
            .await
            .expect("source validation must not block");
            assert!(result.is_err(), "{source:?}");
        }
        // If a FIFO or symlink replaces an inspected path before the open, the
        // open itself neither blocks nor follows, and fstat rejects it.
        let opened = timeout(Duration::from_secs(5), open_nofollow(&fifo))
            .await
            .expect("FIFO open must not block")
            .unwrap();
        assert!(!opened.metadata().await.unwrap().is_file());
        assert!(timeout(Duration::from_secs(5), open_nofollow(&link))
            .await
            .unwrap()
            .is_err());
    }

    // ---- Opt-in relay route -------------------------------------------------

    use crate::direct_relay::{serve_relay, RelayLimits};

    fn relay_limits(max_circuit_bytes: u64) -> RelayLimits {
        RelayLimits {
            max_circuit_bytes,
            max_circuit_duration: Duration::from_secs(60),
            max_circuits: 4,
            max_reservations: 4,
        }
    }

    /// An in-process relay with its own identity. Aborted when dropped.
    struct TestRelay {
        address: Multiaddr,
        key: PeerPublicKey,
        task: tokio::task::JoinHandle<()>,
    }
    impl Drop for TestRelay {
        fn drop(&mut self) {
            self.task.abort();
        }
    }
    impl TestRelay {
        fn route(&self) -> Route {
            Route::Relay {
                address: self.address.clone(),
                key: self.key,
            }
        }
    }

    async fn start_relay(listen: &str, max_circuit_bytes: u64) -> TestRelay {
        let identity = NodeIdentity::from_secret_seed(&[9; 32]);
        let key = identity.public_key();
        let listen: Multiaddr = listen.parse().unwrap();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let _ = serve_relay(
                &identity,
                vec![listen],
                Vec::new(),
                relay_limits(max_circuit_bytes),
                Some(ready_tx),
            )
            .await;
        });
        let mut advertised = timeout(Duration::from_secs(5), ready_rx)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(advertised.len(), 1);
        TestRelay {
            address: advertised.remove(0),
            key,
            task,
        }
    }

    async fn relayed_transfer(listen: &str, data: Vec<u8>) {
        let relay = start_relay(listen, 1 << 20).await;
        relayed_transfer_on(&relay, data).await;
    }

    async fn relayed_transfer_on(relay: &TestRelay, data: Vec<u8>) {
        let dir = TestDir::new();
        let output = dir.0.join("output");
        std::fs::create_dir(&output).unwrap();
        let path = dir.0.join("relayed.bin");
        std::fs::write(&path, &data).unwrap();
        let (alice, bob) = configs();
        let (alice_key, bob_key) = (alice.identity.public_key(), bob.identity.public_key());
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let receiving = receive_routed(
            &bob,
            relay.route(),
            alice_key,
            output.clone(),
            Some(ready_tx),
        );
        let sending = async {
            // The receiver reports its circuit address only after the
            // relay accepted its reservation.
            let listening = ready_rx.await.unwrap();
            assert!(listening.iter().any(|p| p == Protocol::P2pCircuit));
            send_file_routed(&alice, relay.route(), bob_key, path.clone())
                .await
                .unwrap()
        };
        let (received, sent) = timeout(Duration::from_secs(30), async {
            tokio::join!(receiving, sending)
        })
        .await
        .unwrap();
        let received = received.unwrap();
        assert!(received.relayed && sent.relayed);
        assert_eq!((received.bytes, received.sha256), (sent.bytes, sent.sha256));
        assert_eq!(received.bytes, data.len() as u64);
        assert_eq!(std::fs::read(output.join("relayed.bin")).unwrap(), data);
        assert_eq!(std::fs::read_dir(output).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn relayed_tcp_transfer_authenticates_end_to_end_with_multiple_chunks() {
        relayed_transfer(
            "/ip4/127.0.0.1/tcp/0",
            (0..CHUNK_LEN * 2 + 17).map(|n| n as u8).collect(),
        )
        .await;
    }

    #[tokio::test]
    async fn relayed_quic_transfer_authenticates_end_to_end_with_multiple_chunks() {
        relayed_transfer(
            "/ip4/127.0.0.1/udp/0/quic-v1",
            (0..CHUNK_LEN * 3).map(|n| (n * 7) as u8).collect(),
        )
        .await;
    }

    #[tokio::test]
    async fn relay_serves_repeated_quic_transfers_between_the_same_identities() {
        // Functional coverage for back-to-back reservations by one identity.
        // In-process, a dropped QUIC endpoint still sends its close, so the
        // stale-reservation case after an abrupt process exit is covered by
        // scripts/e2e-direct-relay-docker.sh, not here.
        let relay = start_relay("/ip4/127.0.0.1/udp/0/quic-v1", 1 << 20).await;
        for round in 0..3u8 {
            relayed_transfer_on(&relay, vec![round; CHUNK_LEN + 1]).await;
        }
    }

    #[tokio::test]
    async fn relay_routes_are_validated_before_any_network_use() {
        let dir = TestDir::new();
        let path = dir.0.join("source.bin");
        std::fs::write(&path, b"abc").unwrap();
        let (alice, bob) = configs();
        let bob_key = bob.identity.public_key();
        let relay_key = NodeIdentity::from_secret_seed(&[9; 32]).public_key();
        for (address, key) in [
            ("/dns/relay.example/tcp/1", relay_key),
            ("/ip4/127.0.0.1/tcp/1/p2p-circuit", relay_key),
            ("/ip4/127.0.0.1/tcp/1", bob_key),
            ("/ip4/127.0.0.1/tcp/1", alice.identity.public_key()),
        ] {
            let route = Route::Relay {
                address: address.parse().unwrap(),
                key,
            };
            let sent = timeout(
                Duration::from_secs(5),
                send_file_routed(&alice, route.clone(), bob_key, path.clone()),
            )
            .await
            .unwrap();
            assert!(sent.is_err(), "{address}");
            let received = timeout(
                Duration::from_secs(5),
                receive_file_routed(&bob, route, alice.identity.public_key(), dir.0.clone()),
            )
            .await
            .unwrap();
            assert!(received.is_err(), "{address}");
        }
    }

    #[test]
    fn relay_limits_must_be_explicit_and_bounded() {
        assert!(relay_limits(1 << 20).validate().is_ok());
        let mut limits = relay_limits(0);
        assert!(limits.validate().is_err());
        limits.max_circuit_bytes = RelayLimits::MAX_CIRCUIT_BYTES + 1;
        assert!(limits.validate().is_err());
        let mut limits = relay_limits(1 << 20);
        limits.max_circuit_duration = Duration::ZERO;
        assert!(limits.validate().is_err());
        let mut limits = relay_limits(1 << 20);
        limits.max_circuits = 0;
        assert!(limits.validate().is_err());
    }

    /// Runs a receiver through `relay` and a sender with the given route and
    /// expected key; returns the sender's result and the output directory.
    async fn relayed_attempt(
        relay: &TestRelay,
        sender_route: Route,
        sender_expects: Option<PeerPublicKey>,
        data: &[u8],
    ) -> (Result<TransferReceipt>, TestDir) {
        let dir = TestDir::new();
        let output = dir.0.join("output");
        std::fs::create_dir(&output).unwrap();
        let path = dir.0.join("attempt.bin");
        std::fs::write(&path, data).unwrap();
        let (mut alice, bob) = configs();
        let mallory = NodeIdentity::from_secret_seed(&[3; 32]).public_key();
        alice.trusted_peers = TrustedPeers::from_keys([bob.identity.public_key(), mallory]);
        let expected = sender_expects.unwrap_or_else(|| bob.identity.public_key());
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let mut receiving = Box::pin(receive_routed(
            &bob,
            relay.route(),
            alice.identity.public_key(),
            output.clone(),
            Some(ready_tx),
        ));
        let mut sending = Box::pin(async {
            ready_rx.await.unwrap();
            send_file_routed(&alice, sender_route, expected, path.clone()).await
        });
        let sent = timeout(Duration::from_secs(30), async {
            tokio::select! {
                sent = &mut sending => sent,
                received = &mut receiving => {
                    assert!(received.is_err(), "receiver must not succeed");
                    sending.await
                }
            }
        })
        .await
        .unwrap();
        drop(receiving);
        (sent, dir)
    }

    #[tokio::test]
    async fn relay_with_a_different_key_than_pinned_is_refused() {
        let relay = start_relay("/ip4/127.0.0.1/tcp/0", 1 << 20).await;
        let impostor = NodeIdentity::from_secret_seed(&[8; 32]).public_key();
        let route = Route::Relay {
            address: relay.address.clone(),
            key: impostor,
        };
        let (sent, dir) = relayed_attempt(&relay, route, None, b"abc").await;
        assert!(sent.is_err());
        assert_eq!(std::fs::read_dir(dir.0.join("output")).unwrap().count(), 0);
        // A receiver pinning the wrong relay key cannot even reserve.
        let (alice, bob) = configs();
        let result = timeout(
            Duration::from_secs(15),
            receive_file_routed(
                &bob,
                Route::Relay {
                    address: relay.address.clone(),
                    key: impostor,
                },
                alice.identity.public_key(),
                dir.0.join("output"),
            ),
        )
        .await
        .unwrap();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn relay_does_not_deliver_to_an_unreserved_or_unexpected_target() {
        let relay = start_relay("/ip4/127.0.0.1/tcp/0", 1 << 20).await;
        // Bob holds the only reservation; the sender asks the relay for Mallory.
        let mallory = NodeIdentity::from_secret_seed(&[3; 32]).public_key();
        let (sent, dir) = relayed_attempt(&relay, relay.route(), Some(mallory), b"abc").await;
        assert!(sent.is_err());
        assert_eq!(std::fs::read_dir(dir.0.join("output")).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn relay_circuit_limit_cuts_the_transfer_without_publication() {
        // The relay closes the circuit mid-transfer; nothing may be published
        // and the receiver's partial file must be removed.
        let relay = start_relay("/ip4/127.0.0.1/tcp/0", 64 * 1024).await;
        let data: Vec<u8> = (0..CHUNK_LEN * 12).map(|n| n as u8).collect();
        let (sent, dir) = relayed_attempt(&relay, relay.route(), None, &data).await;
        assert!(sent.is_err());
        assert_eq!(std::fs::read_dir(dir.0.join("output")).unwrap().count(), 0);
    }
}

#[derive(Clone, Default)]
struct Codec;

async fn read_frame<T: AsyncRead + Unpin + Send>(io: &mut T) -> io::Result<Vec<u8>> {
    let mut bytes = [0; 4];
    io.read_exact(&mut bytes).await?;
    let len = u32::from_be_bytes(bytes) as usize;
    if len == 0 || len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid direct-file frame length",
        ));
    }
    let mut frame = vec![0; len];
    io.read_exact(&mut frame).await?;
    let mut trailing = [0];
    if io.read(&mut trailing).await? != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing direct-file frame bytes",
        ));
    }
    Ok(frame)
}

async fn write_frame<T: AsyncWrite + Unpin + Send>(io: &mut T, frame: &[u8]) -> io::Result<()> {
    if frame.is_empty() || frame.len() > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid direct-file frame length",
        ));
    }
    io.write_all(&(frame.len() as u32).to_be_bytes()).await?;
    io.write_all(frame).await?;
    io.close().await
}

impl request_response::Codec for Codec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Vec<u8>>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_frame(io).await
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Vec<u8>>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_frame(io).await
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        frame: Vec<u8>,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_frame(io, &frame).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        frame: Vec<u8>,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_frame(io, &frame).await
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event", prelude = "libp2p_swarm::derive_prelude")]
struct Behaviour {
    files: request_response::Behaviour<Codec>,
    limits: connection_limits::Behaviour,
    relay: Toggle<relay::client::Behaviour>,
}

#[derive(Debug)]
enum Event {
    Files(request_response::Event<Vec<u8>, Vec<u8>>),
    Relay(relay::client::Event),
}

impl From<request_response::Event<Vec<u8>, Vec<u8>>> for Event {
    fn from(event: request_response::Event<Vec<u8>, Vec<u8>>) -> Self {
        Self::Files(event)
    }
}
impl From<relay::client::Event> for Event {
    fn from(event: relay::client::Event) -> Self {
        Self::Relay(event)
    }
}
impl From<Infallible> for Event {
    fn from(event: Infallible) -> Self {
        match event {}
    }
}

fn swarm(config: &TunnelConfig, inbound: bool, relayed: bool) -> Result<Swarm<Behaviour>> {
    // A relayed route adds exactly one outgoing connection, to the relay. The
    // sender's relayed connection to the peer is also outgoing; the
    // receiver's arrives as one incoming circuit.
    let extra = u32::from(relayed);
    let limits = ConnectionLimits::default()
        .with_max_pending_incoming(Some(if inbound { 4 } else { 0 }))
        .with_max_pending_outgoing(Some(1 + extra))
        .with_max_established_incoming(Some(if inbound { 1 } else { 0 }))
        .with_max_established_outgoing(Some(1 + extra))
        .with_max_established_per_peer(Some(1))
        .with_max_established(Some(1 + extra));
    let files = request_response::Behaviour::with_codec(
        Codec,
        [(PROTOCOL, ProtocolSupport::Full)],
        request_response::Config::default()
            .with_request_timeout(IO_TIMEOUT)
            .with_max_concurrent_streams(1),
    );
    let limits = connection_limits::Behaviour::new(limits);
    let keypair = transport_keypair(&config.identity)?;
    let configure =
        |c: crate::libp2p::swarm::Config| c.with_idle_connection_timeout(IO_TIMEOUT * 2);
    if relayed {
        crate::libp2p::build_relay_client_swarm(
            keypair,
            |client| Behaviour {
                files,
                limits,
                relay: Toggle::from(Some(client)),
            },
            configure,
        )
    } else {
        crate::libp2p::build_swarm(
            keypair,
            Behaviour {
                files,
                limits,
                relay: Toggle::from(None),
            },
            configure,
        )
    }
    .map_err(|_| anyhow!("cannot initialize direct-file transport"))
}

/// The relay's peer ID, from its separately pinned key, which must not be the
/// local identity or the expected peer.
fn relay_peer(
    config: &TunnelConfig,
    expected: PeerPublicKey,
    key: PeerPublicKey,
) -> Result<PeerId> {
    if key == config.identity.public_key() || key == expected {
        bail!("relay key must differ from the local identity and the expected peer");
    }
    peer_id_of(&key)
}

/// `<relay address>/p2p/<relay>/p2p-circuit`.
fn circuit(address: &Multiaddr, relay: PeerId) -> Multiaddr {
    address
        .clone()
        .with(Protocol::P2p(relay))
        .with(Protocol::P2pCircuit)
}

fn through_relay(endpoint: &ConnectedPoint) -> bool {
    let circuit = |address: &Multiaddr| address.iter().any(|p| p == Protocol::P2pCircuit);
    match endpoint {
        ConnectedPoint::Dialer { address, .. } => circuit(address),
        ConnectedPoint::Listener {
            local_addr,
            send_back_addr,
        } => circuit(local_addr) || circuit(send_back_addr),
    }
}

fn route_label(relayed: bool) -> &'static str {
    if relayed {
        "relay"
    } else {
        "direct"
    }
}

fn validate_peer(config: &TunnelConfig, expected: PeerPublicKey) -> Result<PeerId> {
    if !config.trusted_peers.contains(&expected) {
        bail!("expected peer is not pinned");
    }
    if config.identity.public_key() == expected {
        bail!("expected peer is the local identity");
    }
    peer_id_of(&expected)
}

pub(crate) fn validate_address(address: &Multiaddr) -> Result<()> {
    let mut parts = address.iter();
    if !matches!(parts.next(), Some(Protocol::Ip4(_) | Protocol::Ip6(_))) {
        bail!("direct-file address must begin with an explicit IP address");
    }
    match parts.next() {
        Some(Protocol::Tcp(_)) => {}
        Some(Protocol::Udp(_)) if matches!(parts.next(), Some(Protocol::QuicV1)) => {}
        _ => bail!("direct-file address must use TCP or UDP/QUIC-v1"),
    }
    // The independently supplied pinned key is the sole source of the peer ID.
    if parts.next().is_some() {
        bail!("direct-file address contains unsupported components");
    }
    Ok(())
}

async fn exchange(swarm: &mut Swarm<Behaviour>, peer: PeerId, request: Vec<u8>) -> Result<Vec<u8>> {
    let id = swarm.behaviour_mut().files.send_request(&peer, request);
    timeout(IO_TIMEOUT, async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(Event::Files(request_response::Event::Message {
                    peer: received_peer,
                    message:
                        Message::Response {
                            request_id,
                            response,
                        },
                    ..
                })) if received_peer == peer && request_id == id => return Ok(response),
                SwarmEvent::Behaviour(Event::Files(request_response::Event::OutboundFailure {
                    request_id,
                    ..
                })) if request_id == id => bail!("direct-file request failed"),
                SwarmEvent::Behaviour(Event::Files(request_response::Event::Message {
                    message: Message::Request { .. },
                    ..
                })) => bail!("sender received an unexpected application request"),
                SwarmEvent::ConnectionClosed { peer_id, .. } if peer_id == peer => {
                    bail!("direct-file peer disconnected")
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    // The dial error names addresses and peer IDs only, which
                    // the operator supplied; it lets an unreachable relay be
                    // told apart from an identity mismatch.
                    let target = if peer_id == Some(peer) {
                        "peer"
                    } else {
                        "relay"
                    };
                    bail!(
                        "direct-file {target} is unreachable or has a mismatched identity: {error}"
                    )
                }
                _ => {}
            }
        }
    })
    .await
    .context("direct-file request timed out")?
}

struct Request {
    bytes: Vec<u8>,
    channel: ResponseChannel<Vec<u8>>,
    id: request_response::InboundRequestId,
}

/// The next request from `peer`. Any other connection is closed, except the
/// one to the configured relay, which carries the peer's circuit.
async fn next_request(
    swarm: &mut Swarm<Behaviour>,
    peer: PeerId,
    relay: Option<PeerId>,
) -> Result<Request> {
    timeout(IO_TIMEOUT, async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(Event::Files(request_response::Event::Message {
                    peer: received_peer,
                    message:
                        Message::Request {
                            request,
                            channel,
                            request_id,
                        },
                    ..
                })) if received_peer == peer => {
                    return Ok(Request {
                        bytes: request,
                        channel,
                        id: request_id,
                    })
                }
                SwarmEvent::Behaviour(Event::Files(request_response::Event::InboundFailure {
                    peer: failed,
                    ..
                })) if failed == peer => bail!("invalid or incomplete direct-file request"),
                SwarmEvent::ConnectionClosed { peer_id, .. } if peer_id == peer => {
                    bail!("direct-file peer disconnected")
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. }
                    if peer_id != peer && Some(peer_id) != relay =>
                {
                    let _ = swarm.disconnect_peer_id(peer_id);
                }
                _ => {}
            }
        }
    })
    .await
    .context("direct-file peer timed out")?
}

fn reply(swarm: &mut Swarm<Behaviour>, request: Request, response: Vec<u8>) -> Result<()> {
    swarm
        .behaviour_mut()
        .files
        .send_response(request.channel, response)
        .map_err(|_| anyhow!("direct-file reply channel closed"))
}

fn payload(opcode: u8, bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(MAGIC.len() + 1 + bytes.len());
    out.extend_from_slice(MAGIC);
    out.push(opcode);
    out.extend_from_slice(bytes);
    out
}

fn open_record(session: &Session, bytes: &[u8], expected_seq: u64) -> Result<Vec<u8>> {
    if bytes.len() > MAX_FRAME {
        bail!("oversized direct-file record");
    }
    let (header, data) = session
        .open(bytes)
        .context("direct-file record authentication failed")?;
    if header.message_type != RECORD_TYPE
        || header.stream_id != STREAM_ID
        || header.flags != 0
        || header.epoch != 0
        || header.sequence_number != expected_seq
        || !data.starts_with(MAGIC)
        || data.len() < MAGIC.len() + 1
    {
        bail!("unexpected direct-file record");
    }
    Ok(data)
}

struct Receiver {
    manager: FileTransferManager,
    metadata: Option<FileMetadata>,
    received: u64,
    sequence: u64,
    complete: bool,
}

impl Receiver {
    fn new(output: PathBuf) -> Self {
        Self {
            manager: FileTransferManager::new_redacted(output),
            metadata: None,
            received: 0,
            sequence: 0,
            complete: false,
        }
    }

    fn record(&mut self, session: &Session, request: &[u8]) -> Result<Vec<u8>> {
        if self.complete {
            bail!("transfer already complete");
        }
        let data = open_record(session, request, self.sequence)?;
        let body = &data[MAGIC.len() + 1..];
        match (data[MAGIC.len()], &self.metadata) {
            (META, None) => {
                let metadata =
                    FileMetadata::deserialize(body).context("invalid direct-file metadata")?;
                if metadata.serialize() != body
                    || metadata.filename.len() > 255
                    || metadata.filename.contains(['/', '\\', '\0'])
                    || metadata.filename.chars().any(char::is_control)
                {
                    bail!("noncanonical or unsafe direct-file metadata");
                }
                self.manager
                    .start_incoming_transfer(STREAM_ID, metadata.clone())
                    .map_err(|_| anyhow!("receiver rejected file admission"))?;
                self.complete = metadata.size == 0;
                self.metadata = Some(metadata);
            }
            (CHUNK, Some(metadata)) => {
                if body.len() <= 8 || body.len() > CHUNK_LEN + 8 {
                    bail!("invalid direct-file chunk length");
                }
                let offset = u64::from_be_bytes(body[..8].try_into().expect("length checked"));
                if offset != self.received {
                    bail!("out-of-order direct-file chunk");
                }
                let received = self
                    .received
                    .checked_add((body.len() - 8) as u64)
                    .filter(|received| *received <= metadata.size)
                    .ok_or_else(|| anyhow!("direct-file chunk exceeds declared size"))?;
                self.manager
                    .write_chunk(STREAM_ID, &body[8..])
                    .map_err(|_| anyhow!("receiver rejected file data or verification"))?;
                self.received = received;
                self.complete = received == metadata.size;
            }
            _ => bail!("unexpected direct-file message order"),
        }
        let metadata = self.metadata.as_ref().expect("metadata accepted");
        let mut acknowledgment = Vec::with_capacity(48);
        acknowledgment.extend_from_slice(&self.sequence.to_be_bytes());
        acknowledgment.extend_from_slice(&self.received.to_be_bytes());
        acknowledgment.extend_from_slice(&metadata.hash);
        let response = session.seal(
            STREAM_ID,
            RECORD_TYPE,
            0,
            &payload(
                if self.complete { COMPLETE } else { ACCEPTED },
                &acknowledgment,
            ),
        )?;
        self.sequence += 1;
        Ok(response)
    }

    fn receipt(&self, peer: PeerPublicKey, relayed: bool) -> Result<TransferReceipt> {
        if !self.complete {
            bail!("direct-file transfer is incomplete");
        }
        let metadata = self.metadata.as_ref().expect("completed metadata");
        Ok(TransferReceipt {
            bytes: metadata.size,
            sha256: metadata.hash,
            peer,
            relayed,
        })
    }
}

fn check_reply(
    session: &Session,
    response: &[u8],
    sequence: u64,
    received: u64,
    metadata: &FileMetadata,
) -> Result<bool> {
    let data = open_record(session, response, sequence)?;
    let complete = received == metadata.size;
    let mut expected = sequence.to_be_bytes().to_vec();
    expected.extend_from_slice(&received.to_be_bytes());
    expected.extend_from_slice(&metadata.hash);
    if data != payload(if complete { COMPLETE } else { ACCEPTED }, &expected) {
        bail!("invalid direct-file receiver acknowledgment");
    }
    Ok(complete)
}

const NOT_REGULAR: &str =
    "direct-file source must be a regular file, not a symlink, directory or special file";

/// Open without following a final symlink and without blocking on a FIFO or
/// device that replaced the path after inspection.
async fn open_nofollow(path: &Path) -> io::Result<tokio::fs::File> {
    let mut options = tokio::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    options.open(path).await
}

/// Reject non-regular sources before any potentially blocking open, then
/// verify the opened handle is the same regular file that was inspected.
async fn open_source(path: &Path) -> Result<(tokio::fs::File, std::fs::Metadata)> {
    let inspected = tokio::fs::symlink_metadata(path)
        .await
        .context("cannot inspect direct-file source")?;
    if !inspected.is_file() {
        bail!(NOT_REGULAR);
    }
    let file = open_nofollow(path)
        .await
        .context("cannot open direct-file source")?;
    let opened = file.metadata().await?;
    if !opened.is_file() {
        bail!(NOT_REGULAR);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (opened.dev(), opened.ino()) != (inspected.dev(), inspected.ino()) {
            bail!("direct-file source changed while it was being opened");
        }
    }
    Ok((file, opened))
}

/// Send one regular file to the explicitly pinned peer at a reachable IP multiaddr.
/// Success requires an authenticated reply confirming the exact size and SHA-256
/// after the receiver's existing verification and atomic publication succeeded.
pub async fn send_file(
    config: &TunnelConfig,
    address: Multiaddr,
    expected: PeerPublicKey,
    path: PathBuf,
) -> Result<TransferReceipt> {
    send_file_routed(config, Route::Direct(address), expected, path).await
}

/// [`send_file`] over an explicit [`Route`]. With [`Route::Relay`], the peer is
/// dialed only through that relay's circuit, never directly.
pub async fn send_file_routed(
    config: &TunnelConfig,
    route: Route,
    expected: PeerPublicKey,
    path: PathBuf,
) -> Result<TransferReceipt> {
    let peer = validate_peer(config, expected)?;
    let (dial, relay) = match &route {
        Route::Direct(address) => {
            validate_address(address)?;
            (address.clone(), None)
        }
        Route::Relay { address, key } => {
            validate_address(address)?;
            let relay = relay_peer(config, expected, *key)?;
            (
                circuit(address, relay).with(Protocol::P2p(peer)),
                Some(relay),
            )
        }
    };
    let relayed = relay.is_some();
    let (mut file, details) = open_source(&path).await?;
    if details.len() > MAX_FILE_BYTES {
        bail!("direct-file source must be a regular file of at most 1 GiB");
    }
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("invalid direct-file source name"))?
        .to_owned();
    if filename.len() > 255
        || filename.contains(['/', '\\', '\0'])
        || filename.chars().any(char::is_control)
    {
        bail!("invalid direct-file source name");
    }
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; CHUNK_LEN];
    let mut hashed = 0u64;
    loop {
        let len = file.read(&mut buffer).await?;
        if len == 0 {
            break;
        }
        hashed = hashed
            .checked_add(len as u64)
            .filter(|len| *len <= details.len())
            .ok_or_else(|| anyhow!("direct-file source changed while hashing"))?;
        hasher.update(&buffer[..len]);
    }
    if hashed != details.len() {
        bail!("direct-file source changed while hashing");
    }
    file.seek(io::SeekFrom::Start(0)).await?;
    let metadata = FileMetadata::new(filename, hashed, hasher.finalize().into());
    let mut swarm = swarm(config, false, relayed)?;
    let result: Result<TransferReceipt> = async {
    swarm.dial(DialOpts::peer_id(peer).addresses(vec![dial]).build())?;
    let mut initiator = Initiator::new(&config.identity, &config.trusted_peers, &expected)?;
    let session = timeout(config.handshake_timeout, async {
        let response = exchange(&mut swarm, peer, initiator.init_message().to_vec()).await?;
        let finish = initiator.handle_response(&response)?;
        let confirm = exchange(&mut swarm, peer, finish).await?;
        Ok::<_, anyhow::Error>(Session::from_handshake(initiator.handle_confirm(&confirm)?))
    })
    .await
    .context("direct-file application handshake timed out")??;
    log::info!(
        "event=direct_file_authenticated role=initiator peer={peer} route={}",
        route_label(relayed)
    );
    let mut sequence = 0u64;
    let record = session.seal(
        STREAM_ID,
        RECORD_TYPE,
        0,
        &payload(META, &metadata.serialize()),
    )?;
    let response = exchange(&mut swarm, peer, record).await?;
    check_reply(&session, &response, sequence, 0, &metadata)?;
    let mut sent = 0u64;
    while sent < metadata.size {
        let max = ((metadata.size - sent) as usize).min(CHUNK_LEN);
        let len = file.read(&mut buffer[..max]).await?;
        if len == 0 {
            bail!("direct-file source truncated during transfer");
        }
        sequence += 1;
        let mut chunk = sent.to_be_bytes().to_vec();
        chunk.extend_from_slice(&buffer[..len]);
        let record = session.seal(STREAM_ID, RECORD_TYPE, 0, &payload(CHUNK, &chunk))?;
        let response = exchange(&mut swarm, peer, record).await?;
        sent += len as u64;
        check_reply(&session, &response, sequence, sent, &metadata)?;
    }
    // The receiver remains alive until this close, so it cannot abandon the
    // reliable transport while its final acknowledgment is still in flight.
    let _ = swarm.disconnect_peer_id(peer);
    let _ = timeout(IO_TIMEOUT, async {
        loop {
            if matches!(swarm.select_next_some().await, SwarmEvent::ConnectionClosed { peer_id, .. } if peer_id == peer) { break; }
        }
    }).await;
    Ok(TransferReceipt {
        bytes: metadata.size,
        sha256: metadata.hash,
        peer: expected,
        relayed,
    })
    }
    .await;
    if relayed {
        close_all(&mut swarm).await;
    }
    result
}

/// Close every connection and let the transport signal it, so a relay drops
/// this peer's reservation at once. Otherwise a QUIC connection to the relay
/// stays open on the relay's side until its idle timeout (10 s), and the relay
/// may route the next circuit for this identity into it. Bounded to 2 s.
async fn close_all(swarm: &mut Swarm<Behaviour>) {
    let peers: Vec<PeerId> = swarm.connected_peers().copied().collect();
    for peer in peers {
        let _ = swarm.disconnect_peer_id(peer);
    }
    let _ = timeout(Duration::from_secs(2), async {
        while swarm.connected_peers().next().is_some() {
            swarm.select_next_some().await;
        }
    })
    .await;
}

/// Explicit one-shot listener accepting only `expected`, which must already be
/// pinned. The existing, owner-controlled output directory must not be a symlink.
/// Dropping this future cancels the transfer and cleans any unpublished partial.
pub async fn receive_file(
    config: &TunnelConfig,
    address: Multiaddr,
    expected: PeerPublicKey,
    output_dir: PathBuf,
) -> Result<TransferReceipt> {
    receive_inner(config, address, expected, output_dir, None).await
}

/// [`receive_file`] over an explicit [`Route`]. With [`Route::Relay`], the
/// receiver makes a reservation on that relay and accepts only the expected
/// peer's circuit through it; it opens no direct listener.
pub async fn receive_file_routed(
    config: &TunnelConfig,
    route: Route,
    expected: PeerPublicKey,
    output_dir: PathBuf,
) -> Result<TransferReceipt> {
    receive_routed(config, route, expected, output_dir, None).await
}

async fn receive_inner(
    config: &TunnelConfig,
    address: Multiaddr,
    expected: PeerPublicKey,
    output_dir: PathBuf,
    ready: Option<tokio::sync::oneshot::Sender<Multiaddr>>,
) -> Result<TransferReceipt> {
    receive_routed(config, Route::Direct(address), expected, output_dir, ready).await
}

async fn receive_routed(
    config: &TunnelConfig,
    route: Route,
    expected: PeerPublicKey,
    output_dir: PathBuf,
    ready: Option<tokio::sync::oneshot::Sender<Multiaddr>>,
) -> Result<TransferReceipt> {
    let peer = validate_peer(config, expected)?;
    let (listen, relay) = match &route {
        Route::Direct(address) => {
            validate_address(address)?;
            (address.clone(), None)
        }
        Route::Relay { address, key } => {
            validate_address(address)?;
            let relay = relay_peer(config, expected, *key)?;
            (circuit(address, relay), Some(relay))
        }
    };
    let relayed = relay.is_some();
    let output = std::fs::symlink_metadata(&output_dir)
        .context("direct-file output directory must already exist")?;
    if !output.is_dir() || output.file_type().is_symlink() {
        bail!("direct-file output must be an existing directory, not a symlink");
    }
    let mut swarm = swarm(config, true, relayed)?;
    let result: Result<TransferReceipt> = async {
        swarm.listen_on(listen)?;
        let mut ready = ready;
        let connection: ConnectionId = timeout(LISTEN_TIMEOUT, async {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        log::info!("event=direct_file_listening address={address}");
                        if let Some(ready) = ready.take() {
                            let _ = ready.send(address);
                        }
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        endpoint,
                        ..
                    } if peer_id == peer => {
                        // The peer must arrive over the selected route only.
                        if through_relay(&endpoint) == relayed {
                            return Ok(connection_id);
                        }
                        let _ = swarm.close_connection(connection_id);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } if Some(peer_id) == relay => {
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let _ = swarm.disconnect_peer_id(peer_id);
                    }
                    SwarmEvent::Behaviour(Event::Relay(
                        relay::client::Event::ReservationReqAccepted { .. },
                    )) => log::info!("event=direct_file_relay_reserved"),
                    SwarmEvent::ListenerError { .. } | SwarmEvent::ListenerClosed { .. } => {
                        bail!("direct-file listener failed")
                    }
                    _ => {}
                }
            }
        })
        .await
        .context("direct-file listener timed out waiting for the expected peer")??;
        let session = timeout(config.handshake_timeout, async {
            let mut responder = Responder::new(&config.identity);
            let request = next_request(&mut swarm, peer, relay).await?;
            if request.bytes.len() > MAX_MESSAGE_LEN {
                bail!("oversized direct-file handshake");
            }
            let response = responder.handle_init(&request.bytes, &config.trusted_peers)?;
            if responder.claimed_peer() != Some(expected) {
                bail!("direct-file transport/application identity mismatch");
            }
            reply(&mut swarm, request, response)?;
            let request = next_request(&mut swarm, peer, relay).await?;
            let (confirm, keys) = responder.handle_finish(&request.bytes)?;
            reply(&mut swarm, request, confirm)?;
            Ok::<_, anyhow::Error>(Session::from_handshake(keys))
        })
        .await
        .context("direct-file application handshake timed out")??;
        log::info!(
            "event=direct_file_authenticated role=responder peer={peer} route={}",
            route_label(relayed)
        );
        let mut receiver = Receiver::new(output_dir);
        loop {
            let request = next_request(&mut swarm, peer, relay).await?;
            let response = receiver.record(&session, &request.bytes)?;
            let final_id = request.id;
            reply(&mut swarm, request, response)?;
            if receiver.complete {
                // Drive response flushing and wait for the sender, which closes only
                // after authenticating the final reply. A transport failure remains
                // a reported failure even if a verified file was already published.
                timeout(IO_TIMEOUT, async {
                    let mut flushed = false;
                    loop {
                        match swarm.select_next_some().await {
                            SwarmEvent::Behaviour(Event::Files(
                                request_response::Event::ResponseSent { request_id, .. },
                            )) if request_id == final_id => flushed = true,
                            SwarmEvent::Behaviour(Event::Files(
                                request_response::Event::InboundFailure { request_id, .. },
                            )) if request_id == final_id => {
                                bail!("direct-file completion reply failed")
                            }
                            SwarmEvent::ConnectionClosed { connection_id, .. }
                                if connection_id == connection =>
                            {
                                if !flushed {
                                    bail!(
                                    "direct-file peer closed before completion reply was flushed"
                                );
                                }
                                return receiver.receipt(expected, relayed);
                            }
                            _ => {}
                        }
                    }
                })
                .await
                .context(
                    "verified file published, but completion acknowledgment was not confirmed",
                )??;
                return receiver.receipt(expected, relayed);
            }
        }
    }
    .await;
    // Any unpublished partial was already removed when the receiver state was
    // dropped at the end of the block above.
    if relayed {
        close_all(&mut swarm).await;
    }
    result
}
