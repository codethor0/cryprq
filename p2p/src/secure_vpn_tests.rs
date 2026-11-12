// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

// Tests can use unwrap, expect and panic.
#![allow(clippy::disallowed_methods, clippy::disallowed_macros)]

use super::*;
use rand::RngCore;
use tokio::sync::oneshot;
use tokio::time::timeout;

fn identity() -> NodeIdentity {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    NodeIdentity::from_secret_seed(&seed)
}

/// Something shaped like an IPv4 packet with a recognisable payload.
fn ip_packet(tag: &[u8]) -> Vec<u8> {
    let mut packet = vec![0u8; 20];
    packet[0] = 0x45;
    packet[9] = 17; // UDP
    packet[12..16].copy_from_slice(&[10, 0, 0, 1]);
    packet[16..20].copy_from_slice(&[10, 0, 0, 2]);
    packet.extend_from_slice(tag);
    let len = packet.len() as u16;
    packet[2..4].copy_from_slice(&len.to_be_bytes());
    packet
}

// -------------------------------------------------------------------------
// Endpoint tests. The two endpoints are connected by the test itself, which
// plays a transport that gives no protection at all: it can read, change,
// drop, replay and inject every message. Nothing here involves libp2p.
// -------------------------------------------------------------------------

/// The dialer's name for its link to the listener, and the reverse.
const TO_BOB: u8 = 1;
const FROM_ALICE: u8 = 2;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

struct Nodes {
    alice: NodeIdentity,
    bob: NodeIdentity,
    alice_trust: TrustedPeers,
    bob_trust: TrustedPeers,
}

fn nodes() -> Nodes {
    let alice = identity();
    let bob = identity();
    let alice_trust = TrustedPeers::from_keys([bob.public_key()]);
    let bob_trust = TrustedPeers::from_keys([alice.public_key()]);
    Nodes {
        alice,
        bob,
        alice_trust,
        bob_trust,
    }
}

fn dialer(n: &Nodes) -> Endpoint<'_, u8> {
    Endpoint::new(
        &n.alice,
        &n.alice_trust,
        false,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| true),
    )
}

fn listener(n: &Nodes) -> Endpoint<'_, u8> {
    Endpoint::new(
        &n.bob,
        &n.bob_trust,
        true,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| true),
    )
}

/// Runs a handshake from `dialer` to `listener`. Every message passes through
/// `transport(index, message)`, with index 0..=3 for INIT, RESPONSE, FINISH
/// and CONFIRM. Returns what both sides established, or the first refusal.
fn handshake_through(
    dialer: &mut Endpoint<'_, u8>,
    listener: &mut Endpoint<'_, u8>,
    expected: &PeerPublicKey,
    mut transport: impl FnMut(usize, Vec<u8>) -> Vec<u8>,
) -> Result<(Established<u8>, Established<u8>), String> {
    let now = Instant::now();
    let init = dialer.begin(TO_BOB, expected).map_err(|e| e.to_string())?;
    let response = match listener.on_request(FROM_ALICE, &transport(0, init), now) {
        Inbound::Reply(response) => response,
        other => return Err(format!("INIT: {other:?}")),
    };
    let finish = match dialer.on_response(TO_BOB, &transport(1, response)) {
        Ok(Progress::Send(finish)) => finish,
        other => return Err(format!("RESPONSE: {other:?}")),
    };
    let (confirm, at_listener) = match listener.on_request(FROM_ALICE, &transport(2, finish), now) {
        Inbound::Established(confirm, established) => (confirm, established),
        other => return Err(format!("FINISH: {other:?}")),
    };
    let at_dialer = match dialer.on_response(TO_BOB, &transport(3, confirm)) {
        Ok(Progress::Established(established)) => established,
        other => return Err(format!("CONFIRM: {other:?}")),
    };
    Ok((at_dialer, at_listener))
}

fn connect(
    dialer: &mut Endpoint<'_, u8>,
    listener: &mut Endpoint<'_, u8>,
    expected: &PeerPublicKey,
) {
    handshake_through(dialer, listener, expected, |_, message| message).expect("handshake");
}

fn deliver(to: &mut Endpoint<'_, u8>, link: u8, record: &[u8]) -> Result<Vec<u8>, LinkError> {
    match to.on_request(link, record, Instant::now()) {
        Inbound::Packet(packet) => Ok(packet),
        Inbound::Rejected(error) => Err(error),
        other => panic!("a record produced {other:?}"),
    }
}

#[test]
fn pinned_endpoints_establish_one_session_and_exchange_packets() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    let (at_alice, at_bob) =
        handshake_through(&mut alice, &mut bob, &n.bob.public_key(), |_, m| m).expect("handshake");

    assert_eq!(at_alice.peer, n.bob.public_key());
    assert_eq!(at_bob.peer, n.alice.public_key());
    assert_eq!(at_alice.session_id, at_bob.session_id);
    assert_eq!(at_alice.role, TunnelRole::Initiator);
    assert_eq!(at_bob.role, TunnelRole::Responder);
    assert_eq!((at_alice.replaced, at_bob.replaced), (None, None));
    assert!(!alice.handshake_in_progress());

    let outbound = ip_packet(b"from alice: a recognisable plaintext marker");
    let (link, record) = alice.seal(&outbound).expect("seal");
    assert_eq!(link, TO_BOB);
    assert!(
        !record
            .windows(16)
            .any(|w| outbound.windows(16).any(|p| p == w)),
        "the packet is visible in the record"
    );
    assert_eq!(
        deliver(&mut bob, FROM_ALICE, &record).expect("open"),
        outbound
    );

    let reply = ip_packet(b"from bob");
    let (link, record) = bob.seal(&reply).expect("seal");
    assert_eq!(link, FROM_ALICE);
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), reply);
}

#[test]
fn nothing_is_sent_or_accepted_without_a_session() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    let packet = ip_packet(b"must not leave");

    // Before any handshake, and while one is only half done.
    assert!(matches!(alice.seal(&packet), Err(LinkError::NoSession)));
    assert!(matches!(bob.seal(&packet), Err(LinkError::NoSession)));
    let init = alice.begin(TO_BOB, &n.bob.public_key()).expect("begin");
    assert!(matches!(
        bob.on_request(FROM_ALICE, &init, Instant::now()),
        Inbound::Reply(_)
    ));
    assert!(matches!(alice.seal(&packet), Err(LinkError::NoSession)));
    assert!(matches!(bob.seal(&packet), Err(LinkError::NoSession)));

    // A transport peer that skips the handshake cannot inject a packet.
    for injected in [packet.as_slice(), &[0x01; 64], b"x", &[]] {
        assert!(matches!(
            deliver(&mut bob, FROM_ALICE, injected),
            Err(LinkError::NoSession)
        ));
    }
}

#[test]
fn plain_packets_and_other_record_types_are_refused_inside_a_session() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());

    // An unencrypted IP packet on an authenticated link is not a record.
    assert!(matches!(
        deliver(&mut bob, FROM_ALICE, &ip_packet(b"plaintext")),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));

    // An authentic record that does not carry a tunnelled packet.
    let session = &alice.active.as_ref().expect("session").session;
    for (stream_id, message_type) in [
        (1u32, 0x01u8),
        (1, 0x03),
        (1, 0x10),
        (7, node::MSG_TYPE_VPN_PACKET),
    ] {
        let record = session
            .seal(stream_id, message_type, 0, b"not a packet")
            .expect("seal");
        assert!(matches!(
            deliver(&mut bob, FROM_ALICE, &record),
            Err(LinkError::Record(TunnelError::UnexpectedRecord))
        ));
    }
}

#[test]
fn tampered_records_are_rejected_and_do_not_disturb_the_session() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());

    let packet = ip_packet(b"authentic");
    let (_, record) = alice.seal(&packet).expect("seal");
    for index in 0..record.len() {
        let mut forged = record.clone();
        forged[index] ^= 0x01;
        assert!(
            deliver(&mut bob, FROM_ALICE, &forged).is_err(),
            "a bit flip in byte {index} was accepted"
        );
    }
    for len in 0..record.len() {
        assert!(deliver(&mut bob, FROM_ALICE, &record[..len]).is_err());
    }
    // The untouched record still arrives, exactly once.
    assert_eq!(
        deliver(&mut bob, FROM_ALICE, &record).expect("open"),
        packet
    );
}

