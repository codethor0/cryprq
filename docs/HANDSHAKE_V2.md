# Authenticated handshake, wire version 2

This document describes the handshake that establishes a CrypRQ UDP tunnel
(`node::connect_tunnel` and `node::accept_tunnel`), the session that follows
it, and how to use it from the command line. The implementation is in
`node/src/auth_handshake.rs`, `node/src/establish.rs`, `node/src/identity.rs`
and the `Session` type in `node/src/lib.rs`.

It applies to the UDP tunnel used by `cryprq send-file` and
`cryprq receive-file`, and to the VPN link of `cryprq --vpn`, which carries
the same messages and records over libp2p (see [VPN link](#vpn-link)).
`--listen` and `--peer` without `--vpn` open a plain libp2p connection that
does not run this handshake and carries no tunnel traffic.

## Security goals

- Mutual authentication of two long-term Ed25519 identities. A node talks only
  to peers whose public key it has pinned explicitly. The set of pinned peers
  is empty by default, and nothing is trusted on first use.
- Hybrid key agreement: the session secret depends on both ML-KEM-768 and
  X25519, so it stays secret as long as either holds.
- The session keys are bound to the whole transcript: protocol version, suite,
  both identities, both key shares, the KEM ciphertext and both signatures.
- Explicit key confirmation in both directions. No tunnel exists, and no
  application data is accepted, before the handshake has completed.
- Forward secrecy with respect to the identity keys: the key shares are
  ephemeral, and the identity keys only sign.

Not provided: identity hiding (identity keys are sent in the clear), and
protection of the long-term identity keys against a quantum adversary
(Ed25519 signatures are classical; an attacker must break them during the
handshake to impersonate a peer, recorded traffic is not affected).

## Identities and trust

| Item | Format |
| --- | --- |
| Identity key file | `cryprq-ed25519-identity-v1`, newline, 64 hex digits (the Ed25519 seed), newline. Mode 0600 in a directory that is not accessible to others. |
| Public key | `ed25519:` followed by 64 hex digits |
| Trusted-peers file | One public key per line, optionally followed by a label. Blank lines and lines starting with `#` are ignored. Must not be writable by group or others. |

An identity is created only by `cryprq identity init`. Loading fails closed
when the file is missing, malformed, a symlink, or accessible to other users;
a key is never generated implicitly. Any invalid or duplicate line rejects the
whole trusted-peers file.

## Messages

Every handshake message starts with an 8-byte header:

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | Magic `CRQH` |
| 4 | 1 | Version, `0x02` |
| 5 | 1 | Message type |
| 6 | 2 | Body length, big-endian |

A record starts with the record protocol version `0x01`, so a handshake
message is never mistaken for a record. The length of every message is fixed
by its type; any other length is rejected.

| Message | Type | Direction | Body | Datagram |
| --- | --- | --- | --- | --- |
| INIT | `0x20` | initiator to responder | suite (2), random (32), initiator identity (32), X25519 share (32), ML-KEM-768 encapsulation key (1184) | 1290 bytes |
| RESPONSE | `0x21` | responder to initiator | suite (2), random (32), responder identity (32), X25519 share (32), ML-KEM-768 encapsulation key (1184), signature (64) | 1354 bytes |
| FINISH | `0x22` | initiator to responder | ML-KEM-768 ciphertext (1088), signature (64), key confirmation (32) | 1192 bytes |
| CONFIRM | `0x23` | responder to initiator | key confirmation (32) | 40 bytes |

The only suite is `0x0001`: ML-KEM-768 + X25519, Ed25519, HKDF-SHA256 and
HMAC-SHA256, ChaCha20-Poly1305. Unknown versions and suites are rejected; there
is no negotiation and therefore no downgrade.

## Transcript and signatures

The transcript is a running SHA-256 hash. Each element is absorbed as
`len(label) || label || len(data) || data` with 32-bit big-endian lengths, in
this order:

1. `domain`: `cryprq authenticated hybrid handshake`
2. `version`, `suite`
3. `initiator identity`, `responder identity`
4. `init`: the complete INIT message
5. `response`: RESPONSE without its signature
6. `responder signature`
7. `kem ciphertext`
8. `initiator signature`
9. `initiator confirmation`

The responder signs `"cryprq handshake v2 responder signature" || hash` after
element 5. The initiator signs
`"cryprq handshake v2 initiator signature" || hash` after element 7. The two
contexts differ, so a signature cannot be reused in the other role.

Signatures are verified strictly against the locally pinned key. The initiator
names the identity it expects before sending anything and accepts no other;
the responder accepts an INIT only from a pinned identity.

## Key schedule

```text
PRK              = HKDF-Extract("cryp-rq v2 hs", ss_kem || ss_x)
H                = transcript hash after element 8
confirm_key_i    = HKDF-Expand(PRK, "cryp-rq v2 confirm key initiator" || H, 32)
confirm_key_r    = HKDF-Expand(PRK, "cryp-rq v2 confirm key responder" || H, 32)
master_secret    = HKDF-Expand(PRK, "cryp-rq v2 master secret" || H, 32)
session_id       = HKDF-Expand(PRK, "cryp-rq v2 session id" || H, 16)

confirm_i        = HMAC-SHA256(confirm_key_i, "initiator key confirmation", H)
confirm_r        = HMAC-SHA256(confirm_key_r, "responder key confirmation", H')
```

`H'` is the transcript hash after element 9. X25519 shares that produce an
all-zero shared secret are rejected.

Traffic keys are derived per epoch and per direction from `master_secret`
with the epoch-scoped labels of the v1 key schedule
(`derive_epoch_keys`): one key and one IV for initiator to responder and one
pair for responder to initiator.

## Failure handling

A message that fails a check made before any state is committed (framing,
version, suite, identity, signature) is dropped and the handshake continues.
Such datagrams can be forged by anyone, so they must not be able to abort a
handshake. A failure after the peer has been authenticated (key exchange or
key confirmation) is terminal.

Both sides retransmit: the initiator resends INIT or FINISH every 500 ms until
the handshake times out (10 s by default), the responder answers a repeated
INIT with the same RESPONSE, and for 30 s after completion answers the exact
FINISH that completed the session with the same CONFIRM.

A listening responder keeps at most 16 half-open handshakes, each for at most
the handshake timeout, processes at most 200 datagrams per second before a
session exists, and does not reply to anything that is not a valid INIT from a
pinned identity.

## Session

- Records are ChaCha20-Poly1305 with the 20-byte record header as associated
  data. The nonce is the direction's IV XOR the 64-bit sequence number.
- Each direction has one sequence space per epoch, shared by all message types
  and streams. A (key, nonce) pair is never used twice: sending fails with
  `NonceOverflow` at `u64::MAX - 1000`, and the way out is key rotation.
- Key rotation moves one sending direction to the next epoch: new key, new IV,
  sequence number 0. The receiver follows when a record of a later epoch
  authenticates; a forged header cannot move it. The epoch it left stays
  usable for 10 s to absorb reordering and is then discarded. Rotation runs
  every 300 s and can be requested with `Tunnel::rotate_keys`.
- The epoch is 8 bits and does not wrap. After epoch 255 the session fails
  with `SessionExpired` and a new handshake is required.
- Each receiving epoch has a 2048-entry replay window. A record is checked in
  this order: rate limit, AEAD, replay window. Only a record that passes all
  three updates the peer address.

## VPN link

`cryprq --vpn` (`p2p/src/secure_vpn.rs`) runs the same handshake and the same
session over a libp2p connection (QUIC or TCP) instead of bare UDP.

- libp2p is a transport only. A connection, a peer id or a completed QUIC/TLS
  handshake does not admit a peer: IP packets are exchanged only as
  `VPN_PACKET` records of a session that both sides confirmed. Until then, and
  whenever the session is lost, packets from the local device are dropped.
  There is no unauthenticated or plaintext fallback.
- The node's libp2p key is its CrypRQ identity key, so the transport peer id
  follows from the pinned key. Connections from peer ids that belong to no
  pinned key are closed, and the identity claimed in INIT must be the
  transport peer. This is defence in depth; authentication is the signature
  and key confirmation of the handshake.
- Messages use the request-response protocol `/cryprq/vpn/2.0.0`: INIT is
  answered by RESPONSE, FINISH by CONFIRM, and a record by an empty response.
  Requests are limited to one record around a 65535-byte packet, a listener
  keeps at most 16 half-open handshakes, and at most 1024 records are in
  flight.
- An endpoint has one session. The dialer names the pinned peer it expects
  (`--peer-key`, optional when exactly one peer is pinned), dials again with
  backoff whenever the connection is lost, and replaces the session by a new
  handshake every `CRYPRQ_REHANDSHAKE_SECS` (default 3600). Sending keys
  rotate every `CRYPRQ_ROTATE_SECS` (default 300).
- When a side has used up its epochs it stops sending. A dialer starts a new
  handshake on the same connection; a listener closes the connection, and the
  dialer comes back with a new handshake.
- A renewal is a new handshake on a connection that already has a session.
  Each side installs the new session when it has verified the other side's
  signature and key confirmation: the listener on FINISH, the dialer one
  message later on CONFIRM. Neither side holds a session the other has not
  confirmed, and a renewal that fails at any message leaves the existing
  session in place on the side that rejected it. Around the switch:
  - The replaced session is kept for 10 s for receiving only, together with
    its replay window, so records in flight arrive once and a record that
    was already delivered stays a replay. After 10 s its keys are wiped, on
    a timer, whether or not traffic arrives.
  - The dialer sends in the new session as soon as it has it.
  - The listener keeps sending in the replaced session until a record of
    the new session arrives from the dialer, and for at most 5 s. Without
    this, packets it sent between FINISH and CONFIRM could not be decrypted
    by the dialer. Sequence numbers of the replaced session continue, so no
    nonce is reused. If the replaced session can no longer send, the new one
    is used at once.
- A dialer opens a connection only when it has none, and by then it has
  dropped its session. When a pinned peer opens a new connection while the
  node still holds an older one to it, the node closes the older connection
  and drops the session it held for that peer (`reason="the peer
  reconnected"`). The handshake that follows starts from nothing: it is not
  treated as a renewal, and nothing is sent on the dead connection or in a
  session the peer no longer has. Only the pinned peer can cause this, since
  the transport authenticates the new connection as that identity.
- If the path fails during a renewal after the listener has installed the new
  session and before CONFIRM reaches the dialer, the listener holds a session
  the dialer never got. The dialer keeps the old session, gives up on the
  handshake after the handshake timeout (10 s by default), closes the
  connection, which drops its session, and dials again after 1 s (doubling up
  to 30 s while the peer stays unreachable). Until then each side accepts only
  records of a session it holds itself; after the reconnect both hold one new
  session and the listener may send first.
- On SIGTERM or Ctrl+C the node closes its connections, so the peer drops the
  session at once.

IP packets come from a TUN device, or with `--packet-bridge 127.0.0.1:PORT`
from a loopback UDP socket that stands in for one (userspace network stacks,
tests without root). On macOS the device is always named `utun<N>`; a
`--tun-name` of another form lets the system choose.

The TUN path fails closed:

- The address, netmask and MTU are checked before a device is requested.
  Reserved addresses, non-contiguous netmasks, subnets wider than /8 or
  narrower than /30 and an MTU below 576 are refused, so a mistyped netmask
  cannot turn the tunnel's connected route into a default route.
- `--tun-netmask` sets the netmask (default `255.255.255.0`). The connected
  route of the device covers exactly that subnet, so `255.255.255.252` keeps
  it to the two addresses of the link.
- If the device cannot be created or its address cannot be configured, the
  node exits with an error instead of running a link that forwards nothing.
- If the device goes away while the link is running, the node closes its
  connections, so the peer drops the session, and exits with an error.

Before using a TUN device on macOS, run `scripts/vpn_preflight_macos.py
[--tun-address A] [--netmask M] [--tun-name N]`. It only reads the interface
list, the routing tables, the DNS resolvers and the VPN services, and exits
with status 1 if the tunnel address is already assigned, its subnet overlaps
an interface or a route, or the requested `utun<N>` exists. It also reports
an active VPN, and that IPv6 traffic and DNS lookups do not use the tunnel.
If the interface list or the routing table cannot be read, that is a failure,
not a clean result. With `--suggest`, or after a conflict, it names a /30
subnet that is free at that moment; check that address again immediately
before using it.

```bash
# Listener
cryprq --vpn --listen /ip4/0.0.0.0/udp/9999/quic-v1
# Dialer
cryprq --vpn --peer /ip4/203.0.113.7/udp/9999/quic-v1 --peer-key ed25519:<listener key>
```

`--vpn --test-mode` keeps the previous libp2p link without the handshake. It
is refused unless `CRYPRQ_TEST_MODE` is set to `1` or `true`.

## Command line

```sh
# On each node: create the identity and print the public key.
cryprq identity init
cryprq identity show

# On each node: pin the other node's key.
echo "ed25519:<peer key> laptop" >> ~/.cryprq/trusted_peers

# Receiver
cryprq receive-file --listen /ip4/0.0.0.0/udp/20440 --output-dir ./inbox

# Sender
cryprq send-file --peer /ip4/203.0.113.7/udp/20440 --file report.pdf \
    --peer-key ed25519:<receiver key>
```

`--identity` and `--trusted-peers` select other locations than
`~/.cryprq/identity.key` and `~/.cryprq/trusted_peers`. `--peer-key` may be
omitted when exactly one peer is pinned.

`--test-mode` selects publicly known fixed keys instead of the handshake. It
provides no security and works only when `CRYPRQ_TEST_MODE` is set to `1` or
`true` on both ends.

## Known limitations

- A receiver serves one session per process.
- File transfer has no acknowledgement or retransmission; a lost datagram
  leaves the received file incomplete. The sender paces itself below the
  receiver's rate limit but does not learn whether the file arrived.
- A UDP tunnel session that has used up its epochs is not re-established
  automatically. The VPN link does re-establish it.
- The VPN link creates the TUN device and assigns its address. It installs no
  routes, DNS settings or firewall rules, so by itself it does not send the
  host's traffic through the tunnel and gives no protection against leaks.
- The TUN path has been verified on Linux only, between two containers:
  `scripts/e2e-tun-docker.sh` sends ICMP and TCP from one kernel through its
  TUN device, the link and the other TUN device to the other kernel, in both
  directions, and checks that the wire carries only the link's datagrams.
  macOS utun devices have been exercised only through packet framing and
  address checks in unit tests; creating one needs root on the host and has
  not been done. See `docs/VPN_TUN_ISOLATED_TEST_PLAN.md`.
- `cryprq` itself does not look for address or route conflicts on the host;
  that is what the preflight script is for. The default `--tun-address
  10.0.0.1` lies in a range that other VPNs commonly use.
- One VPN session per endpoint: a listener serves one dialer at a time.
- If CONFIRM is lost during a renewal, packets are not carried for the time it
  takes the dialer to notice and reconnect: up to the handshake timeout plus
  the redial delay, about 11 s with the defaults once the path works again.
  Packets sent in that time are dropped, never sent unprotected. On QUIC or
  TCP a lost CONFIRM means the connection itself failed.

## Verification

- `cargo test -p node` covers identity storage, the handshake state machines
  (including bit flips in every message, replay of earlier sessions, key-share
  substitution and impersonation) and the session (nonce and epoch exhaustion,
  rekeying, replay and key isolation), on sessions in memory and on two
  tunnels over loopback UDP.
- `scripts/e2e-authenticated-file-transfer.sh` runs two separate processes
  with separate identities and checks an authenticated file transfer together
  with the cases that must fail.
- `cargo test -p p2p secure_vpn` covers the VPN link: its state machine with
  the test acting as a transport that reads, changes, replays and injects
  every message, and two engines over QUIC on loopback. Two of the network
  tests put a UDP relay between the engines that can cut the path: one cuts
  it while the CONFIRM of a renewal is in transit and checks that the dialer
  reconnects within the handshake timeout plus the redial delay, the other
  removes the dialer without a close and brings it back before the listener
  has noticed.
- `scripts/e2e-authenticated-vpn.sh` runs two separate `cryprq --vpn`
  processes through `--packet-bridge` and a UDP relay that duplicates,
  replays and corrupts datagrams, and checks packet delivery, key rotation,
  session renewal (with packets crossing the link without pause while the
  session is renewed), shutdown, recovery and the configurations that must
  be refused. It needs bash and python3, no root, and only loopback.
- `scripts/e2e-tun-docker.sh` is the device test: two `cryprq --vpn`
  endpoints on real TUN devices in two Linux containers on an internal
  Docker network, plus a third container with an unpinned identity. It needs
  Docker, gives the containers `NET_ADMIN` and `/dev/net/tun` only, and
  compares the host's routes, DNS, interfaces and Docker's state before and
  after. 71 checks.
- `python3 -m unittest discover -s scripts/tests -p
  test_vpn_preflight_macos.py` covers the preflight's parsing and conflict
  detection on recorded tool output.
