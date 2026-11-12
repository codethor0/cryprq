// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // Tests can use unwrap, expect and panic
mod tunnel_tests {
    use crate::auth_handshake::{Initiator, Responder, SessionKeys};
    use crate::identity::test_support::ephemeral_identity;
    use crate::{
        accept_tunnel, connect_tunnel, create_test_mode_tunnel, create_tunnel, epoch_keys,
        NodeIdentity, PeerPublicKey, Session, TrustedPeers, Tunnel, TunnelConfig, TunnelError,
        TunnelRole, MAX_NONCE_VALUE, TEST_MODE_ENV, VPN_STREAM_ID,
    };
    use cryprq_core::{
        MSG_TYPE_CONTROL, MSG_TYPE_DATA, MSG_TYPE_FILE_ACK, MSG_TYPE_FILE_CHUNK,
        MSG_TYPE_FILE_META, MSG_TYPE_VPN_PACKET, RECORD_HEADER_SIZE,
    };
    use std::collections::HashSet;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::net::UdpSocket;
    use tokio::time::timeout;

    // ---------------------------------------------------------------------
    // Session-level helpers. These run the authenticated handshake in memory,
    // so the nonce, replay, rekey and key-isolation tests never depend on the
    // network and are never skipped.
    //
    // This module is a child of the crate root, so it reads and sets the
    // private session state directly. No test-only hook exists in production
    // code for moving a sequence counter or an epoch.
    // ---------------------------------------------------------------------

    /// Runs a complete authenticated handshake between two fresh identities.
    fn handshake() -> (SessionKeys, SessionKeys) {
        let alice = ephemeral_identity();
        let bob = ephemeral_identity();
        let alice_trust = TrustedPeers::from_keys([bob.public_key()]);
        let bob_trust = TrustedPeers::from_keys([alice.public_key()]);

        let mut initiator =
            Initiator::new(&alice, &alice_trust, &bob.public_key()).expect("initiator");
        let mut responder = Responder::new(&bob);
        let init = initiator.init_message().to_vec();
        let response = responder.handle_init(&init, &bob_trust).expect("init");
        let finish = initiator.handle_response(&response).expect("response");
        let (confirm, responder_keys) = responder.handle_finish(&finish).expect("finish");
        let initiator_keys = initiator.handle_confirm(&confirm).expect("confirm");
        (initiator_keys, responder_keys)
    }

    fn session_from(keys: SessionKeys) -> Session {
        Session::new(
            keys.role.into(),
            keys.master_secret,
            Some(keys.peer),
            keys.session_id,
        )
    }

    /// (initiator session, responder session) of one handshake.
    fn session_pair() -> (Session, Session) {
        let (initiator_keys, responder_keys) = handshake();
        (session_from(initiator_keys), session_from(responder_keys))
    }

    fn seal(session: &Session, payload: &[u8]) -> Vec<u8> {
        session
            .seal(VPN_STREAM_ID, MSG_TYPE_VPN_PACKET, 0, payload)
            .expect("seal")
    }

    /// (epoch, next sequence number, key, iv) of the sending direction.
    fn send_snapshot(session: &Session) -> (u8, u64, [u8; 32], [u8; 12]) {
        let send = session.send.lock().expect("send lock");
        (send.epoch, send.next_seq, send.keys.key, send.keys.iv)
    }

    /// (epoch, key, iv) of the current receiving epoch.
    fn recv_snapshot(session: &Session) -> (u8, [u8; 32], [u8; 12]) {
        let recv = session.recv.lock().expect("recv lock");
        (
            recv.current.epoch,
            recv.current.keys.key,
            recv.current.keys.iv,
        )
    }

    fn set_next_seq(session: &Session, next_seq: u64) {
        session.send.lock().expect("send lock").next_seq = next_seq;
    }

    /// Puts the sending direction at `epoch` with that epoch's real keys.
    fn jump_send_epoch(session: &Session, epoch: u8) {
        let (keys, _) = epoch_keys(&session.master_secret, epoch, session.role);
        let mut send = session.send.lock().expect("send lock");
        send.epoch = epoch;
        send.keys = keys;
        send.next_seq = 0;
    }

    // ---------------------------------------------------------------------
    // Nonce exhaustion
    // ---------------------------------------------------------------------

    #[test]
    fn test_max_nonce_value_constant() {
        assert_eq!(MAX_NONCE_VALUE, u64::MAX - 1000);
    }

    #[test]
    fn sequence_numbers_are_consumed_once_and_shared_by_all_message_types() {
        let (a, b) = session_pair();
        let types = [
            MSG_TYPE_DATA,
            MSG_TYPE_FILE_META,
            MSG_TYPE_FILE_CHUNK,
            MSG_TYPE_FILE_ACK,
            MSG_TYPE_VPN_PACKET,
            MSG_TYPE_CONTROL,
        ];
        for (expected_seq, message_type) in types.into_iter().enumerate() {
            // Different streams and types must not get their own counters: the
            // nonce depends on the sequence number alone.
            let record = a
                .seal(expected_seq as u32 + 1, message_type, 0, b"payload")
                .expect("seal");
            let (header, plaintext) = b.open(&record).expect("open");
            assert_eq!(header.sequence_number, expected_seq as u64);
            assert_eq!(header.message_type, message_type);
            assert_eq!(header.epoch, 0);
            assert_eq!(plaintext, b"payload");
        }
        assert_eq!(send_snapshot(&a).1, types.len() as u64);
    }