#[test]
fn replayed_and_reflected_records_are_rejected() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());

    let records: Vec<Vec<u8>> = (0..4u8)
        .map(|i| alice.seal(&ip_packet(&[i])).expect("seal").1)
        .collect();
    // Out of order is fine; a second delivery is not.
    for index in [2usize, 0, 3, 1] {
        deliver(&mut bob, FROM_ALICE, &records[index]).expect("first delivery");
    }
    for record in &records {
        assert!(matches!(
            deliver(&mut bob, FROM_ALICE, record),
            Err(LinkError::Record(TunnelError::ReplayDetected))
        ));
    }
    // Sent back to its author, a record does not authenticate.
    assert!(matches!(
        deliver(&mut alice, TO_BOB, &records[0]),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));
    // Arriving over a link that is not the session's, it is not even tried.
    assert!(matches!(
        deliver(&mut bob, 99, &alice.seal(&ip_packet(b"x")).expect("seal").1),
        Err(LinkError::NoSession)
    ));
}

#[test]
fn any_modified_handshake_message_prevents_the_session() {
    let n = nodes();
    let expected = n.bob.public_key();
    let lengths = {
        let (mut alice, mut bob) = (dialer(&n), listener(&n));
        let mut lengths = [0usize; 4];
        handshake_through(&mut alice, &mut bob, &expected, |index, message| {
            lengths[index] = message.len();
            message
        })
        .expect("handshake");
        lengths
    };

    for (target, len) in lengths.into_iter().enumerate() {
        let step = (len / 61).max(1);
        for position in (0..len).step_by(step).chain([len - 1]) {
            let (mut alice, mut bob) = (dialer(&n), listener(&n));
            let result =
                handshake_through(&mut alice, &mut bob, &expected, |index, mut message| {
                    if index == target {
                        message[position] ^= 0x01;
                    }
                    message
                });
            assert!(
                result.is_err(),
                "bit flip at byte {position} of message {target} went unnoticed"
            );
            // The dialer never ends up with a session it did not verify.
            assert!(alice.active.is_none());
            assert!(matches!(
                alice.seal(&ip_packet(b"x")),
                Err(LinkError::NoSession)
            ));
            if target < 3 {
                assert!(bob.active.is_none());
            }
        }
    }
}

#[test]
fn a_recorded_handshake_cannot_be_replayed_into_a_session() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    let mut recorded = Vec::new();
    handshake_through(&mut alice, &mut bob, &n.bob.public_key(), |_, message| {
        recorded.push(message.clone());
        message
    })
    .expect("handshake");
    let session_before = bob.session_id().expect("session");

    // The attacker replays Alice's INIT and FINISH to the listener.
    let now = Instant::now();
    assert!(matches!(
        bob.on_request(FROM_ALICE, &recorded[0], now),
        Inbound::Reply(_)
    ));
    assert!(matches!(
        bob.on_request(FROM_ALICE, &recorded[2], now),
        Inbound::Rejected(LinkError::Handshake(HandshakeError::BadSignature))
    ));
    // A FINISH with no INIT before it, and a second try, get nowhere either.
    for _ in 0..2 {
        assert!(matches!(
            bob.on_request(FROM_ALICE, &recorded[2], now),
            Inbound::Rejected(LinkError::Unexpected(_))
        ));
    }
    // The genuine session is untouched and still works.
    assert_eq!(bob.session_id(), Some(session_before));
    let (_, record) = alice.seal(&ip_packet(b"still here")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &record).expect("open");

    // Replaying the listener's messages to a new dialer attempt fails too.
    let mut alice_again = dialer(&n);
    alice_again
        .begin(TO_BOB, &n.bob.public_key())
        .expect("begin");
    assert!(matches!(
        alice_again.on_response(TO_BOB, &recorded[1]),
        Err(LinkError::Handshake(HandshakeError::BadSignature))
    ));
    assert!(alice_again.active.is_none());
    assert!(!alice_again.handshake_in_progress());
}

#[test]
fn unpinned_and_unexpected_identities_get_no_session() {
    let n = nodes();
    let mallory = identity();
    let mallory_trust = TrustedPeers::from_keys([n.bob.public_key()]);

    // Mallory knows Bob's key, but Bob has not pinned Mallory.
    let mut rogue: Endpoint<'_, u8> = Endpoint::new(
        &mallory,
        &mallory_trust,
        false,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| true),
    );
    let mut bob = listener(&n);
    let init = rogue.begin(TO_BOB, &n.bob.public_key()).expect("begin");
    assert!(matches!(
        bob.on_request(FROM_ALICE, &init, Instant::now()),
        Inbound::Rejected(LinkError::Handshake(HandshakeError::UntrustedPeer))
    ));
    assert!(bob.pending.is_empty());
    assert!(bob.active.is_none());

    // Alice refuses to start towards an identity she has not pinned.
    let mut alice = dialer(&n);
    assert!(matches!(
        alice.begin(TO_BOB, &mallory.public_key()),
        Err(LinkError::Handshake(HandshakeError::UntrustedPeer))
    ));

    // Alice expects Bob, but Mallory (pinning Alice) answers in his place.
    let rogue_trust = TrustedPeers::from_keys([n.alice.public_key()]);
    let mut impostor: Endpoint<'_, u8> = Endpoint::new(
        &mallory,
        &rogue_trust,
        true,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| true),
    );
    let result = handshake_through(&mut alice, &mut impostor, &n.bob.public_key(), |_, m| m);
    assert!(result.is_err(), "an impostor completed the handshake");
    assert!(alice.active.is_none());
    assert!(impostor.active.is_none());

    // A dialer does not answer handshakes at all.
    let mut bob_dialer: Endpoint<'_, u8> = Endpoint::new(
        &n.bob,
        &n.bob_trust,
        false,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| true),
    );
    let init = alice.begin(TO_BOB, &n.bob.public_key()).expect("begin");
    assert!(matches!(
        bob_dialer.on_request(FROM_ALICE, &init, Instant::now()),
        Inbound::Rejected(LinkError::Unexpected(_))
    ));
}

#[test]
fn the_handshake_identity_must_be_the_transport_peer() {
    let n = nodes();
    // The transport says the link does not belong to the claimed identity.
    let mut alice: Endpoint<'_, u8> = Endpoint::new(
        &n.alice,
        &n.alice_trust,
        false,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| false),
    );
    assert!(matches!(
        alice.begin(TO_BOB, &n.bob.public_key()),
        Err(LinkError::IdentityNotBound)
    ));

    let mut alice = dialer(&n);
    let mut bob: Endpoint<'_, u8> = Endpoint::new(
        &n.bob,
        &n.bob_trust,
        true,
        HANDSHAKE_TIMEOUT,
        Box::new(|_, _| false),
    );
    let init = alice.begin(TO_BOB, &n.bob.public_key()).expect("begin");
    assert!(matches!(
        bob.on_request(FROM_ALICE, &init, Instant::now()),
        Inbound::Rejected(LinkError::IdentityNotBound)
    ));
    assert!(bob.pending.is_empty());
}

#[test]
fn key_rotation_is_followed_and_old_records_stay_rejected() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());

    let (_, before) = alice.seal(&ip_packet(b"before")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &before).expect("open");

    for expected_epoch in 1..=3u8 {
        assert!(matches!(alice.rotate(), Some(Ok(epoch)) if epoch == expected_epoch));
        let packet = ip_packet(&[expected_epoch]);
        let (_, record) = alice.seal(&packet).expect("seal");
        assert_eq!(record[3], expected_epoch, "record is not in the new epoch");
        assert_eq!(
            deliver(&mut bob, FROM_ALICE, &record).expect("open"),
            packet
        );
        assert!(matches!(
            deliver(&mut bob, FROM_ALICE, &record),
            Err(LinkError::Record(TunnelError::ReplayDetected))
        ));
        // The other direction is unaffected.
        let (_, reply) = bob.seal(&ip_packet(b"reply")).expect("seal");
        assert_eq!(reply[3], 0);
        deliver(&mut alice, TO_BOB, &reply).expect("open reply");
    }
    assert!(deliver(&mut bob, FROM_ALICE, &before).is_err());
    assert!(
        dialer(&n).rotate().is_none(),
        "nothing to rotate without a session"
    );
}

