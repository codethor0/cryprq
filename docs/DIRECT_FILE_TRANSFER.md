# Opt-in direct file transfer: Phase 1 candidate

This candidate adds `direct-send-file` and `direct-receive-file`. It carries one
authenticated file transfer over an explicitly selected, reachable libp2p QUIC
or TCP address. Both nodes run CrypRQ and independently pin each other's
Ed25519 public identity. The existing UDP file commands, browser portal and
VPN/TUN modes retain their existing interfaces and defaults.

**WAN TEST NOT VERIFIED.** No separate-network test hosts or public endpoints
have been approved for this implementation session. Local processes and Docker
containers on one host are useful regression tests, not internet witnesses.
This remains an unaudited research prototype, not a production-readiness claim.

## Interface and prerequisites

Build the same reviewed revision on both nodes:

```bash
cargo build --release --locked -p cryprq
git rev-parse HEAD
```

Each node needs its own private identity file and a trust file containing the
other node's independently verified public key. `--identity` and
`--trusted-peers` accept explicit paths; otherwise the existing defaults are
`~/.cryprq/identity.key` and `~/.cryprq/trusted_peers`. Missing identity files,
empty trust, an unpinned expected key and the local node's own key are rejected.
There is no implicit key generation or trust on first use.

Provision an identity only if the intended identity file does not already exist:

```bash
./target/release/cryprq --identity ./direct-state/identity.key identity init
./target/release/cryprq --identity ./direct-state/identity.key identity show
```

Exchange and compare the printed public keys over a trusted channel, then put
the remote key, in `ed25519:<hex>` form, on its own line in
`./direct-state/trusted_peers`. Never exchange private identity files. Each
direct command requires `--peer-key` even when the trust file has only one key.

| Command | Required arguments |
| --- | --- |
| `direct-send-file` | `--peer MULTIADDR --peer-key KEY --file PATH` |
| `direct-receive-file` | `--listen MULTIADDR --peer-key KEY --output-dir DIRECTORY` |

Supported address shapes are exactly:

```text
/ip4/ADDRESS/udp/PORT/quic-v1
/ip6/ADDRESS/udp/PORT/quic-v1
/ip4/ADDRESS/tcp/PORT
/ip6/ADDRESS/tcp/PORT
```

DNS, `/p2p` suffixes, circuit/relay components and additional components are
rejected. The independently pinned key determines the transport peer ID; an
address cannot select a different identity. IP address support does not prove
IPv6 or internet reachability on a particular network.

The receiver's output directory must already exist, must be a directory rather
than a symlink, and must remain under the operator's control along with its
ancestors. Do not use a shared directory whose entries or ancestry an
untrusted local user can replace. Existing destination files and symlinks are
rejected; received names cannot select another directory. Source files must be
regular files with valid single-component names.

`--test-mode` and an enabled `CRYPRQ_TEST_MODE` environment variable are both
rejected for these commands. There is no insecure fallback. The listener is
created only by `direct-receive-file`; starting CrypRQ normally does not enable
this mode or expose the local portal.

## Local example

Create two separate identity/trust stores as above. Set `SENDER_KEY` and
`RECEIVER_KEY` to their independently checked public keys and pin them on the
opposite nodes. Use separate terminals:

```bash
# Receiver B: create a private output directory once.
mkdir -m 700 ./b/inbox
./target/release/cryprq --identity ./b/identity.key --trusted-peers ./b/trusted_peers \
  direct-receive-file --listen /ip4/127.0.0.1/udp/20440/quic-v1 \
  --peer-key "$SENDER_KEY" --output-dir ./b/inbox
```

```bash
# Sender A:
./target/release/cryprq --identity ./a/identity.key --trusted-peers ./a/trusted_peers \
  direct-send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 \
  --peer-key "$RECEIVER_KEY" --file ./example.txt
```

For TCP, replace both addresses with `/ip4/127.0.0.1/tcp/20440`.
Each process exits after one verified file. The receiver waits up to 300 seconds
for the expected transport peer. Requests and idle application waits are bounded
to 15 seconds; the application handshake also uses the existing configured
handshake deadline. Ctrl-C or SIGTERM cancels the command and drops its state.

On success, the CLI prints:

```text
route=direct peer_id=... bytes=... sha256=...
```

`route=direct` describes the selected transport. It does not distinguish a
loopback address from an internet route and is not WAN evidence by itself.