    #[test]
    fn nonce_exhaustion_refuses_to_seal_instead_of_reusing_a_nonce() {
        let (a, b) = session_pair();

        // The last sequence number below the limit is still usable.
        set_next_seq(&a, MAX_NONCE_VALUE - 1);
        let last = seal(&a, b"last record of the epoch");
        let (header, plaintext) = b.open(&last).expect("open last");
        assert_eq!(header.sequence_number, MAX_NONCE_VALUE - 1);
        assert_eq!(plaintext, b"last record of the epoch");

        // At the limit every send fails, for every message type, and the
        // counter neither advances nor wraps.
        for message_type in [
            MSG_TYPE_DATA,
            MSG_TYPE_FILE_META,
            MSG_TYPE_FILE_CHUNK,
            MSG_TYPE_FILE_ACK,
            MSG_TYPE_VPN_PACKET,
            MSG_TYPE_CONTROL,
        ] {
            let result = a.seal(VPN_STREAM_ID, message_type, 0, b"test");
            assert!(
                matches!(result, Err(TunnelError::NonceOverflow)),
                "expected NonceOverflow for type {message_type:#04x}, got {result:?}"
            );
            assert_eq!(send_snapshot(&a).1, MAX_NONCE_VALUE);
        }

        // Counters beyond the limit (which only a bug could produce) fail too.
        for beyond in [MAX_NONCE_VALUE + 1, u64::MAX - 1, u64::MAX] {
            set_next_seq(&a, beyond);
            assert!(matches!(
                a.seal(VPN_STREAM_ID, MSG_TYPE_VPN_PACKET, 0, b"test"),
                Err(TunnelError::NonceOverflow)
            ));
            assert_eq!(send_snapshot(&a).1, beyond);
        }
    }

    #[test]
    fn rekey_after_nonce_exhaustion_restores_sending_under_a_new_key() {
        let (a, b) = session_pair();
        let (_, _, exhausted_key, exhausted_iv) = send_snapshot(&a);
        set_next_seq(&a, MAX_NONCE_VALUE);
        assert!(matches!(
            a.seal(VPN_STREAM_ID, MSG_TYPE_VPN_PACKET, 0, b"blocked"),
            Err(TunnelError::NonceOverflow)
        ));

        assert!(matches!(a.rotate_send_keys(), Ok(1)));
        let (epoch, next_seq, key, iv) = send_snapshot(&a);
        assert_eq!((epoch, next_seq), (1, 0));
        assert_ne!(key, exhausted_key, "sequence space reset without a new key");
        assert_ne!(iv, exhausted_iv);

        let record = seal(&a, b"after rekey");
        let (header, plaintext) = b.open(&record).expect("open after rekey");
        assert_eq!((header.epoch, header.sequence_number), (1, 0));
        assert_eq!(plaintext, b"after rekey");
    }

    #[test]
    fn epoch_exhaustion_expires_the_session_instead_of_wrapping() {
        let (a, b) = session_pair();
        jump_send_epoch(&a, u8::MAX - 1);

        assert!(matches!(a.rotate_send_keys(), Ok(u8::MAX)));
        let record = seal(&a, b"last epoch");
        let (header, _) = b.open(&record).expect("open last epoch");
        assert_eq!(header.epoch, u8::MAX);

        // There is no epoch after 255: rotation must not wrap to epoch 0 and
        // reuse its keys with a fresh sequence space.
        for _ in 0..3 {
            assert!(matches!(
                a.rotate_send_keys(),
                Err(TunnelError::SessionExpired)
            ));
            assert!(matches!(
                a.seal(VPN_STREAM_ID, MSG_TYPE_VPN_PACKET, 0, b"expired"),
                Err(TunnelError::SessionExpired)
            ));
        }
        let (epoch, next_seq, _, _) = send_snapshot(&a);
        assert_eq!(epoch, u8::MAX);
        assert_eq!(next_seq, 1, "an expired session must not consume nonces");

        // The receiving direction of the same session is unaffected.
        let reply = seal(&b, b"reply");
        assert_eq!(a.open(&reply).expect("open reply").1, b"reply");
    }

    // ---------------------------------------------------------------------
    // Key isolation
    // ---------------------------------------------------------------------

    #[test]
    fn traffic_keys_are_separated_by_direction() {
        let (initiator_keys, responder_keys) = handshake();
        let master = *initiator_keys.master_secret;
        let (a, b) = (session_from(initiator_keys), session_from(responder_keys));
        assert_eq!(a.role, TunnelRole::Initiator);
        assert_eq!(b.role, TunnelRole::Responder);

        let (_, _, a_send_key, a_send_iv) = send_snapshot(&a);
        let (_, a_recv_key, a_recv_iv) = recv_snapshot(&a);
        let (_, _, b_send_key, b_send_iv) = send_snapshot(&b);
        let (_, b_recv_key, b_recv_iv) = recv_snapshot(&b);

        // Each side sends with what the other receives with.
        assert_eq!((a_send_key, a_send_iv), (b_recv_key, b_recv_iv));
        assert_eq!((b_send_key, b_send_iv), (a_recv_key, a_recv_iv));
        // The two directions never share a key or an IV.
        assert_ne!(a_send_key, a_recv_key);
        assert_ne!(a_send_iv, a_recv_iv);
        // Keys are derived, not zero and not the master secret itself.
        for key in [a_send_key, a_recv_key] {
            assert_ne!(key, [0u8; 32]);
            assert_ne!(key, master);
        }
        assert_ne!(a_send_iv, [0u8; 12]);
        assert_ne!(a_recv_iv, [0u8; 12]);

        // A record reflected back to its sender does not authenticate.
        let record = seal(&a, b"to the responder");
        assert!(matches!(
            a.open(&record),
            Err(TunnelError::DecryptionFailed)
        ));
        assert_eq!(b.open(&record).expect("open").1, b"to the responder");
    }