#[test]
fn epoch_exhaustion_stops_sending_until_the_session_is_renewed() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let first_session = alice.session_id().expect("session");
    let (_, first_epoch) = alice.seal(&ip_packet(b"epoch 0")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &first_epoch).expect("open");

    for epoch in 1..=u8::MAX {
        assert!(matches!(alice.rotate(), Some(Ok(e)) if e == epoch));
    }
    // The last epoch still carries traffic, and the receiver follows to it.
    let last = ip_packet(b"last epoch");
    let (_, record) = alice.seal(&last).expect("seal in the last epoch");
    assert_eq!(record[3], u8::MAX);
    assert_eq!(deliver(&mut bob, FROM_ALICE, &record).expect("open"), last);

    // There is no epoch after the last one. The epoch does not wrap around to
    // keys that were used before: the session refuses to rotate and to send.
    for _ in 0..2 {
        assert!(matches!(
            alice.rotate(),
            Some(Err(TunnelError::SessionExpired))
        ));
        assert!(matches!(
            alice.seal(&ip_packet(b"after the last epoch")),
            Err(LinkError::Record(TunnelError::SessionExpired))
        ));
    }
    // A record delivered in the first epoch does not become valid again.
    assert!(deliver(&mut bob, FROM_ALICE, &first_epoch).is_err());

    // The other direction has its own epochs and still works.
    let (_, reply) = bob.seal(&ip_packet(b"reply")).expect("seal");
    assert_eq!(reply[3], 0);
    deliver(&mut alice, TO_BOB, &reply).expect("open reply");

    // Only a new handshake lets the exhausted side send again, in epoch 0 of
    // a new session with new keys.
    let (at_alice, at_bob) =
        handshake_through(&mut alice, &mut bob, &n.bob.public_key(), |_, m| m).expect("renewal");
    assert_ne!(at_alice.session_id, first_session);
    assert_eq!(at_alice.session_id, at_bob.session_id);
    let renewed = ip_packet(b"renewed");
    let (_, record) = alice.seal(&renewed).expect("seal after renewal");
    assert_eq!(record[3], 0);
    assert_eq!(
        deliver(&mut bob, FROM_ALICE, &record).expect("open"),
        renewed
    );
    assert!(matches!(alice.rotate(), Some(Ok(1))));
}

#[test]
fn renewing_the_session_replaces_the_keys() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let first_session = alice.session_id().expect("session");
    let (_, used) = alice.seal(&ip_packet(b"used")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &used).expect("open");
    let (_, in_flight) = alice.seal(&ip_packet(b"in flight")).expect("seal");
    let (_, too_late) = alice.seal(&ip_packet(b"too late")).expect("seal");

    let (at_alice, at_bob) =
        handshake_through(&mut alice, &mut bob, &n.bob.public_key(), |_, m| m).expect("renewal");
    assert_ne!(at_alice.session_id, first_session);
    assert_eq!(at_alice.session_id, at_bob.session_id);
    assert_eq!((at_alice.replaced, at_bob.replaced), (None, None));

    // The new session works in both directions.
    let (_, record) = alice.seal(&ip_packet(b"renewed")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &record).expect("open");
    let (_, record) = bob.seal(&ip_packet(b"renewed reply")).expect("seal");
    deliver(&mut alice, TO_BOB, &record).expect("open");

    // A record that was in flight during the renewal still arrives once; a
    // record already delivered under the old session stays a replay.
    deliver(&mut bob, FROM_ALICE, &in_flight).expect("in-flight record");
    assert!(deliver(&mut bob, FROM_ALICE, &in_flight).is_err());
    assert!(deliver(&mut bob, FROM_ALICE, &used).is_err());

    // After the grace period the old keys are gone.
    bob.previous.as_mut().expect("previous session").1 = Instant::now();
    assert!(matches!(
        deliver(&mut bob, FROM_ALICE, &too_late),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));
    assert!(bob.previous.is_none());
}

/// Runs a renewal up to the point where the listener has the new session and
/// the dialer does not: CONFIRM is returned instead of being delivered.
fn renew_up_to_confirm(
    alice: &mut Endpoint<'_, u8>,
    bob: &mut Endpoint<'_, u8>,
    expected: &PeerPublicKey,
) -> Vec<u8> {
    let now = Instant::now();
    let init = alice.begin(TO_BOB, expected).expect("begin");
    let response = match bob.on_request(FROM_ALICE, &init, now) {
        Inbound::Reply(response) => response,
        other => unreachable!("INIT: {other:?}"),
    };
    let finish = match alice.on_response(TO_BOB, &response) {
        Ok(Progress::Send(finish)) => finish,
        other => unreachable!("RESPONSE: {other:?}"),
    };
    match bob.on_request(FROM_ALICE, &finish, now) {
        Inbound::Established(confirm, _) => confirm,
        other => unreachable!("FINISH: {other:?}"),
    }
}

#[test]
fn no_packet_is_lost_while_a_renewal_is_between_its_two_ends() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let first_session = alice.session_id().expect("session");

    let confirm = renew_up_to_confirm(&mut alice, &mut bob, &n.bob.public_key());
    assert_ne!(bob.session_id(), Some(first_session));
    assert_eq!(alice.session_id(), Some(first_session));

    // The listener has the new session and the dialer does not yet. Packets
    // still arrive in both directions.
    let packet = ip_packet(b"listener to dialer, before CONFIRM arrived");
    let (_, record) = bob.seal(&packet).expect("seal");
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), packet);
    let packet = ip_packet(b"dialer to listener, before CONFIRM arrived");
    let (_, record) = alice.seal(&packet).expect("seal");
    assert_eq!(
        deliver(&mut bob, FROM_ALICE, &record).expect("open"),
        packet
    );

    assert!(matches!(
        alice.on_response(TO_BOB, &confirm),
        Ok(Progress::Established(_))
    ));
    assert_eq!(alice.session_id(), bob.session_id());
    let packet = ip_packet(b"listener to dialer, after CONFIRM");
    let (_, record) = bob.seal(&packet).expect("seal");
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), packet);

    // Once the dialer has used the new session, the listener uses it too:
    // its records no longer depend on the previous session.
    let (_, record) = alice.seal(&ip_packet(b"first use")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &record).expect("open");
    assert!(bob.send_on_previous_until.is_none());
    alice.previous = None;
    let packet = ip_packet(b"in the new session");
    let (_, record) = bob.seal(&packet).expect("seal");
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), packet);
}

#[test]
fn a_listener_moves_to_the_new_session_by_itself_if_the_dialer_stays_silent() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let confirm = renew_up_to_confirm(&mut alice, &mut bob, &n.bob.public_key());
    assert!(matches!(
        alice.on_response(TO_BOB, &confirm),
        Ok(Progress::Established(_))
    ));

    // The time in which the listener sends in the previous session ends well
    // before the dialer stops accepting that session.
    let switch = bob.send_on_previous_until.expect("renewed as responder");
    let accepted_until = alice.previous.as_ref().expect("previous session").1;
    assert!(switch + Duration::from_secs(1) < accepted_until);
    assert!(
        alice.send_on_previous_until.is_none(),
        "only the responder waits"
    );

    // After that time the listener sends in the new session, without having
    // received anything from the dialer.
    bob.send_on_previous_until = Some(Instant::now());
    alice.previous = None;
    let packet = ip_packet(b"listener speaks first");
    let (_, record) = bob.seal(&packet).expect("seal");
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), packet);
}

/// One packet in each direction arrives, and arrives only once.
fn both_ways(alice: &mut Endpoint<'_, u8>, bob: &mut Endpoint<'_, u8>, tag: &str) {
    let packet = ip_packet(format!("{tag}: dialer to listener").as_bytes());
    let (_, record) = alice.seal(&packet).expect("seal");
    assert_eq!(deliver(bob, FROM_ALICE, &record).expect(tag), packet);
    assert!(
        deliver(bob, FROM_ALICE, &record).is_err(),
        "{tag}: delivered twice to the listener"
    );
    let packet = ip_packet(format!("{tag}: listener to dialer").as_bytes());
    let (_, record) = bob.seal(&packet).expect("seal");
    assert_eq!(deliver(alice, TO_BOB, &record).expect(tag), packet);
    assert!(
        deliver(alice, TO_BOB, &record).is_err(),
        "{tag}: delivered twice to the dialer"
    );
}

/// A sealed record in each direction that has not been delivered yet.
struct InFlight {
    to_bob: (Vec<u8>, Vec<u8>),
    to_alice: (Vec<u8>, Vec<u8>),
}

fn launch(alice: &Endpoint<'_, u8>, bob: &Endpoint<'_, u8>, tag: &str) -> InFlight {
    let to_bob = ip_packet(format!("{tag}: in flight to the listener").as_bytes());
    let to_alice = ip_packet(format!("{tag}: in flight to the dialer").as_bytes());
    InFlight {
        to_bob: (alice.seal(&to_bob).expect("seal").1, to_bob),
        to_alice: (bob.seal(&to_alice).expect("seal").1, to_alice),
    }
}