## Docker

`docker-compose.direct.yml` runs the same CLI from the portal image
(`Dockerfile.web`) as two independent nodes, `node-a` and `node-b`, plus an
unpinned `node-c` for rejection tests. Each node keeps its identity, trust
store and inbox in its own named volume. The containers run as the non-root
`node` user with a read-only root filesystem, all capabilities dropped,
`no-new-privileges`, memory/CPU/process limits, and an `internal` network.
No port is published, the portal is not started, and no host networking,
privileged mode or Docker socket is used.

```bash
docker build -t cryprq-web:latest -f Dockerfile.web .
bash scripts/e2e-direct-docker.sh
```

The script creates and pins three identities, checks that they persist in
their volumes, checks the container posture on a live receiver, delivers a
nonempty and an empty file over QUIC and TCP with matching SHA-256 receipts,
rejects an unexpected sender and an unexpected receiver key, refuses
`CRYPRQ_TEST_MODE`, and checks that SIGTERM stops a waiting receiver with
nothing published. It removes its containers, network and volumes on exit.

All containers share one Docker host, so this is packaging and isolation
evidence, not internet evidence. Container-to-container traffic on Docker
Desktop stays inside its Linux VM; host-published UDP ports on Docker Desktop
are a separate path that only a WAN test can exercise. Publishing a peer port
for such a test requires separate approval and does not make a host
reachable through its router.

## Relayed route (opt-in)

When neither peer can reach the other, both can use a Circuit Relay v2 server
that they can reach and whose key they have verified. Replace the direct
address with `--relay ADDRESS --relay-key KEY` on both commands:

```bash
# Receiver: reserve on the relay and accept the sender only through it.
cryprq --identity ... --trusted-peers ... direct-receive-file \
  --relay /ip4/RELAY_IP/udp/20450/quic-v1 --relay-key "$RELAY_KEY" \
  --peer-key "$SENDER_KEY" --output-dir ./inbox
# Sender: reach the receiver only through the same relay.
cryprq --identity ... --trusted-peers ... direct-send-file \
  --relay /ip4/RELAY_IP/udp/20450/quic-v1 --relay-key "$RELAY_KEY" \
  --peer-key "$RECEIVER_KEY" --file ./example.txt
```

Both commands then print `route=relay`. The relay address uses the same
explicit IP/QUIC/TCP forms as a direct address. The relay key is pinned on the
command line and is never added to a trust file: the relay is not an
application peer and cannot authorize anything.

Security boundaries:

- The connection between the peers is upgraded with Noise inside the relay
  circuit and bound to the receiver's pinned transport key, and the unchanged
  hybrid handshake, key confirmation and record encryption run inside it. The
  relay forwards ciphertext; it has no session key and cannot impersonate a
  peer or alter records undetected.
- The relay sees metadata: both peer IDs, their IP addresses, timing and
  transfer size. It can drop, delay or cut a transfer. A cut transfer fails on
  both sides and the receiver removes its unpublished partial file.
- A relay with a different key than `--relay-key` is refused. A relay only
  connects a sender to a peer that holds a reservation, and the receiver still
  accepts only its expected peer, only through the relay.
- There is no NAT traversal or direct-connection upgrade in this version: a
  relayed transfer stays relayed.
- Both commands close their relay connection when they finish or fail. A
  command stopped with Ctrl-C or SIGTERM cannot, so a receiver that reserved
  over QUIC stays reserved on the relay until QUIC's 10-second idle timeout.
  A receiver restarted with the same identity within that window may see its
  first relayed attempt fail; retry after 10 seconds, or reserve over TCP.

`cryprq relay-serve` runs a bounded relay. It needs only its own identity file
(`cryprq --identity relay.key identity init`), loads no trust store and refuses
test mode. Every limit is explicit, because the library defaults (2 minutes and
128 KiB per circuit) are far below a file transfer:

| Option | Default | Meaning |
| --- | --- | --- |
| `--listen` (repeatable, required) | none | QUIC or TCP listen multiaddr |
| `--external-address` (repeatable) | concrete listen addresses | Address peers use to reach the relay |
| `--max-circuit-bytes` | 64 MiB | Bytes one circuit may carry before it is cut |
| `--max-circuit-secs` | 600 | Seconds one circuit may stay open |
| `--max-circuits` / `--max-reservations` | 16 / 32 | Concurrent circuits and reservations |