    #[test]
    fn test_key_uniqueness() {
        // Independent sessions must not share traffic keys or identifiers.
        let (a1, b1) = session_pair();
        let (a2, b2) = session_pair();
        assert_ne!(*a1.master_secret, *a2.master_secret);
        assert_ne!(a1.session_id, a2.session_id);
        assert_ne!(send_snapshot(&a1).2, send_snapshot(&a2).2);
        assert_ne!(send_snapshot(&a1).3, send_snapshot(&a2).3);
        assert_ne!(recv_snapshot(&a1).1, recv_snapshot(&a2).1);

        // A record of one session is rejected by every party of the other.
        let record = seal(&a1, b"session one");
        assert!(matches!(
            b2.open(&record),
            Err(TunnelError::DecryptionFailed)
        ));
        assert!(matches!(
            a2.open(&record),
            Err(TunnelError::DecryptionFailed)
        ));
        assert_eq!(b1.open(&record).expect("open").1, b"session one");
    }

    #[test]
    fn every_epoch_and_direction_has_its_own_key_and_iv() {
        let (initiator_keys, _) = handshake();
        let master = *initiator_keys.master_secret;
        let mut keys = HashSet::new();
        let mut ivs = HashSet::new();
        for epoch in 0..=u8::MAX {
            let (send, recv) = epoch_keys(&master, epoch, TunnelRole::Initiator);
            let (peer_send, peer_recv) = epoch_keys(&master, epoch, TunnelRole::Responder);
            assert_eq!((send.key, send.iv), (peer_recv.key, peer_recv.iv));
            assert_eq!((recv.key, recv.iv), (peer_send.key, peer_send.iv));
            for direction in [&send, &recv] {
                assert!(keys.insert(direction.key), "key reused at epoch {epoch}");
                assert!(ivs.insert(direction.iv), "iv reused at epoch {epoch}");
            }
        }
        assert_eq!(keys.len(), 512);
        assert!(!keys.contains(&master));
    }

    // ---------------------------------------------------------------------
    // Replay protection and record authentication on a session
    // ---------------------------------------------------------------------

    #[test]
    fn replayed_and_reordered_records_are_handled_per_session() {
        let (a, b) = session_pair();
        let records: Vec<Vec<u8>> = (0..5u8).map(|i| seal(&a, &[i])).collect();

        // Reordered delivery is accepted exactly once per record.
        for index in [3usize, 0, 4, 1, 2] {
            let (header, plaintext) = b.open(&records[index]).expect("first delivery");
            assert_eq!(header.sequence_number, index as u64);
            assert_eq!(plaintext, [index as u8]);
        }
        for record in &records {
            assert!(matches!(b.open(record), Err(TunnelError::ReplayDetected)));
        }
    }

    #[test]
    fn records_older_than_the_replay_window_are_rejected() {
        let (a, b) = session_pair();
        let old = seal(&a, b"old");
        set_next_seq(&a, crate::REPLAY_WINDOW_SIZE as u64 + 10);
        let recent = seal(&a, b"recent");
        b.open(&recent).expect("open recent");
        // Never seen, but too old to be tracked: must be refused.
        assert!(matches!(b.open(&old), Err(TunnelError::ReplayDetected)));
    }

    #[test]
    fn any_modified_or_truncated_record_is_rejected_without_changing_state() {
        let (a, b) = session_pair();
        let record = seal(&a, b"authentic payload");
        assert!(record.len() > RECORD_HEADER_SIZE);

        // Every byte of the header and of the ciphertext is authenticated.
        for index in 0..record.len() {
            let mut forged = record.clone();
            forged[index] ^= 0x01;
            assert!(
                b.open(&forged).is_err(),
                "flipping a bit in byte {index} was accepted"
            );
        }
        for len in 0..record.len() {
            assert!(
                b.open(&record[..len]).is_err(),
                "truncation to {len} accepted"
            );
        }
        let mut extended = record.clone();
        extended.push(0);
        assert!(b.open(&extended).is_err());

        // None of the forgeries advanced the epoch or the replay window.
        {
            let recv = b.recv.lock().expect("recv lock");
            assert_eq!(recv.current.epoch, 0);
            assert!(recv.previous.is_none());
        }
        assert_eq!(b.open(&record).expect("open").1, b"authentic payload");
    }

    #[test]
    fn large_and_empty_payloads_round_trip() {
        let (a, b) = session_pair();
        let empty = seal(&a, &[]);
        assert!(b.open(&empty).expect("open empty").1.is_empty());

        let large = vec![0xAA; 60000];
        let record = seal(&a, &large);
        assert_eq!(b.open(&record).expect("open large").1, large);
    }

    // ---------------------------------------------------------------------
    // Key rotation
    // ---------------------------------------------------------------------

    #[test]
    fn key_rotation_moves_one_direction_to_fresh_keys() {
        let (a, b) = session_pair();
        let (_, _, old_key, old_iv) = send_snapshot(&a);
        let early = seal(&a, b"early");
        let delayed = seal(&a, b"delayed");
        let late = seal(&a, b"late");
        b.open(&early).expect("open early");

        assert!(matches!(a.rotate_send_keys(), Ok(1)));
        let (epoch, next_seq, new_key, new_iv) = send_snapshot(&a);
        assert_eq!((epoch, next_seq), (1, 0));
        assert_ne!(new_key, old_key);
        assert_ne!(new_iv, old_iv);

        // The receiver follows on the first authentic record of the new epoch.
        assert_eq!(recv_snapshot(&b).0, 0);
        let fresh = seal(&a, b"fresh");
        let (header, plaintext) = b.open(&fresh).expect("open fresh");
        assert_eq!((header.epoch, header.sequence_number), (1, 0));
        assert_eq!(plaintext, b"fresh");
        let (recv_epoch, recv_key, _) = recv_snapshot(&b);
        assert_eq!(recv_epoch, 1);
        assert_eq!(recv_key, new_key);

        // A reordered record of the previous epoch is still accepted once...
        assert_eq!(b.open(&delayed).expect("open delayed").1, b"delayed");
        // ...and replay protection covers both epochs.
        for replay in [&early, &delayed, &fresh] {
            assert!(matches!(b.open(replay), Err(TunnelError::ReplayDetected)));
        }

        // Once the grace period is over the old keys are gone for good.
        b.recv
            .lock()
            .expect("recv lock")
            .previous
            .as_mut()
            .expect("previous epoch is kept")
            .1 = Instant::now();
        assert!(matches!(b.open(&late), Err(TunnelError::DecryptionFailed)));
        assert!(b.recv.lock().expect("recv lock").previous.is_none());

        // Rotation is per direction: the other direction is untouched.
        assert_eq!(send_snapshot(&b).0, 0);
        let reply = seal(&b, b"reply");
        let (header, plaintext) = a.open(&reply).expect("open reply");
        assert_eq!(header.epoch, 0);
        assert_eq!(plaintext, b"reply");
        assert_eq!(recv_snapshot(&a).0, 0);
    }