fn land(alice: &mut Endpoint<'_, u8>, bob: &mut Endpoint<'_, u8>, flight: InFlight, tag: &str) {
    let (record, packet) = flight.to_bob;
    assert_eq!(deliver(bob, FROM_ALICE, &record).expect(tag), packet);
    assert!(deliver(bob, FROM_ALICE, &record).is_err(), "{tag}: twice");
    let (record, packet) = flight.to_alice;
    assert_eq!(deliver(alice, TO_BOB, &record).expect(tag), packet);
    assert!(deliver(alice, TO_BOB, &record).is_err(), "{tag}: twice");
}

/// Renews the session one handshake message at a time. Around every message a
/// record in each direction is sealed before it and delivered after it, and
/// after every message a packet is exchanged in each direction. Everything
/// must arrive exactly once. Also checks which side holds which session.
fn renew_stepwise(
    alice: &mut Endpoint<'_, u8>,
    bob: &mut Endpoint<'_, u8>,
    expected: &PeerPublicKey,
    tag: &str,
) {
    let old = alice.session_id().expect("session");
    assert_eq!(bob.session_id(), Some(old));
    let now = Instant::now();

    let flight = launch(alice, bob, tag);
    let init = alice.begin(TO_BOB, expected).expect("begin");
    let response = match bob.on_request(FROM_ALICE, &init, now) {
        Inbound::Reply(response) => response,
        other => unreachable!("INIT: {other:?}"),
    };
    assert_eq!(
        (alice.session_id(), bob.session_id()),
        (Some(old), Some(old))
    );
    land(alice, bob, flight, tag);
    both_ways(alice, bob, tag);

    let flight = launch(alice, bob, tag);
    let finish = match alice.on_response(TO_BOB, &response) {
        Ok(Progress::Send(finish)) => finish,
        other => unreachable!("RESPONSE: {other:?}"),
    };
    assert_eq!(
        (alice.session_id(), bob.session_id()),
        (Some(old), Some(old))
    );
    land(alice, bob, flight, tag);
    both_ways(alice, bob, tag);

    // FINISH: the listener has verified the dialer and installs the session.
    let flight = launch(alice, bob, tag);
    let confirm = match bob.on_request(FROM_ALICE, &finish, now) {
        Inbound::Established(confirm, _) => confirm,
        other => unreachable!("FINISH: {other:?}"),
    };
    let new = bob.session_id().expect("session");
    assert_ne!(new, old);
    assert_eq!(alice.session_id(), Some(old), "installed before CONFIRM");
    land(alice, bob, flight, tag);
    both_ways(alice, bob, tag);

    // CONFIRM: the dialer has verified the listener and installs it too.
    let flight = launch(alice, bob, tag);
    assert!(matches!(
        alice.on_response(TO_BOB, &confirm),
        Ok(Progress::Established(_))
    ));
    assert_eq!(
        (alice.session_id(), bob.session_id()),
        (Some(new), Some(new))
    );
    land(alice, bob, flight, tag);
    both_ways(alice, bob, tag);
    assert!(bob.send_on_previous_until.is_none());
}

#[test]
fn every_step_of_a_renewal_carries_packets_in_both_directions() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    renew_stepwise(&mut alice, &mut bob, &n.bob.public_key(), "renewal");
}

#[test]
fn repeated_renewals_under_traffic_lose_and_duplicate_nothing() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let mut seen = std::collections::HashSet::new();
    seen.insert(alice.session_id().expect("session"));
    for round in 0..200 {
        renew_stepwise(
            &mut alice,
            &mut bob,
            &n.bob.public_key(),
            &format!("round {round}"),
        );
        if round % 3 == 0 {
            // Key rotation and renewal do not get in each other's way.
            assert!(matches!(alice.rotate(), Some(Ok(1))));
            assert!(matches!(bob.rotate(), Some(Ok(1))));
            both_ways(&mut alice, &mut bob, "rotated");
        }
        assert!(
            seen.insert(alice.session_id().expect("session")),
            "a session id was used twice"
        );
    }
}

#[test]
fn a_failed_renewal_leaves_the_existing_session_in_place() {
    let n = nodes();
    let expected = n.bob.public_key();
    for target in 0..4 {
        let (mut alice, mut bob) = (dialer(&n), listener(&n));
        connect(&mut alice, &mut bob, &expected);
        let first = alice.session_id().expect("session");

        let result = handshake_through(&mut alice, &mut bob, &expected, |index, mut message| {
            if index == target {
                let middle = message.len() / 2;
                message[middle] ^= 0x01;
            }
            message
        });
        assert!(
            result.is_err(),
            "a modified message {target} renewed the session"
        );

        // The dialer installs nothing it did not verify.
        assert_eq!(alice.session_id(), Some(first));
        if target < 3 {
            assert_eq!(bob.session_id(), Some(first));
            assert!(bob.previous.is_none());
        } else {
            // Only CONFIRM was damaged: the listener had already verified the
            // dialer. The dialer keeps the session it has, which the listener
            // still serves until the link is closed.
            assert_ne!(bob.session_id(), Some(first));
        }
        both_ways(&mut alice, &mut bob, "after the failed renewal");

        // The link can be renewed afterwards, or closed without leftovers.
        if target % 2 == 0 {
            renew_stepwise(&mut alice, &mut bob, &expected, "second attempt");
        }
        assert!(alice.close(TO_BOB));
        assert!(bob.close(FROM_ALICE));
        assert!(alice.active.is_none() && alice.previous.is_none() && alice.outbound.is_none());
        assert!(bob.active.is_none() && bob.previous.is_none() && bob.pending.is_empty());
        assert!(bob.send_on_previous_until.is_none());
    }
}

#[test]
fn the_replaced_session_is_retired_on_time_even_without_traffic() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let (_, old_record) = alice
        .seal(&ip_packet(b"sealed in the old session"))
        .expect("seal");
    handshake_through(&mut alice, &mut bob, &n.bob.public_key(), |_, m| m).expect("renewal");

    let now = Instant::now();
    for endpoint in [&mut alice, &mut bob] {
        endpoint.retire_previous(now);
        assert!(
            endpoint.previous.is_some(),
            "retired within the grace period"
        );
        endpoint.retire_previous(now + PREVIOUS_SESSION_GRACE + Duration::from_millis(1));
        assert!(endpoint.previous.is_none(), "kept beyond the grace period");
        assert!(endpoint.send_on_previous_until.is_none());
    }

    // The old keys are gone: its records are refused, the new session works.
    assert!(matches!(
        deliver(&mut bob, FROM_ALICE, &old_record),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));
    both_ways(&mut alice, &mut bob, "after retirement");
}

#[test]
fn a_listener_whose_old_session_is_used_up_sends_in_the_new_one() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    for _ in 0..u8::MAX {
        assert!(matches!(bob.rotate(), Some(Ok(_))));
    }
    assert!(matches!(
        bob.rotate(),
        Some(Err(TunnelError::SessionExpired))
    ));
    assert!(bob.seal(&ip_packet(b"used up")).is_err());

    let confirm = renew_up_to_confirm(&mut alice, &mut bob, &n.bob.public_key());
    // The old session cannot send any more, so the new one is used at once
    // and never the expired keys.
    let packet = ip_packet(b"in the new session");
    let (_, record) = bob.seal(&packet).expect("seal in the new session");
    assert_eq!(
        record[3], 0,
        "not a record of the new session's first epoch"
    );
    assert!(matches!(
        alice.on_response(TO_BOB, &confirm),
        Ok(Progress::Established(_))
    ));
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), packet);
    both_ways(&mut alice, &mut bob, "after the renewal");
}

