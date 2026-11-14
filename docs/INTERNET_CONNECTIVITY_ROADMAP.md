# Internet connectivity roadmap

This document separates the additive Phase 1 direct-file candidate from the
remaining connectivity goals. The candidate adds explicit direct QUIC/TCP file
commands; it does not establish that transfer across separate internet networks
works. **WAN TEST NOT VERIFIED.** See the [direct transfer guide](DIRECT_FILE_TRANSFER.md)
for its interface, security boundaries and validation record.

## Where the project stands

- The authenticated file transfer and the VPN link have been tested between
  two processes on one machine and between containers on one Docker network.
  **Connectivity across the internet (WAN) has not been validated**: no test
  covers NAT, firewalls, packet loss, reordering or changing addresses.
- There are separate network paths in the code, and they should not be
  confused:
  - **The authenticated UDP transfer** used by `send-file` and `receive-file`
    (and by the browser portal). It runs the CrypRQ handshake directly over a
    UDP socket to an address the user supplies. It has no NAT traversal, no
    relay, no acknowledgement and no retransmission.
  - **The libp2p transport** (QUIC and TCP) used by the legacy connection mode
    and by the `--vpn` link. It provides connection handling, but no relay,
    hole punching or peer discovery component is included in the build.
  - **The opt-in direct-file candidate**, `direct-send-file` and
    `direct-receive-file`, carries the existing authenticated application
    handshake and encrypted file records over libp2p QUIC/TCP. It requires an
    explicit IP multiaddr and pinned expected identity on both ends, transfers
    one file and returns an encrypted receiver receipt after size/hash
    verification and atomic publication. Local and hosted results must be read
    from the candidate's validation record, not inferred from this roadmap.
- Direct modes require an address and port the sender already knows and can
  reach. No command provisions public endpoints or changes host networking.

## Goals

The order is the intended order of work. Each item needs its own design,
threat analysis and tests before it is described as working.

### 1. Direct connectivity over the internet

The Phase 1 candidate adds a separate libp2p direct-file path while preserving
the existing UDP file commands and portal. Validate this path between two
authorized hosts on genuinely different internet networks with an already
reachable address. Record the exact revision, application authentication,
route and SHA-256 evidence using the [WAN recipe](DIRECT_FILE_TRANSFER.md#separate-network-wan-test-recipe).
No WAN infrastructure is approved for the current implementation session, so
that milestone remains unverified. Loopback and a shared Docker host do not
satisfy it. IPv6 reachability, realistic loss/delay and changing addresses also
need their own evidence; no universal NAT-success claim follows from a direct
test.

### 2. NAT traversal

Let two devices behind NAT connect without manual port forwarding: discovery
of the externally visible address, simultaneous connection attempts (hole
punching), and clear reporting when the NAT type makes a direct path
impossible. Traversal needs a rendezvous point, which learns the addresses of
both devices; that exposure has to be documented.

### 3. Self-hostable relay fallback

When no direct path exists, forward traffic through a relay that users can run
themselves. The relay must only see ciphertext of the authenticated session
and must not be trusted for confidentiality or integrity. It does see who
talks to whom, when, and how much. Resource limits and abuse controls are part
of the design.

### 4. Delivery acknowledgements and retries

The existing UDP file commands and portal have no application acknowledgement
or retransmission: a sender that finishes has not proven delivery. The
direct-file candidate adds ordered encrypted replies and a final receiver
receipt after verified publication; QUIC/TCP supplies transport reliability.
If the last receipt is lost, the receiver may already have published the file
while the sender reports failure. Application retries, transfer recovery and
authenticated resumption remain future work and require separate review. They
must not weaken replay protection or overwrite existing files.

### 5. Optional encrypted store-and-forward

Allow a file to be left for a device that is offline and retrieved later. The
storage node would hold only ciphertext it cannot read. This requires key
agreement with an absent peer, which the current interactive handshake does
not provide, so it is a protocol extension with its own review, not a
configuration option. It would be opt-in.

### 6. Research: I2P

Investigate whether CrypRQ sessions can run over I2P for users who need to
hide network addresses. This is research only: feasibility, performance and
the resulting threat model are open questions, and no commitment to ship it
exists.

## Confidentiality is not anonymity

CrypRQ aims at confidentiality and integrity of the content between pinned
peers. It does not hide who communicates:

- peers see each other's IP addresses, and so does anyone on the path;
- identity keys are sent in the clear during the handshake;
- a rendezvous point or relay learns addresses, timing and volume;
- traffic analysis is not addressed.

Relays and store-and-forward nodes would change who can observe this metadata;
they do not remove it. Anonymity would require an anonymity network such as
the one considered in item 6, and would be claimed only after it has been
designed, implemented and evaluated.

## What will not change

- Peers stay explicitly pinned. No item introduces trust on first contact.
- Infrastructure (rendezvous, relay, storage) is never trusted with plaintext
  or keys.
- A capability is documented as available only after tests demonstrate it.