A transfer larger than the relay's byte limit, or slower than its time limit,
fails cleanly. Direct-file transfers acknowledge every 16 KiB chunk, so
throughput through a relay is about 16 KiB per round trip; size the limits
accordingly. The relay admits any peer that can reach it within these limits
(no allowlist yet), so do not expose it publicly without approval. It logs
operational events at `info` and peer IDs only at `debug`.

`docker-compose.relay.yml` puts `node-a` and `node-c` on one internal network,
`node-b` on another, and the relay on both, so the sides have no direct path:

```bash
docker build -t cryprq-web:latest -f Dockerfile.web .
bash scripts/e2e-direct-relay-docker.sh
```

The script checks container posture and that a direct attempt cannot reach the
other side. It then delivers files through the relay over QUIC and TCP (nonempty
and empty) with matching SHA-256 receipts and `route=relay` on both sides. It
rejects a relay with the wrong key, a sender asking for an unexpected peer, and
a sender the receiver does not expect. It checks that a circuit cut by the
relay's byte limit publishes nothing, and that `relay-serve` refuses test mode.
It is single-host evidence: **WAN TEST NOT VERIFIED**.

## Authentication and receiver safety

The transport reuses the existing Ed25519 identity to derive libp2p's peer ID.
Both sides then perform the unchanged signed, transcript-bound
ML-KEM-768 + X25519 CrypRQ application handshake, including suite/version checks,
fresh exchange material and mutual key confirmation. The expected application
identity is checked against the expected transport identity. A QUIC, TLS or
Noise connection event alone does not authorize file records.

File metadata, chunks and receiver replies use the existing authenticated
session record encryption with a separate versioned direct-file payload domain.
The adapter accepts one connection, one application session and one file at a
time. Ordered stop-and-wait replies bind the request sequence, accepted byte
count and announced SHA-256. The sender reports success only after authenticating
the receiver's completion reply for that exact size and hash.

The existing `FileTransferManager` remains responsible for file admission and
publication: maximum 1 GiB per file, declared-size bounds, storage reservation
before creating a temporary file, safe names, SHA-256 verification, private
temporary files and atomic publication without overwrite. Its existing aggregate
limits remain intact; this adapter admits only one file per process. Chunks are
at most 16 KiB and wire frames at most 16,512 bytes, with one active request to
provide bounded buffering and backpressure. Replayed, early, malformed,
out-of-order or invalidly authenticated records fail closed.

Failure, expiry, disconnect or cancellation removes an unpublished partial file
when receiver state is dropped. A verified file may already have been atomically
published when the final reply or connection close fails. In that case the sender
can report failure while the verified recipient file remains. Check both logs
and the recipient hash before retrying; the command does not overwrite an
existing file or resume a partial transfer.

QUIC/TCP provides transport reliability, but this phase adds no application retry,
resume, relay, NAT traversal, DHT/discovery, browser share link or portal UX.
There is no anonymity claim: peers and network observers can learn endpoint and
traffic metadata, and the existing application handshake transmits public
identities. Do not log private keys or file contents.

## Separate-network WAN test recipe

Run this only after the owner approves two controlled hosts on genuinely
different internet networks and an already reachable receiver endpoint. The
operators must provision and authorize the endpoint independently. These commands
do not change firewall, routing, DNS, TUN, port forwarding or cloud resources.
An unreachable endpoint should fail with a nonzero exit; do not substitute an
insecure mode or a same-host test and call it WAN.

1. On both hosts, check out the same reviewed Phase 1 revision, build with the
   command above, and record `git rev-parse HEAD`. Record the approved host/network
   labels, date, chosen IP/port, transport and how the receiver is already
   reachable. An independent operator should keep the recipient evidence.
2. Provision separate identities, compare public keys over a trusted channel,
   and pin each other. Use the explicit private paths below. Set `SENDER_KEY`
   on receiver B and `RECEIVER_KEY` on sender A to the verified remote key.
3. On receiver B, create a private output directory and start one listener.
   The wildcard listener below is intentional exposure on the approved host;
   choose the permitted bind address and port for that host.

   ```bash
   mkdir -m 700 ./direct-state/inbox
   set -o pipefail
   ./target/release/cryprq --identity ./direct-state/identity.key \
     --trusted-peers ./direct-state/trusted_peers \
     direct-receive-file --listen /ip4/0.0.0.0/udp/20440/quic-v1 \
     --peer-key "$SENDER_KEY" --output-dir ./direct-state/inbox \
     2>&1 | tee receiver-direct.log
   ```