/// CONFIRM of a renewal never reaches the dialer: the listener holds a session
/// the dialer does not have. Until the dialer has started over, each side
/// accepts only records of a session it holds itself, and afterwards nothing
/// that was recorded on the way is accepted anywhere.
#[test]
fn a_renewal_whose_confirm_is_lost_accepts_nothing_unauthorized_and_recovers() {
    let n = nodes();
    let expected = n.bob.public_key();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &expected);
    let first = alice.session_id().expect("session");
    let (_, old_from_alice) = alice
        .seal(&ip_packet(b"old session, from the dialer"))
        .expect("seal");
    let (_, old_from_bob) = bob
        .seal(&ip_packet(b"old session, from the listener"))
        .expect("seal");

    let lost_confirm = renew_up_to_confirm(&mut alice, &mut bob, &expected);
    let half_open = bob.session_id().expect("session");
    assert_ne!(half_open, first);
    assert_eq!(alice.session_id(), Some(first));
    assert!(alice.handshake_in_progress());

    // Records that were in flight in the old session still arrive, once.
    deliver(&mut bob, FROM_ALICE, &old_from_alice).expect("in flight to the listener");
    deliver(&mut alice, TO_BOB, &old_from_bob).expect("in flight to the dialer");

    // The listener stops waiting and sends in the session only it holds. The
    // dialer has no keys for it, refuses the record and keeps its session.
    bob.send_on_previous_until = Some(Instant::now());
    let (_, unconfirmed) = bob
        .seal(&ip_packet(b"in the session the dialer never confirmed"))
        .expect("seal");
    assert!(matches!(
        deliver(&mut alice, TO_BOB, &unconfirmed),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));
    assert_eq!(alice.session_id(), Some(first));

    // After the grace period the listener no longer has the old session, so
    // what the dialer still seals in it is refused.
    bob.retire_previous(Instant::now() + PREVIOUS_SESSION_GRACE + Duration::from_millis(1));
    let (_, late) = alice
        .seal(&ip_packet(b"old session, after the grace period"))
        .expect("seal");
    assert!(matches!(
        deliver(&mut bob, FROM_ALICE, &late),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));

    // The dialer's handshake times out and it closes the link. From here it
    // sends nothing, and the CONFIRM it waited for is no longer of any use.
    assert!(alice.close(TO_BOB));
    assert!(!alice.handshake_in_progress());
    assert!(matches!(
        alice.seal(&ip_packet(b"no session")),
        Err(LinkError::NoSession)
    ));
    assert!(matches!(
        alice.on_response(TO_BOB, &lost_confirm),
        Err(LinkError::Unexpected(_))
    ));
    assert!(matches!(
        deliver(&mut alice, TO_BOB, &unconfirmed),
        Err(LinkError::NoSession)
    ));
    assert_eq!(alice.session_id(), None);

    // The dialer comes back. The recorded CONFIRM does not complete the new
    // handshake either: it confirms a different key agreement.
    let substituted = handshake_through(&mut alice, &mut bob, &expected, |index, message| {
        if index == 3 {
            lost_confirm.clone()
        } else {
            message
        }
    });
    assert!(
        substituted.is_err(),
        "a recorded CONFIRM completed a handshake"
    );
    assert_eq!(alice.session_id(), None);
    assert!(!alice.handshake_in_progress());

    // The listener learns that the dialer started over and drops what it
    // held. The next handshake gives both the same, new session.
    assert!(bob.close(FROM_ALICE));
    let (at_alice, at_bob) =
        handshake_through(&mut alice, &mut bob, &expected, |_, m| m).expect("recovery");
    assert_eq!(at_alice.session_id, at_bob.session_id);
    assert!(at_alice.session_id != first && at_alice.session_id != half_open);
    assert!(bob.previous.is_none() && bob.send_on_previous_until.is_none());

    // The listener may speak first: it sends in the session the dialer has.
    let packet = ip_packet(b"listener speaks first after the recovery");
    let (_, record) = bob.seal(&packet).expect("seal");
    assert_eq!(deliver(&mut alice, TO_BOB, &record).expect("open"), packet);
    both_ways(&mut alice, &mut bob, "after the recovery");

    // Everything recorded along the way is refused by both sides and leaves
    // the recovered session as it is.
    for record in [&old_from_alice, &late, &old_from_bob, &unconfirmed] {
        assert!(deliver(&mut bob, FROM_ALICE, record).is_err());
        assert!(deliver(&mut alice, TO_BOB, record).is_err());
    }
    assert!(matches!(
        bob.on_request(FROM_ALICE, &lost_confirm, Instant::now()),
        Inbound::Rejected(_)
    ));
    assert!(alice.on_response(TO_BOB, &lost_confirm).is_err());
    assert_eq!(alice.session_id(), Some(at_alice.session_id));
    assert_eq!(bob.session_id(), Some(at_alice.session_id));
    both_ways(&mut alice, &mut bob, "after the replays");
}

#[test]
fn closing_a_link_ends_its_session_and_a_new_link_takes_over() {
    let n = nodes();
    let (mut alice, mut bob) = (dialer(&n), listener(&n));
    connect(&mut alice, &mut bob, &n.bob.public_key());
    let (_, old_record) = alice.seal(&ip_packet(b"old")).expect("seal");

    // Closing an unrelated link changes nothing.
    assert!(!bob.close(99));
    assert!(bob.active.is_some());

    assert!(bob.close(FROM_ALICE));
    assert!(alice.close(TO_BOB));
    for endpoint in [&alice, &bob] {
        assert!(endpoint.active.is_none() && endpoint.previous.is_none());
        assert!(matches!(
            endpoint.seal(&ip_packet(b"x")),
            Err(LinkError::NoSession)
        ));
    }
    assert!(matches!(
        deliver(&mut bob, FROM_ALICE, &old_record),
        Err(LinkError::NoSession)
    ));

    // Recovery is a complete new handshake with new keys.
    connect(&mut alice, &mut bob, &n.bob.public_key());
    assert!(matches!(
        deliver(&mut bob, FROM_ALICE, &old_record),
        Err(LinkError::Record(TunnelError::DecryptionFailed))
    ));
    let (_, record) = alice.seal(&ip_packet(b"new")).expect("seal");
    deliver(&mut bob, FROM_ALICE, &record).expect("open");

    // A session established over another link replaces this one and names it.
    let mut alice_elsewhere = dialer(&n);
    let now = Instant::now();
    let init = alice_elsewhere
        .begin(TO_BOB, &n.bob.public_key())
        .expect("begin");
    let Inbound::Reply(response) = bob.on_request(77, &init, now) else {
        panic!("INIT refused");
    };
    let Ok(Progress::Send(finish)) = alice_elsewhere.on_response(TO_BOB, &response) else {
        panic!("RESPONSE refused");
    };
    let Inbound::Established(_, established) = bob.on_request(77, &finish, now) else {
        panic!("FINISH refused");
    };
    assert_eq!(established.replaced, Some(FROM_ALICE));
    assert_eq!(bob.active_link(), Some(77));
}

#[test]
fn half_open_handshakes_are_bounded_and_expire() {
    let n = nodes();
    let mut bob = listener(&n);
    let start = Instant::now();
    for link in 0..(MAX_PENDING_HANDSHAKES as u8 + 8) {
        let mut alice = dialer(&n);
        let init = alice.begin(TO_BOB, &n.bob.public_key()).expect("begin");
        assert!(matches!(
            bob.on_request(link, &init, start + Duration::from_millis(link as u64)),
            Inbound::Reply(_)
        ));
        assert!(bob.pending.len() <= MAX_PENDING_HANDSHAKES);
    }
    assert_eq!(bob.pending.len(), MAX_PENDING_HANDSHAKES);
    assert!(
        !bob.pending.contains_key(&0),
        "the oldest entry was not evicted"
    );

    bob.expire_pending(start + HANDSHAKE_TIMEOUT + Duration::from_secs(1));
    assert!(bob.pending.is_empty());

    // A FINISH that arrives after the timeout is refused.
    let mut alice = dialer(&n);
    let init = alice.begin(TO_BOB, &n.bob.public_key()).expect("begin");
    let Inbound::Reply(response) = bob.on_request(FROM_ALICE, &init, start) else {
        panic!("INIT refused");
    };
    let Ok(Progress::Send(finish)) = alice.on_response(TO_BOB, &response) else {
        panic!("RESPONSE refused");
    };
    let late = start + HANDSHAKE_TIMEOUT + Duration::from_secs(1);
    assert!(matches!(
        bob.on_request(FROM_ALICE, &finish, late),
        Inbound::Rejected(LinkError::Unexpected(_))
    ));
    assert!(bob.active.is_none());
}

#[test]
fn transport_peer_ids_are_derived_from_the_pinned_identity() {
    let node = identity();
    let keypair = transport_keypair(&node).expect("keypair");
    let peer_id = peer_id_of(&node.public_key()).expect("peer id");
    assert_eq!(keypair.public().to_peer_id(), peer_id);
    assert!(link_is_bound(&peer_id, &node.public_key()));
    assert!(!link_is_bound(&peer_id, &identity().public_key()));
    // The same identity always yields the same transport peer id.
    assert_eq!(
        transport_keypair(&node)
            .expect("keypair")
            .public()
            .to_peer_id(),
        peer_id
    );
}

// -------------------------------------------------------------------------
// Network tests: two engines, each with its own identity and swarm, connected
// over QUIC on the loopback interface.
// -------------------------------------------------------------------------

const WAIT: Duration = Duration::from_secs(20);
const QUIET: Duration = Duration::from_millis(1500);