    #[test]
    fn an_epoch_left_behind_is_never_accepted_again() {
        let (a, b) = session_pair();
        let epoch0 = seal(&a, b"epoch 0");
        assert!(matches!(a.rotate_send_keys(), Ok(1)));
        let epoch1 = seal(&a, b"epoch 1");
        let epoch1_delayed = seal(&a, b"epoch 1 delayed");
        assert!(matches!(a.rotate_send_keys(), Ok(2)));
        let epoch2 = seal(&a, b"epoch 2");
        assert!(matches!(a.rotate_send_keys(), Ok(3)));
        let epoch3 = seal(&a, b"epoch 3");

        b.open(&epoch1).expect("open epoch 1");
        // The receiver may skip epochs; the sender proves each one with a record.
        b.open(&epoch3).expect("open epoch 3");
        assert_eq!(recv_snapshot(&b).0, 3);

        // Only the directly superseded epoch (1) is kept for reordering.
        assert_eq!(
            b.open(&epoch1_delayed).expect("open delayed").1,
            b"epoch 1 delayed"
        );
        for stale in [&epoch0, &epoch2] {
            assert!(matches!(b.open(stale), Err(TunnelError::DecryptionFailed)));
        }
        assert_eq!(recv_snapshot(&b).0, 3);
    }

    #[test]
    fn a_forged_epoch_change_does_not_move_the_receiver() {
        let (a, b) = session_pair();
        let record = seal(&a, b"genuine");
        let (_, key_before, iv_before) = recv_snapshot(&b);

        for claimed_epoch in 1..=u8::MAX {
            let mut forged = record.clone();
            forged[3] = claimed_epoch;
            assert!(matches!(
                b.open(&forged),
                Err(TunnelError::DecryptionFailed)
            ));
        }
        assert_eq!(recv_snapshot(&b), (0, key_before, iv_before));
        assert!(b.recv.lock().expect("recv lock").previous.is_none());
        assert_eq!(b.open(&record).expect("open").1, b"genuine");
    }

    #[test]
    fn records_sealed_before_and_after_rotation_never_share_key_and_nonce() {
        // Same sequence number, same plaintext, consecutive epochs: with a
        // reused (key, nonce) pair the ciphertexts would be identical.
        let (a, _b) = session_pair();
        let before = seal(&a, b"same plaintext");
        assert!(matches!(a.rotate_send_keys(), Ok(1)));
        let after = seal(&a, b"same plaintext");
        assert_eq!(before[8..16], after[8..16], "both use sequence number 0");
        assert_ne!(before[RECORD_HEADER_SIZE..], after[RECORD_HEADER_SIZE..]);
    }

    // ---------------------------------------------------------------------
    // Authenticated tunnels over loopback UDP
    // ---------------------------------------------------------------------

    const IO_TIMEOUT: Duration = Duration::from_secs(10);

    /// Binds a loopback socket.
    ///
    /// A denied bind fails the test: a tunnel test that could not open its
    /// socket has not run and must not count as passed. Only with
    /// `CRYPRQ_ALLOW_UDP_SKIP=1`, for sandboxes without UDP, does it return
    /// `None` and let the caller skip, which is then printed.
    async fn bind_loopback() -> Option<UdpSocket> {
        match UdpSocket::bind("127.0.0.1:0").await {
            Ok(socket) => Some(socket),
            Err(e)
                if e.kind() == std::io::ErrorKind::PermissionDenied
                    && std::env::var("CRYPRQ_ALLOW_UDP_SKIP").as_deref() == Ok("1") =>
            {
                eprintln!("SKIP: UDP bind denied in this environment");
                None
            }
            Err(e) => panic!("cannot bind a loopback UDP socket: {e:?}"),
        }
    }

    fn config<I>(identity: NodeIdentity, trusted: I, handshake_timeout: Duration) -> TunnelConfig
    where
        I: IntoIterator<Item = PeerPublicKey>,
    {
        let mut config = TunnelConfig::new(Arc::new(identity), TrustedPeers::from_keys(trusted));
        config.handshake_timeout = handshake_timeout;
        config
    }

    struct TunnelPair {
        initiator: Tunnel,
        responder: Tunnel,
        initiator_id: PeerPublicKey,
        responder_id: PeerPublicKey,
        initiator_addr: SocketAddr,
        responder_addr: SocketAddr,
    }

    /// Two mutually pinned peers with an established tunnel between them.
    async fn tunnel_pair() -> Option<TunnelPair> {
        let alice = ephemeral_identity();
        let bob = ephemeral_identity();
        let (initiator_id, responder_id) = (alice.public_key(), bob.public_key());
        let alice_config = config(alice, [responder_id], IO_TIMEOUT);
        let bob_config = config(bob, [initiator_id], IO_TIMEOUT);

        let alice_socket = bind_loopback().await?;
        let bob_socket = bind_loopback().await?;
        let initiator_addr = alice_socket.local_addr().expect("local addr");
        let responder_addr = bob_socket.local_addr().expect("local addr");

        let (accepted, connected) = tokio::join!(
            timeout(IO_TIMEOUT, accept_tunnel(&bob_config, bob_socket)),
            connect_tunnel(&alice_config, alice_socket, responder_addr, &responder_id),
        );
        Some(TunnelPair {
            initiator: connected.expect("connect"),
            responder: accepted.expect("accept in time").expect("accept"),
            initiator_id,
            responder_id,
            initiator_addr,
            responder_addr,
        })
    }

