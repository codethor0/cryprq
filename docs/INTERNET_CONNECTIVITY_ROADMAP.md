# Internet connectivity roadmap

This document lists goals. None of them is implemented, and nothing here is a
statement about what CrypRQ does today.

## Where the project stands

- The authenticated file transfer and the VPN link have been tested between
  two processes on one machine and between containers on one Docker network.
  **Connectivity across the internet (WAN) has not been validated**: no test
  covers NAT, firewalls, packet loss, reordering or changing addresses.
- There are two separate network paths in the code, and they should not be
  confused:
  - **The authenticated UDP transfer** used by `send-file` and `receive-file`
    (and by the browser portal). It runs the CrypRQ handshake directly over a
    UDP socket to an address the user supplies. It has no NAT traversal, no
    relay, no acknowledgement and no retransmission.
  - **The libp2p transport** (QUIC and TCP) used by the legacy connection mode
    and by the `--vpn` link. It provides connection handling, but no relay,
    hole punching or peer discovery component is included in the build.
- A device is reached by an IPv4 address and UDP port that the other side must
  already know and be able to reach.

## Goals

The order is the intended order of work. Each item needs its own design,
threat analysis and tests before it is described as working.

### 1. Direct connectivity over the internet

Validate the existing paths between hosts on different networks with public or
forwarded addresses: IPv4 and IPv6, realistic loss and delay, address changes
during a session. Decide whether file transfer stays on its own UDP path or
moves onto the libp2p transport. Until this is done, every later item rests on
an untested base.

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

File transfer currently has no acknowledgement and no retransmission: a sender
that finishes has not proven delivery. The goal is an authenticated
acknowledgement from the receiver after it has verified the file, bounded
retransmission of lost data, and resumption of an interrupted transfer, all
inside the existing authenticated session and without weakening replay
protection.

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