struct Peer {
    events: mpsc::UnboundedReceiver<VpnEvent>,
    /// Stands in for the device: packets to send, packets received.
    device_out: mpsc::Sender<Vec<u8>>,
    device_in: mpsc::Receiver<Vec<u8>>,
    stop: Option<oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<Result<()>>,
}

struct Timing {
    rotate: Duration,
    rehandshake: Duration,
}

const SLOW: Timing = Timing {
    rotate: Duration::from_secs(3600),
    rehandshake: Duration::from_secs(3600),
};

fn start(seed: &[u8; 32], trusted: &[PeerPublicKey], mode: VpnMode, timing: Timing) -> Peer {
    let mut tunnel = TunnelConfig::new(
        std::sync::Arc::new(NodeIdentity::from_secret_seed(seed)),
        TrustedPeers::from_keys(trusted.iter().copied()),
    );
    tunnel.handshake_timeout = Duration::from_secs(5);
    let (event_tx, events) = mpsc::unbounded_channel();
    let (device_out, from_device) = mpsc::channel(64);
    let (to_device, device_in) = mpsc::channel(64);
    let (stop, stopped) = oneshot::channel::<()>();
    let settings = VpnSettings {
        tunnel,
        rotate_interval: timing.rotate,
        rehandshake_interval: timing.rehandshake,
        events: Some(event_tx),
    };
    let io = PacketIo {
        from_device,
        to_device,
    };
    let task = tokio::spawn(async move {
        run_secure_vpn(settings, mode, io, async {
            let _ = stopped.await;
        })
        .await
    });
    Peer {
        events,
        device_out,
        device_in,
        stop: Some(stop),
        task,
    }
}

impl Peer {
    async fn event<T>(&mut self, mut pick: impl FnMut(&VpnEvent) -> Option<T>) -> T {
        timeout(WAIT, async {
            loop {
                let event = self.events.recv().await.expect("engine stopped");
                if let Some(value) = pick(&event) {
                    return value;
                }
            }
        })
        .await
        .expect("event did not arrive in time")
    }

    async fn listening(&mut self) -> Multiaddr {
        self.event(|e| match e {
            VpnEvent::Listening(addr) => Some(addr.clone()),
            _ => None,
        })
        .await
    }

    async fn session(&mut self) -> (PeerPublicKey, [u8; 16], TunnelRole) {
        self.event(|e| match e {
            VpnEvent::SessionEstablished {
                peer,
                session_id,
                role,
            } => Some((*peer, *session_id, *role)),
            _ => None,
        })
        .await
    }

    async fn closed(&mut self) {
        self.event(|e| matches!(e, VpnEvent::SessionClosed { .. }).then_some(()))
            .await
    }

    async fn rejected(&mut self) -> String {
        self.event(|e| match e {
            VpnEvent::Rejected(reason) => Some(reason.clone()),
            _ => None,
        })
        .await
    }

    /// Whether a session is established within the quiet period.
    async fn gets_a_session(&mut self) -> bool {
        timeout(QUIET, async {
            loop {
                match self.events.recv().await {
                    Some(VpnEvent::SessionEstablished { .. }) => return true,
                    Some(_) => {}
                    None => std::future::pending::<()>().await,
                }
            }
        })
        .await
        .unwrap_or(false)
    }

    async fn send(&self, packet: &[u8]) {
        self.device_out
            .send(packet.to_vec())
            .await
            .expect("device queue");
    }

    async fn receive(&mut self) -> Vec<u8> {
        timeout(WAIT, self.device_in.recv())
            .await
            .expect("packet did not arrive in time")
            .expect("device channel closed")
    }

    async fn receives_nothing(&mut self) -> bool {
        timeout(QUIET, self.device_in.recv()).await.is_err()
    }

    async fn shut_down(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        timeout(WAIT, self.task)
            .await
            .expect("engine did not stop")
            .expect("engine task")
            .expect("engine result");
    }
}

fn seed() -> ([u8; 32], PeerPublicKey) {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    (seed, NodeIdentity::from_secret_seed(&seed).public_key())
}

fn loopback() -> Multiaddr {
    "/ip4/127.0.0.1/udp/0/quic-v1".parse().expect("multiaddr")
}

async fn exchange(a: &mut Peer, b: &mut Peer, tag: &[u8]) {
    let packet = ip_packet(tag);
    a.send(&packet).await;
    assert_eq!(b.receive().await, packet);
    let reply = ip_packet(&[tag, b" reply"].concat());
    b.send(&reply).await;
    assert_eq!(a.receive().await, reply);
}

#[tokio::test]
async fn two_pinned_peers_exchange_packets_and_rotate_keys_over_quic() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let fast = || Timing {
        rotate: Duration::from_millis(300),
        rehandshake: Duration::from_secs(3600),
    };

    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), fast());
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        fast(),
    );

    let (peer_at_alice, session_at_alice, role_at_alice) = alice.session().await;
    let (peer_at_bob, session_at_bob, role_at_bob) = bob.session().await;
    assert_eq!(peer_at_alice, bob_key);
    assert_eq!(peer_at_bob, alice_key);
    assert_eq!(session_at_alice, session_at_bob);
    assert_eq!(role_at_alice, TunnelRole::Initiator);
    assert_eq!(role_at_bob, TunnelRole::Responder);

    exchange(&mut alice, &mut bob, b"first").await;

    // Both directions rotate on their own timers and traffic keeps flowing.
    for peer in [&mut alice, &mut bob] {
        let epoch = peer
            .event(|e| match e {
                VpnEvent::KeysRotated(epoch) => Some(*epoch),
                _ => None,
            })
            .await;
        assert!(epoch >= 1);
    }
    for round in 0..5u8 {
        exchange(&mut alice, &mut bob, &[b'r', round]).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    alice.shut_down().await;
    bob.closed().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn the_session_is_renewed_by_a_new_handshake() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        Timing {
            rotate: Duration::from_secs(3600),
            rehandshake: Duration::from_millis(700),
        },
    );

    let (_, first, _) = alice.session().await;
    assert_eq!(bob.session().await.1, first);
    exchange(&mut alice, &mut bob, b"first session").await;

    let (_, second, _) = alice.session().await;
    assert_eq!(bob.session().await.1, second);
    assert_ne!(first, second, "the renewal kept the old session");
    exchange(&mut alice, &mut bob, b"second session").await;

    alice.shut_down().await;
    bob.shut_down().await;
}

/// Rotates fast enough to use up all 255 key epochs of a session in about
/// 2.5 seconds.
const EXHAUSTING: Timing = Timing {
    rotate: Duration::from_millis(10),
    rehandshake: Duration::from_secs(3600),
};

async fn last_epoch(peer: &mut Peer) {
    peer.event(|e| matches!(e, VpnEvent::KeysRotated(u8::MAX)).then_some(()))
        .await
}