4. On sender A, create a small non-sensitive test file with a destination name
   that does not exist in B's inbox. Record its size/hash and send it to B's actual
   approved address. `RECEIVER_PUBLIC_IP` is that address, not a documentation
   placeholder or an automatically discovered endpoint.

   ```bash
   python3 -c 'from pathlib import Path; p=Path("wan-test.bin"); p.open("xb").write(bytes(range(256))*1024)'
   python3 -c 'import hashlib,pathlib; p=pathlib.Path("wan-test.bin"); print(p.stat().st_size, hashlib.sha256(p.read_bytes()).hexdigest())'
   set -o pipefail
   ./target/release/cryprq --identity ./direct-state/identity.key \
     --trusted-peers ./direct-state/trusted_peers \
     direct-send-file --peer "/ip4/${RECEIVER_PUBLIC_IP}/udp/20440/quic-v1" \
     --peer-key "$RECEIVER_KEY" --file ./wan-test.bin \
     2>&1 | tee sender-direct.log
   ```

5. On B, independently calculate the published file's size and SHA-256:

   ```bash
   python3 -c 'import hashlib,pathlib; p=pathlib.Path("direct-state/inbox/wan-test.bin"); print(p.stat().st_size, hashlib.sha256(p.read_bytes()).hexdigest())'
   ```

   Preserve both command exit statuses, both `direct_file_authenticated` log
   events, both `route=direct` receipts, and matching source/recipient hashes.
   Inspect the inbox for leftover partials. Keep private keys and plaintext out of
   shared evidence. Repeat in the opposite direction with a distinct filename
   and a separately approved reachable listener. To test TCP, use the matching
   `/ip4/ADDRESS/tcp/PORT` addresses and record the separate result.
6. With owner-approved test identities, attempt a mismatched expected identity
   and an unapproved sender. Confirm nonzero sender status and no recipient
   file. Preserve failure evidence; a transport connection without both
   application-authentication events is not a successful transfer.

Until this recipe has been independently executed with recorded evidence, the
WAN outcome remains **NOT VERIFIED**. A successful direct test would establish
only that route and configuration; it would not establish NAT traversal or
universal internet connectivity.

## Validation commands and evidence

Run from the repository root, retaining failures and the exact tested revision:

```bash
cargo test -p p2p direct_file --locked
cargo test -p cryprq direct_cli_tests --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo deny check
cargo audit
cargo build --release --locked -p cryprq
CRYPRQ_E2E_KEEP_LOGS=1 python3 scripts/e2e-direct-file.py ./target/release/cryprq
bash scripts/e2e-direct-docker.sh
bash scripts/e2e-direct-relay-docker.sh
bash scripts/e2e-authenticated-file-transfer.sh ./target/release/cryprq
bash scripts/e2e-authenticated-vpn.sh ./target/release/cryprq
bash scripts/e2e-tun-docker.sh
(cd web && npm ci && npm run build && npm run test:adapter)
```

The direct-file Python test runs two local processes over both QUIC and TCP. It
checks nonempty/empty delivery, application-authentication events, matching
hash receipts, wrong identities, existing destination files and symlinks,
directory/symlink/FIFO sources and both test-mode controls. It prints
`route=local; WAN TEST NOT VERIFIED` and keeps failure logs. Unit tests cover
additional malformed-record, replay, key-confirmation, disconnect, cancellation
and receiver-safety cases. Browser Playwright,
legacy Docker and TUN regressions also remain subject to their existing hosted
jobs; none is replaced by the new test.

| Evidence for this candidate revision | Where it is recorded |
| --- | --- |
| Direct transport unit/security tests, CLI guards, two-process QUIC/TCP E2E | Pull request description, for the exact commit; local only |
| Formatting, Clippy, full workspace tests, dependency/security scanners | Pull request description and the hosted CI run for that commit |
| Existing file, portal, VPN, legacy Docker and TUN regression gates | The eight required hosted checks on the signed candidate commit |
| Separate-network internet transfer | **NOT VERIFIED; no approved WAN infrastructure** |

Results are recorded against the exact candidate commit, not in this file. Passing
checks on an earlier baseline do not validate new code, skipped jobs do not
count as passing, and this phase does not authorize a merge or Phase 2 work.