    async fn recv(tunnel: &Tunnel) -> Result<(u8, u32, Vec<u8>), TunnelError> {
        timeout(IO_TIMEOUT, tunnel.recv_record())
            .await
            .expect("record arrives in time")
    }

    #[tokio::test]
    async fn test_tunnel_creation_with_real_handshake() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };

        // Roles and mutually authenticated identities.
        assert_eq!(pair.initiator.role(), TunnelRole::Initiator);
        assert_eq!(pair.responder.role(), TunnelRole::Responder);
        assert_eq!(pair.initiator.peer_identity(), Some(pair.responder_id));
        assert_eq!(pair.responder.peer_identity(), Some(pair.initiator_id));
        assert_eq!(pair.initiator.session_id(), pair.responder.session_id());
        assert_ne!(pair.initiator.session_id(), [0u8; 16]);

        // Keys are non-zero, differ by direction and match across the peers.
        let (_, _, out_key, out_iv) = send_snapshot(&pair.initiator.session);
        let (_, in_key, in_iv) = recv_snapshot(&pair.initiator.session);
        assert_ne!(out_key, [0u8; 32], "Outbound key should not be zero");
        assert_ne!(out_iv, [0u8; 12], "Outbound IV should not be zero");
        assert_ne!(in_key, [0u8; 32], "Inbound key should not be zero");
        assert_ne!(in_iv, [0u8; 12], "Inbound IV should not be zero");
        assert_ne!(out_key, in_key, "Outbound and inbound keys should differ");
        let (_, peer_in_key, peer_in_iv) = recv_snapshot(&pair.responder.session);
        assert_eq!((out_key, out_iv), (peer_in_key, peer_in_iv));
    }

    #[tokio::test]
    async fn test_tunnel_send_packet() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };

        // Both directions, including an empty and an MTU-sized packet.
        let payloads: [Vec<u8>; 3] = [b"test packet data".to_vec(), Vec::new(), vec![0xAA; 1200]];
        for payload in &payloads {
            pair.initiator
                .send_packet(payload)
                .await
                .expect("initiator send");
            let (message_type, stream_id, received) = recv(&pair.responder).await.expect("recv");
            assert_eq!(message_type, MSG_TYPE_VPN_PACKET);
            assert_eq!(stream_id, VPN_STREAM_ID);
            assert_eq!(&received, payload);

            pair.responder
                .send_record(7, MSG_TYPE_DATA, 0, payload)
                .await
                .expect("responder send");
            let (message_type, stream_id, received) = recv(&pair.initiator).await.expect("recv");
            assert_eq!((message_type, stream_id), (MSG_TYPE_DATA, 7));
            assert_eq!(&received, payload);
        }
    }

    #[tokio::test]
    async fn plaintext_never_appears_on_the_wire() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };
        let secret = b"extremely recognisable plaintext marker 0123456789";
        let record = pair
            .initiator
            .session
            .seal(VPN_STREAM_ID, MSG_TYPE_VPN_PACKET, 0, secret)
            .expect("seal");
        assert!(!record
            .windows(16)
            .any(|w| secret.windows(16).any(|s| s == w)));
        // Header + ciphertext + 16-byte tag.
        assert_eq!(record.len(), RECORD_HEADER_SIZE + secret.len() + 16);
    }

    #[tokio::test]
    async fn test_nonce_overflow_protection() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };

        // Put the shared sequence counter at the limit.
        set_next_seq(&pair.initiator.session, MAX_NONCE_VALUE);
        let result = pair.initiator.send_packet(b"test").await;
        assert!(
            matches!(result, Err(TunnelError::NonceOverflow)),
            "Expected NonceOverflow, got {result:?}"
        );
        for result in [
            pair.initiator.send_file_chunk(9, b"chunk").await,
            pair.initiator.send_file_ack(9, b"ack").await,
            pair.initiator.send_vpn_packet(b"vpn").await,
        ] {
            assert!(matches!(result, Err(TunnelError::NonceOverflow)));
        }
        assert_eq!(send_snapshot(&pair.initiator.session).1, MAX_NONCE_VALUE);

        // Rotating the keys is the way out, and the peer follows.
        assert!(matches!(pair.initiator.rotate_keys(), Ok(1)));
        pair.initiator
            .send_packet(b"after rekey")
            .await
            .expect("send after rekey");
        let (_, _, received) = recv(&pair.responder).await.expect("recv after rekey");
        assert_eq!(received, b"after rekey");
        assert_eq!(recv_snapshot(&pair.responder.session).0, 1);
    }

    #[tokio::test]
    async fn key_rotation_is_followed_by_the_peer_over_the_wire() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };
        for expected_epoch in 1..=3u8 {
            assert!(matches!(pair.initiator.rotate_keys(), Ok(epoch) if epoch == expected_epoch));
            pair.initiator
                .send_packet(&[expected_epoch])
                .await
                .expect("send");
            assert_eq!(
                recv(&pair.responder).await.expect("recv").2,
                [expected_epoch]
            );
            assert_eq!(recv_snapshot(&pair.responder.session).0, expected_epoch);

            // The reverse direction keeps working on its own epoch.
            pair.responder.send_packet(b"reply").await.expect("reply");
            assert_eq!(recv(&pair.initiator).await.expect("recv reply").2, b"reply");
            assert_eq!(send_snapshot(&pair.responder.session).0, 0);
        }
    }

    #[tokio::test]
    async fn replayed_and_forged_datagrams_are_rejected_on_the_wire() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };
        let Some(attacker) = bind_loopback().await else {
            return;
        };

        let record = seal(&pair.initiator.session, b"once only");
        pair.initiator
            .socket
            .send_to(&record, pair.responder_addr)
            .await
            .expect("send");
        assert_eq!(recv(&pair.responder).await.expect("recv").2, b"once only");

        // The same datagram again, from the peer and from another address.
        for socket in [&*pair.initiator.socket, &attacker] {
            socket
                .send_to(&record, pair.responder_addr)
                .await
                .expect("replay");
            assert!(matches!(
                recv(&pair.responder).await,
                Err(TunnelError::ReplayDetected)
            ));
        }

        // A modified record and plain garbage from the attacker.
        let mut forged = seal(&pair.initiator.session, b"to be modified");
        let last = forged.len() - 1;
        forged[last] ^= 0x80;
        for datagram in [forged.as_slice(), &[0x01; 64], b"x"] {
            attacker
                .send_to(datagram, pair.responder_addr)
                .await
                .expect("forged send");
            assert!(matches!(
                recv(&pair.responder).await,
                Err(TunnelError::DecryptionFailed)
            ));
        }

        // Unauthenticated datagrams never redirect the session's traffic.
        let peer_addr = *pair.responder.peer_addr().read().expect("peer addr");
        assert_eq!(peer_addr, Some(pair.initiator_addr));
        pair.responder
            .send_packet(b"still yours")
            .await
            .expect("send");
        assert_eq!(recv(&pair.initiator).await.expect("recv").2, b"still yours");
    }

    #[tokio::test]
    async fn handshake_datagrams_are_never_delivered_as_data() {
        let Some(pair) = tunnel_pair().await else {
            return;
        };
        let echo = pair.responder.confirm_echo.as_ref().expect("confirm echo");
        let (finish, confirm) = (echo.finish.clone(), echo.confirm.clone());
        assert!(pair.initiator.confirm_echo.is_none());

        // A retransmitted FINISH is answered with the stored CONFIRM; a FINISH
        // that differs in any way is ignored. Neither surfaces as a record.
        let mut altered = finish.clone();
        let last = altered.len() - 1;
        altered[last] ^= 0x01;
        assert!(pair
            .responder
            .confirm_for_retransmitted_finish(&altered)
            .is_none());
        for datagram in [&altered, &finish] {
            pair.initiator
                .socket
                .send_to(datagram, pair.responder_addr)
                .await
                .expect("send handshake datagram");
        }
        pair.initiator
            .send_packet(b"data")
            .await
            .expect("send data");
        assert_eq!(recv(&pair.responder).await.expect("recv").2, b"data");

        // Exactly one CONFIRM came back, and the initiator skips it as well.
        pair.responder.send_packet(b"reply").await.expect("reply");
        let mut buf = vec![0u8; 2048];
        let (len, from) = timeout(IO_TIMEOUT, pair.initiator.socket.recv_from(&mut buf))
            .await
            .expect("confirm arrives")
            .expect("recv_from");
        assert_eq!(from, pair.responder_addr);
        assert_eq!(&buf[..len], confirm.as_slice());
        assert_eq!(recv(&pair.initiator).await.expect("recv reply").2, b"reply");
    }

    #[tokio::test]
    async fn an_unpinned_initiator_never_gets_a_tunnel() {
        let alice = ephemeral_identity();
        let mallory = ephemeral_identity();
        let bob = ephemeral_identity();
        let bob_id = bob.public_key();
        // Bob pins Alice only. Mallory knows Bob's key and address.
        let bob_config = config(bob, [alice.public_key()], Duration::from_secs(2));
        let mallory_config = config(mallory, [bob_id], Duration::from_millis(1200));

        let (Some(bob_socket), Some(mallory_socket)) =
            (bind_loopback().await, bind_loopback().await)
        else {
            return;
        };
        let bob_addr = bob_socket.local_addr().expect("local addr");

        let (accepted, connected) = tokio::join!(
            timeout(
                Duration::from_millis(1800),
                accept_tunnel(&bob_config, bob_socket)
            ),
            connect_tunnel(&mallory_config, mallory_socket, bob_addr, &bob_id),
        );
        assert!(
            matches!(connected, Err(TunnelError::HandshakeFailed(_))),
            "an unpinned initiator must not get a tunnel"
        );
        assert!(accepted.is_err(), "the responder must keep waiting");
    }

    #[tokio::test]
    async fn test_mismatched_identity_key_rejected() {
        // Alice pins Carol and dials what she believes is Carol's address, but
        // Bob answers there. Bob's valid signature under his own key must not
        // be accepted in place of the pinned identity.
        let alice = ephemeral_identity();
        let bob = ephemeral_identity();
        let carol = ephemeral_identity();
        let carol_id = carol.public_key();
        let bob_config = config(bob, [alice.public_key()], Duration::from_secs(2));
        let alice_config = config(alice, [carol_id], Duration::from_millis(1200));

        let (Some(bob_socket), Some(alice_socket)) = (bind_loopback().await, bind_loopback().await)
        else {
            return;
        };
        let bob_addr = bob_socket.local_addr().expect("local addr");

        let (accepted, connected) = tokio::join!(
            timeout(
                Duration::from_millis(1800),
                accept_tunnel(&bob_config, bob_socket)
            ),
            connect_tunnel(&alice_config, alice_socket, bob_addr, &carol_id),
        );
        assert!(
            matches!(connected, Err(TunnelError::HandshakeFailed(_))),
            "a responder with another identity must not be accepted"
        );
        assert!(accepted.is_err(), "no session may exist on the responder");
    }

    #[tokio::test]
    async fn test_invalid_peer_signature_rejected() {
        // A relay between the peers corrupts the responder's signature. The
        // initiator must not establish a session on the forged RESPONSE.
        let alice = ephemeral_identity();
        let bob = ephemeral_identity();
        let (alice_id, bob_id) = (alice.public_key(), bob.public_key());
        let bob_config = config(bob, [alice_id], Duration::from_secs(2));
        let alice_config = config(alice, [bob_id], Duration::from_millis(1200));

        let (Some(bob_socket), Some(alice_socket), Some(relay)) = (
            bind_loopback().await,
            bind_loopback().await,
            bind_loopback().await,
        ) else {
            return;
        };
        let bob_addr = bob_socket.local_addr().expect("local addr");
        let alice_addr = alice_socket.local_addr().expect("local addr");
        let relay_addr = relay.local_addr().expect("local addr");

        let relay_task = async {
            let mut buf = vec![0u8; 4096];
            let mut corrupted = 0u32;
            loop {
                let Ok(Ok((len, from))) =
                    timeout(Duration::from_millis(1800), relay.recv_from(&mut buf)).await
                else {
                    return corrupted;
                };
                if from == alice_addr {
                    let _ = relay.send_to(&buf[..len], bob_addr).await;
                } else {
                    // The signature is the last 64 bytes of RESPONSE.
                    buf[len - 1] ^= 0x01;
                    corrupted += 1;
                    let _ = relay.send_to(&buf[..len], alice_addr).await;
                }
            }
        };

        let (accepted, connected, corrupted) = tokio::join!(
            timeout(
                Duration::from_millis(1800),
                accept_tunnel(&bob_config, bob_socket)
            ),
            connect_tunnel(&alice_config, alice_socket, relay_addr, &bob_id),
            relay_task,
        );
        assert!(corrupted > 0, "the relay never saw a RESPONSE");
        assert!(
            matches!(connected, Err(TunnelError::HandshakeFailed(_))),
            "Should reject invalid signature"
        );
        assert!(accepted.is_err(), "no session may exist on the responder");
    }

    #[tokio::test]
    async fn connecting_to_an_unpinned_or_own_identity_is_refused_immediately() {
        let alice = ephemeral_identity();
        let alice_id = alice.public_key();
        let stranger = ephemeral_identity().public_key();
        let pinned = ephemeral_identity().public_key();
        let alice_config = config(alice, [pinned, alice_id], IO_TIMEOUT);

        for target in [stranger, alice_id] {
            let Some(socket) = bind_loopback().await else {
                return;
            };
            let addr = socket.local_addr().expect("local addr");
            let started = Instant::now();
            let result = connect_tunnel(&alice_config, socket, addr, &target).await;
            assert!(matches!(result, Err(TunnelError::InvalidPeerIdentity)));
            assert!(started.elapsed() < Duration::from_secs(2), "must not dial");
        }
    }

    #[tokio::test]
    async fn listening_without_trusted_peers_is_refused() {
        let Some(socket) = bind_loopback().await else {
            return;
        };
        let empty = config(ephemeral_identity(), [], IO_TIMEOUT);
        let result = timeout(IO_TIMEOUT, accept_tunnel(&empty, socket))
            .await
            .expect("must fail immediately");
        assert!(matches!(result, Err(TunnelError::HandshakeFailed(_))));
    }

    // ---------------------------------------------------------------------
    // Legacy fixed-key entry point
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn test_tunnel_creation() {
        if bind_loopback().await.is_none() {
            return;
        }
        // This is the only test that touches TEST_MODE_ENV, so it cannot race
        // with another test in this process.
        let saved = std::env::var_os(TEST_MODE_ENV);

        // Credentials that are not the public test placeholders are refused in
        // every mode: this entry point performs no key exchange.
        let mut peer_pk = [0u8; 32];
        peer_pk[0] = 0x30;
        peer_pk[1] = 0x40;
        for mode in ["1", "0"] {
            std::env::set_var(TEST_MODE_ENV, mode);
            let result = create_tunnel(
                &[0x10; 32],
                &peer_pk,
                &[0x55; 32],
                &[0x66; 64],
                "127.0.0.1:0",
            )
            .await;
            assert!(
                matches!(result, Err(TunnelError::HandshakeFailed(_))),
                "unauthenticated credentials must never yield a tunnel"
            );
        }

        // The placeholders are refused unless test mode is explicitly on.
        let fixed = || {
            create_tunnel(
                &[1u8; 32],
                &[2u8; 32],
                &[3u8; 32],
                &[4u8; 64],
                "127.0.0.1:0",
            )
        };
        std::env::remove_var(TEST_MODE_ENV);
        assert!(matches!(
            fixed().await,
            Err(TunnelError::InvalidPeerIdentity)
        ));
        for off in ["", "0", "false", "no", "yes", "2"] {
            std::env::set_var(TEST_MODE_ENV, off);
            assert!(
                matches!(fixed().await, Err(TunnelError::InvalidPeerIdentity)),
                "{TEST_MODE_ENV}={off:?} must not enable test mode"
            );
        }

        std::env::set_var(TEST_MODE_ENV, "1");
        let tunnel = fixed().await.expect("Tunnel creation should succeed");
        // A fixed-key tunnel authenticates nobody and says so.
        assert_eq!(tunnel.peer_identity(), None);
        tunnel
            .send_packet(b"test packet data")
            .await
            .expect("Sending packet should succeed");

        // Two test-mode processes interoperate only as initiator and responder.
        let listener = create_test_mode_tunnel(TunnelRole::Responder, "127.0.0.1:0", None)
            .await
            .expect("test-mode responder");
        let listener_addr = listener.local_addr().expect("local addr");
        *tunnel.peer_addr().write().expect("peer addr") = Some(listener_addr);
        tunnel.send_packet(b"to the listener").await.expect("send");
        assert_eq!(recv(&listener).await.expect("recv").2, b"to the listener");
        listener.send_packet(b"and back").await.expect("reply");
        assert_eq!(recv(&tunnel).await.expect("recv reply").2, b"and back");

        // The role-specific entry point is gated in the same way.
        std::env::set_var(TEST_MODE_ENV, "0");
        for role in [TunnelRole::Initiator, TunnelRole::Responder] {
            assert!(matches!(
                create_test_mode_tunnel(role, "127.0.0.1:0", None).await,
                Err(TunnelError::InvalidPeerIdentity)
            ));
        }

        match saved {
            Some(value) => std::env::set_var(TEST_MODE_ENV, value),
            None => std::env::remove_var(TEST_MODE_ENV),
        }
    }

    // ---------------------------------------------------------------------
    // Replay window, rate limiter and buffer pool units
    // ---------------------------------------------------------------------

    #[test]
    fn test_replay_window_sequential() {
        use crate::ReplayWindow;

        let mut window = ReplayWindow::new();

        // Accept sequential nonces
        assert!(window.check_and_update(1).is_ok());
        assert!(window.check_and_update(2).is_ok());
        assert!(window.check_and_update(3).is_ok());

        // Reject replay of nonce 2
        assert!(matches!(
            window.check_and_update(2),
            Err(TunnelError::ReplayDetected)
        ));
    }

    #[test]
    fn test_replay_window_out_of_order() {
        use crate::ReplayWindow;

        let mut window = ReplayWindow::new();

        // Accept out-of-order within window
        assert!(window.check_and_update(100).is_ok());
        assert!(window.check_and_update(99).is_ok());
        assert!(window.check_and_update(50).is_ok());

        // Reject replays
        assert!(matches!(
            window.check_and_update(100),
            Err(TunnelError::ReplayDetected)
        ));
        assert!(matches!(
            window.check_and_update(50),
            Err(TunnelError::ReplayDetected)
        ));
    }

    #[test]
    fn test_replay_window_old_nonces() {
        use crate::ReplayWindow;

        let mut window = ReplayWindow::new();

        // Accept nonce 3000
        assert!(window.check_and_update(3000).is_ok());

        // Nonce 900 is outside window (3000 - 2048 = 952), should be rejected
        assert!(matches!(
            window.check_and_update(900),
            Err(TunnelError::ReplayDetected)
        ));

        // Nonce 1000 is within window (3000 - 1000 = 2000 < 2048)
        assert!(window.check_and_update(1000).is_ok());
    }

    #[test]
    fn test_replay_window_large_gap() {
        use crate::ReplayWindow;

        let mut window = ReplayWindow::new();

        assert!(window.check_and_update(100).is_ok());

        // Large gap clears window
        assert!(window.check_and_update(5000).is_ok());

        // Old nonce rejected
        assert!(matches!(
            window.check_and_update(100),
            Err(TunnelError::ReplayDetected)
        ));
    }

    #[test]
    fn test_rate_limiter_basic() {
        use crate::RateLimiter;

        let mut limiter = RateLimiter::new(10, 10); // 10 pps, 10 burst

        // Should accept up to burst size
        for _ in 0..10 {
            assert!(limiter.check_and_consume().is_ok());
        }

        // Next packet should be rejected (bucket empty)
        assert!(matches!(
            limiter.check_and_consume(),
            Err(TunnelError::RateLimitExceeded)
        ));
    }

    #[test]
    fn test_rate_limiter_refill() {
        use crate::RateLimiter;
        use std::thread;
        use std::time::Duration;

        let mut limiter = RateLimiter::new(10, 5); // 10 pps, 5 burst

        // Consume all tokens
        for _ in 0..5 {
            assert!(limiter.check_and_consume().is_ok());
        }

        // Bucket empty
        assert!(matches!(
            limiter.check_and_consume(),
            Err(TunnelError::RateLimitExceeded)
        ));

        // Wait for refill (200ms = 2 tokens at 10 pps)
        thread::sleep(Duration::from_millis(200));

        // Should accept 2 packets now
        assert!(limiter.check_and_consume().is_ok());
        assert!(limiter.check_and_consume().is_ok());

        // Third should fail
        assert!(matches!(
            limiter.check_and_consume(),
            Err(TunnelError::RateLimitExceeded)
        ));
    }

    #[test]
    fn test_rate_limiter_burst_then_sustained() {
        use crate::RateLimiter;

        let mut limiter = RateLimiter::new(100, 200); // 100 pps sustained, 200 burst

        // Consume burst
        for _ in 0..200 {
            assert!(limiter.check_and_consume().is_ok());
        }

        // Exceeded
        assert!(matches!(
            limiter.check_and_consume(),
            Err(TunnelError::RateLimitExceeded)
        ));
    }

    #[test]
    fn test_buffer_pool_basic() {
        use crate::BufferPool;

        let pool = BufferPool::new(4);

        // Get buffers
        let buf1 = pool.get();
        let buf2 = pool.get();

        assert_eq!(buf1.capacity(), 65535);
        assert_eq!(buf2.capacity(), 65535);

        // Return to pool
        pool.put(buf1);
        pool.put(buf2);

        // Get again (should reuse)
        let buf3 = pool.get();
        assert_eq!(buf3.capacity(), 65535);
    }

    #[test]
    fn test_buffer_pool_reuse() {
        use crate::BufferPool;

        let pool = BufferPool::new(2);

        // Exhaust pool
        let buf1 = pool.get();
        let buf2 = pool.get();

        // Pool empty - should allocate new
        let buf3 = pool.get();
        assert_eq!(buf3.capacity(), 65535);

        // Return buffers
        pool.put(buf1);
        pool.put(buf2);
        pool.put(buf3);

        // Should have 2 in pool (capacity limit)
        let buf4 = pool.get();
        let buf5 = pool.get();
        assert_eq!(buf4.capacity(), 65535);
        assert_eq!(buf5.capacity(), 65535);
    }

    #[test]
    fn test_buffer_pool_clear_on_return() {
        use crate::BufferPool;

        let pool = BufferPool::new(2);

        let mut buf = pool.get();
        buf.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(buf.len(), 5);

        // Return should clear
        pool.put(buf);

        let buf2 = pool.get();
        assert_eq!(buf2.len(), 0); // Should be empty
    }
}