#[tokio::test]
async fn a_dialer_that_runs_out_of_epochs_renews_the_session() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        EXHAUSTING,
    );
    let (_, first, _) = alice.session().await;
    assert_eq!(bob.session().await.1, first);
    exchange(&mut alice, &mut bob, b"first session").await;

    // Renewal is not due for an hour; it happens because the dialer reaches
    // its last epoch and cannot rotate any further.
    last_epoch(&mut alice).await;
    let (_, second, _) = alice.session().await;
    assert_eq!(bob.session().await.1, second);
    assert_ne!(first, second, "the exhausted session was kept");
    exchange(&mut alice, &mut bob, b"second session").await;

    alice.shut_down().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn a_listener_that_runs_out_of_epochs_makes_the_dialer_come_back() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(
        &bob_seed,
        &[alice_key],
        VpnMode::Listen(loopback()),
        EXHAUSTING,
    );
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        SLOW,
    );
    let (_, first, _) = alice.session().await;
    assert_eq!(bob.session().await.1, first);
    exchange(&mut alice, &mut bob, b"first session").await;

    // A listener cannot start a handshake. It ends the session it can no
    // longer send in, and the dialer establishes a new one.
    last_epoch(&mut bob).await;
    bob.closed().await;
    alice.closed().await;
    let (_, second, _) = alice.session().await;
    assert_eq!(bob.session().await.1, second);
    assert_ne!(first, second, "the exhausted session was kept");
    exchange(&mut alice, &mut bob, b"second session").await;

    alice.shut_down().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn traffic_keeps_flowing_while_the_session_is_renewed_again_and_again() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        Timing {
            rotate: Duration::from_secs(3600),
            rehandshake: Duration::from_millis(250),
        },
    );
    alice.session().await;
    bob.session().await;

    // The listener speaks first in every round: that is the direction in
    // which a packet sent between the two ends of a renewal would be lost.
    // No packet may be missing, whichever moment the renewal picks.
    let started = Instant::now();
    let mut round = 0u32;
    while started.elapsed() < Duration::from_secs(4) {
        let packet = ip_packet(format!("listener first {round}").as_bytes());
        bob.send(&packet).await;
        assert_eq!(alice.receive().await, packet);
        let packet = ip_packet(format!("dialer answers {round}").as_bytes());
        alice.send(&packet).await;
        assert_eq!(bob.receive().await, packet);
        round += 1;
    }

    let mut sessions = std::collections::HashSet::new();
    while let Ok(event) = alice.events.try_recv() {
        if let VpnEvent::SessionEstablished { session_id, .. } = event {
            sessions.insert(session_id);
        }
    }
    assert!(
        sessions.len() >= 5,
        "only {} renewals in {round} rounds",
        sessions.len()
    );
    assert!(round >= 100, "only {round} rounds");
    assert!(alice.receives_nothing().await && bob.receives_nothing().await);

    alice.shut_down().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn losing_the_device_stops_the_link_with_an_error() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        SLOW,
    );
    alice.session().await;
    bob.session().await;
    exchange(&mut alice, &mut bob, b"while the device is there").await;

    // The device goes away while nobody asked the link to stop.
    let Peer {
        device_out,
        task,
        stop: _still_running,
        ..
    } = alice;
    drop(device_out);
    let result = timeout(WAIT, task)
        .await
        .expect("engine did not stop")
        .expect("engine task");
    let error = result.expect_err("a link without a device reported success");
    assert!(error.to_string().contains("device"), "{error}");

    // The peer is not left with a session nobody serves.
    bob.closed().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn teardown_drops_the_session_and_the_dialer_recovers() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();

    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    let dial = || VpnMode::Dial {
        addr: addr.clone(),
        peer: bob_key,
    };
    let mut alice = start(&alice_seed, &[bob_key], dial(), SLOW);
    let (_, first, _) = alice.session().await;
    bob.session().await;
    exchange(&mut alice, &mut bob, b"before").await;

    // The dialer goes away: the listener notices and drops the session.
    alice.shut_down().await;
    bob.closed().await;
    bob.send(&ip_packet(b"into the void")).await;

    // The same identity comes back: a new session with new keys.
    let mut alice = start(&alice_seed, &[bob_key], dial(), SLOW);
    let (_, second, _) = alice.session().await;
    assert_eq!(bob.session().await.1, second);
    assert_ne!(first, second);
    assert!(
        alice.receives_nothing().await,
        "a packet sent without a session arrived"
    );
    exchange(&mut alice, &mut bob, b"after dialer restart").await;

    // The listener restarts on the same address: the dialer reconnects by
    // itself and runs a new handshake.
    bob.shut_down().await;
    alice.closed().await;
    alice.send(&ip_packet(b"listener is down")).await;
    // The old socket is released shortly after the engine has stopped.
    let mut bob = loop {
        let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(addr.clone()), SLOW);
        match timeout(WAIT, bob.events.recv())
            .await
            .expect("listener start")
        {
            Some(VpnEvent::Listening(_)) => break bob,
            Some(other) => panic!("unexpected event {other:?}"),
            None => {
                let error = bob
                    .task
                    .await
                    .expect("engine task")
                    .expect_err("engine failed");
                assert!(
                    format!("{error:#}").contains("listen"),
                    "unexpected failure: {error:#}"
                );
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    };
    let (_, third, _) = alice.session().await;
    assert_eq!(bob.session().await.1, third);
    assert_ne!(second, third);
    assert!(
        bob.receives_nothing().await,
        "a packet sent without a session arrived"
    );
    exchange(&mut alice, &mut bob, b"after listener restart").await;

    alice.shut_down().await;
    bob.shut_down().await;
}

/// A UDP relay between the dialer and the listener, standing in for the
/// network path. It can be cut, which drops every datagram in both
/// directions, including those still on their way. Datagrams towards the
/// dialer are delayed, so the path can be cut while a message the listener
/// has just sent is still in transit.
struct Relay {
    /// The address the dialer uses instead of the listener's.
    addr: Multiaddr,
    cut: std::sync::Arc<std::sync::atomic::AtomicBool>,
    cuts: std::sync::Arc<std::sync::atomic::AtomicU64>,
    task: tokio::task::JoinHandle<()>,
}

impl Relay {
    async fn start(listener: &Multiaddr, delay_to_dialer: Duration) -> Relay {
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::sync::Arc;
        use tokio::net::UdpSocket;

        let port = listener
            .iter()
            .find_map(|part| match part {
                libp2p::multiaddr::Protocol::Udp(port) => Some(port),
                _ => None,
            })
            .expect("a UDP listener address");
        let target = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let front = Arc::new(UdpSocket::bind("127.0.0.1:0").await.expect("relay socket"));
        let back = UdpSocket::bind("127.0.0.1:0").await.expect("relay socket");
        let addr: Multiaddr = format!(
            "/ip4/127.0.0.1/udp/{}/quic-v1",
            front.local_addr().expect("relay address").port()
        )
        .parse()
        .expect("multiaddr");
        let cut = Arc::new(AtomicBool::new(false));
        let cuts = Arc::new(AtomicU64::new(0));

        let (is_cut, cut_count) = (cut.clone(), cuts.clone());
        let task = tokio::spawn(async move {
            let mut dialer = None;
            let mut from_dialer = vec![0u8; 65536];
            let mut from_listener = vec![0u8; 65536];
            loop {
                tokio::select! {
                    received = front.recv_from(&mut from_dialer) => {
                        let Ok((len, from)) = received else { continue };
                        dialer = Some(from);
                        if !is_cut.load(Ordering::SeqCst) {
                            let _ = back.send_to(&from_dialer[..len], target).await;
                        }
                    }
                    received = back.recv_from(&mut from_listener) => {
                        let Ok((len, _)) = received else { continue };
                        let Some(dialer) = dialer else { continue };
                        if is_cut.load(Ordering::SeqCst) {
                            continue;
                        }
                        let datagram = from_listener[..len].to_vec();
                        let sent_during = cut_count.load(Ordering::SeqCst);
                        let (front, is_cut, cut_count) =
                            (front.clone(), is_cut.clone(), cut_count.clone());
                        tokio::spawn(async move {
                            tokio::time::sleep(delay_to_dialer).await;
                            // A cut also loses what was in transit, even if
                            // the path is back by now.
                            if !is_cut.load(Ordering::SeqCst)
                                && cut_count.load(Ordering::SeqCst) == sent_during
                            {
                                let _ = front.send_to(&datagram, dialer).await;
                            }
                        });
                    }
                }
            }
        });
        Relay {
            addr,
            cut,
            cuts,
            task,
        }
    }

    fn cut(&self) {
        use std::sync::atomic::Ordering;
        self.cuts.fetch_add(1, Ordering::SeqCst);
        self.cut.store(true, Ordering::SeqCst);
    }

