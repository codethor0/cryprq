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