    fn restore(&self) {
        self.cut.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Drop for Relay {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// The path fails after the listener completed a renewal and before its
/// CONFIRM reached the dialer. The listener then holds a session the dialer
/// never got. The dialer must notice within its handshake timeout, reconnect,
/// and end up in one new session with the listener, and nothing may be
/// delivered that was not sent inside a session both ends hold.
#[tokio::test]
async fn a_renewal_whose_confirm_is_lost_recovers_by_reconnecting() {
    const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5); // set in `start`
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let relay = Relay::start(&bob.listening().await, Duration::from_millis(300)).await;
    let mut alice = start(
        &alice_seed,
        &[bob_key],
        VpnMode::Dial {
            addr: relay.addr.clone(),
            peer: bob_key,
        },
        Timing {
            rotate: Duration::from_secs(3600),
            rehandshake: Duration::from_secs(2),
        },
    );
    let (_, first, _) = alice.session().await;
    assert_eq!(bob.session().await.1, first);
    exchange(&mut alice, &mut bob, b"first session").await;

    // The renewal: the listener has the new session as soon as FINISH
    // arrives. Its CONFIRM is still in transit when the path fails.
    let (_, half_open, _) = bob.session().await;
    relay.cut();
    let lost = Instant::now();
    assert_ne!(half_open, first);

    // During the outage nothing arrives anywhere.
    alice.send(&ip_packet(b"dialer, during the outage")).await;
    bob.send(&ip_packet(b"listener, during the outage")).await;
    assert!(alice.receives_nothing().await && bob.receives_nothing().await);

    // The dialer gives up on the handshake and drops the session it had.
    let reason = alice.rejected().await;
    assert!(reason.contains("handshake"), "unexpected refusal: {reason}");
    alice.closed().await;
    alice.send(&ip_packet(b"dialer, without a session")).await;

    // The path comes back and the dialer reconnects on its own.
    tokio::time::sleep(Duration::from_millis(300)).await;
    relay.restore();
    let (_, recovered, role) = alice.session().await;
    let recovery = lost.elapsed();
    assert_eq!(role, TunnelRole::Initiator);
    assert_ne!(
        recovered, half_open,
        "CONFIRM reached the dialer; the test did not lose it"
    );
    assert_ne!(recovered, first);

    let mut closed_because = Vec::new();
    let at_bob = bob
        .event(|e| match e {
            VpnEvent::SessionClosed { reason } => {
                closed_because.push(reason.clone());
                None
            }
            VpnEvent::SessionEstablished { session_id, .. } => Some(*session_id),
            _ => None,
        })
        .await;
    assert_eq!(at_bob, recovered);
    assert!(
        recovery < HANDSHAKE_TIMEOUT + REDIAL_MIN + Duration::from_secs(4),
        "recovery took {recovery:?}"
    );

    // The listener speaks first, and in the session the dialer has: not in
    // the one whose CONFIRM was lost.
    let packet = ip_packet(b"listener speaks first after the recovery");
    bob.send(&packet).await;
    assert_eq!(alice.receive().await, packet);
    exchange(&mut alice, &mut bob, b"recovered session").await;
    // The listener dropped the session whose CONFIRM was lost exactly once:
    // because the dialer's close still reached it once the path was back, or
    // because the dialer came back on a new connection first.
    assert_eq!(closed_because.len(), 1, "closures: {closed_because:?}");
    assert!(
        ["connection closed", "the peer reconnected"].contains(&closed_because[0].as_str()),
        "closures: {closed_because:?}"
    );
    assert!(
        alice.receives_nothing().await && bob.receives_nothing().await,
        "a packet from the outage arrived"
    );

    alice.shut_down().await;
    bob.shut_down().await;
}

/// The dialer disappears without closing its connection and comes back with
/// the same identity before the listener has noticed. The listener must not
/// treat the new handshake as a renewal of the session it still holds: the
/// new dialer has no keys for that session, so anything sent in it is lost.
#[tokio::test]
async fn a_dialer_that_comes_back_before_the_listener_noticed_gets_a_fresh_session() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let listener = bob.listening().await;
    let dial = |relay: &Relay| VpnMode::Dial {
        addr: relay.addr.clone(),
        peer: bob_key,
    };

    let old_path = Relay::start(&listener, Duration::ZERO).await;
    let mut alice = start(&alice_seed, &[bob_key], dial(&old_path), SLOW);
    let (_, first, _) = alice.session().await;
    assert_eq!(bob.session().await.1, first);
    exchange(&mut alice, &mut bob, b"first session").await;

    // The dialer is gone at once and nothing it sends on the way out arrives.
    old_path.cut();
    alice.task.abort();
    let _ = alice.task.await;

    // It comes back on another path while the listener still holds the old
    // connection.
    let new_path = Relay::start(&listener, Duration::ZERO).await;
    let mut alice = start(&alice_seed, &[bob_key], dial(&new_path), SLOW);
    let (_, second, _) = alice.session().await;
    assert_ne!(second, first);
    let mut closed_because = Vec::new();
    let at_bob = bob
        .event(|e| match e {
            VpnEvent::SessionClosed { reason } => {
                closed_because.push(reason.clone());
                None
            }
            VpnEvent::SessionEstablished { session_id, .. } => Some(*session_id),
            _ => None,
        })
        .await;
    assert_eq!(at_bob, second);
    assert_eq!(
        closed_because,
        ["the peer reconnected"],
        "the listener was meant to still hold the old connection"
    );

    // The listener speaks first, in the session the new dialer has.
    let packet = ip_packet(b"listener speaks first to the new dialer");
    bob.send(&packet).await;
    assert_eq!(alice.receive().await, packet);
    exchange(&mut alice, &mut bob, b"second session").await;

    // When the old connection finally times out, the session stays.
    assert!(alice.receives_nothing().await && bob.receives_nothing().await);
    exchange(&mut alice, &mut bob, b"still the second session").await;

    alice.shut_down().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn an_unpinned_dialer_gets_no_session_and_delivers_nothing() {
    let (_, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let (mallory_seed, _) = seed();

    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    // Mallory has Bob's address and public key, and pins him.
    let mut mallory = start(
        &mallory_seed,
        &[bob_key],
        VpnMode::Dial {
            addr,
            peer: bob_key,
        },
        SLOW,
    );
    mallory.send(&ip_packet(b"from mallory")).await;

    assert!(bob.rejected().await.contains("not a pinned identity"));
    assert!(!mallory.gets_a_session().await);
    assert!(!bob.gets_a_session().await);
    assert!(bob.receives_nothing().await);

    mallory.shut_down().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn a_dialer_does_not_accept_another_identity_at_the_address() {
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let (_, carol_key) = seed();

    // Bob listens and has pinned Alice. Alice has pinned Bob and Carol, and
    // dials Bob's address expecting Carol.
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;
    let mut alice = start(
        &alice_seed,
        &[bob_key, carol_key],
        VpnMode::Dial {
            addr,
            peer: carol_key,
        },
        SLOW,
    );
    alice.send(&ip_packet(b"for carol only")).await;

    assert!(!alice.gets_a_session().await);
    assert!(!bob.gets_a_session().await);
    assert!(bob.receives_nothing().await);

    alice.shut_down().await;
    bob.shut_down().await;
}

#[tokio::test]
async fn a_pinned_transport_identity_without_the_handshake_delivers_nothing() {
    // The peer below holds a pinned key and completes the libp2p handshake
    // with it, but then skips or botches the CrypRQ handshake. The transport
    // connection alone must not let a single packet through.
    let (alice_seed, alice_key) = seed();
    let (bob_seed, bob_key) = seed();
    let mut bob = start(&bob_seed, &[alice_key], VpnMode::Listen(loopback()), SLOW);
    let addr = bob.listening().await;

    let alice = NodeIdentity::from_secret_seed(&alice_seed);
    let mut raw = build_swarm(&alice).expect("swarm");
    let bob_id = peer_id_of(&bob_key).expect("peer id");
    raw.dial(DialOpts::peer_id(bob_id).addresses(vec![addr]).build())
        .expect("dial");
    timeout(WAIT, async {
        loop {
            if let SwarmEvent::ConnectionEstablished { .. } = raw.select_next_some().await {
                break;
            }
        }
    })
    .await
    .expect("transport connection");

    let attempts: [Vec<u8>; 4] = [
        ip_packet(b"raw ip packet"),
        vec![0x01; 80],
        [b"CRQH\x02\x22\x00\x04".as_slice(), &[0u8; 4]].concat(),
        [b"CRQH\x02\x20\x00\x04".as_slice(), &[0u8; 4]].concat(),
    ];
    for attempt in attempts {
        raw.behaviour_mut().send_request(&bob_id, attempt);
    }
    let drive = async {
        loop {
            raw.select_next_some().await;
        }
    };
    let observe = async {
        for _ in 0..3 {
            bob.rejected().await;
        }
        assert!(!bob.gets_a_session().await);
        assert!(bob.receives_nothing().await);
    };
    tokio::select! {
        _ = drive => unreachable!(),
        _ = observe => {}
    }
    bob.shut_down().await;
}

#[tokio::test]
async fn the_link_refuses_to_start_without_trust_or_towards_an_unpinned_peer() {
    let (alice_seed, _) = seed();
    let (_, bob_key) = seed();
    let (_, carol_key) = seed();

    let run = |trusted: Vec<PeerPublicKey>, mode: VpnMode| {
        let mut peer = start(&alice_seed, &trusted, mode, SLOW);
        async move {
            let _ = peer.stop.take();
            timeout(WAIT, peer.task)
                .await
                .expect("stops")
                .expect("task")
        }
    };
    assert!(run(vec![], VpnMode::Listen(loopback())).await.is_err());
    let dial = |peer| VpnMode::Dial {
        addr: "/ip4/127.0.0.1/udp/9/quic-v1".parse().expect("multiaddr"),
        peer,
    };
    assert!(run(vec![bob_key], dial(carol_key)).await.is_err());
    let own_key = NodeIdentity::from_secret_seed(&alice_seed).public_key();
    assert!(run(vec![bob_key, own_key], dial(own_key)).await.is_err());
}
