# Security Notes for CrypRQ

Status: **research prototype, pre-production.** Originally written for the
v1.0.1 web preview; kept current for the authenticated protocol branch.

## Open findings register — November 11, 2026

Four separate items. None is suppressed and none is resolved by a passing
test run; each stays open until the stated condition is met.

**1. `pqcrypto-*` is unmaintained** (RUSTSEC-2026-0161, -0162, -0163).
`pqcrypto-mlkem`, `pqcrypto-traits` and `pqcrypto-internals` provide the
ML-KEM-768 implementation used by the handshake. `cargo deny` and
`cargo audit` both fail on them. There is no patched release. Closing this
means replacing the ML-KEM backend, a cryptographic change that needs its own
review with known-answer vectors and wire compatibility checks. Not started.

**2. `hickory-proto` advisories remain in `Cargo.lock`** (RUSTSEC-2026-0118,
-0119, -0487, -0489, -0490, -0491). The crate is not compiled or linked:
`cargo tree -i hickory-proto` is empty and `cargo deny` does not report it.
`cargo audit` reads the lockfile, where Cargo keeps `libp2p-mdns` as an
unbuilt optional dependency of `libp2p`, and therefore still fails on all
six. They leave the lockfile only with libp2p 0.57 or by depending on the
individual `libp2p-*` crates. Not started.

**3. 17 npm advisories in `web/` development dependencies** (10 high,
7 moderate). `npm audit --omit=dev` reports none: they are in build and test
tooling, not in what is served. No non-breaking update removes them; it needs
a tooling migration. Not started.

**4. mDNS was removed, not fixed.** Local peer discovery no longer exists in
this build: the `mdns` feature of libp2p is not compiled, and
`CRYPRQ_ENABLE_MDNS=1` only logs that discovery is unavailable. This is a
loss of functionality chosen to take the vulnerable DNS parser out of the
binary. Its discovery events were never used, so no other behaviour changed.
It must not be restored on libp2p 0.56.

## Dependency gate checkpoint — November 11, 2026

Two changes since the checkpoint below; neither adds an advisory exception.

- `rustix` 1.1.3 → 1.1.6 (lockfile only) clears RUSTSEC-2026-0486 in both
  scanners.
- mDNS is no longer compiled: the `mdns` feature of libp2p is dropped from
  `p2p`, and with it `libp2p-mdns` and `hickory-proto` leave the build. mDNS
  had been made opt-in first; its discovery events were never consumed, so
  nothing depended on it. `cargo tree -i hickory-proto` is empty and
  `cargo deny check` no longer reports the six `hickory-proto` advisories.
  **`cargo audit` still reports all six**, because it reads `Cargo.lock`,
  and Cargo records `libp2p-mdns` there as an optional dependency of the
  `libp2p` crate whether or not it is built. They disappear from the lockfile
  only with libp2p 0.57 or by depending on the individual `libp2p-*` crates.
  The audit job therefore still fails, visibly, on code that is not linked.

Still open and blocking in both scanners: RUSTSEC-2026-0161, -0162, -0163
(`pqcrypto-*` unmaintained). Clearing them means replacing the ML-KEM backend,
which is a cryptographic change and needs its own review; see below.

## Dependency gate checkpoint — November 10, 2026

The Rust 1.88 recovery candidate retains libp2p 0.56 and updates reqwest to
0.12.28, quinn-proto to 0.11.18, time to 0.3.47, and OpenSSL to 0.10.80.
Seven vulnerability instances from the previous lockfile and three newly
reported OpenSSL findings are removed. This is not a release clearance:
cargo-deny and cargo-audit still fail on the remaining findings below.
Both scanners execute independently in CI; no new advisory exceptions are added.

| Finding | Active dependency path | Required follow-up |
| --- | --- | --- |
| [RUSTSEC-2026-0118](https://rustsec.org/advisories/RUSTSEC-2026-0118.html) | libp2p 0.56 → libp2p-mdns 0.48 → hickory-proto 0.25.2 | DNSSEC features required by this advisory are absent from the normal production graph. The dependency advisory remains visible and blocking. |
| [RUSTSEC-2026-0119](https://rustsec.org/advisories/RUSTSEC-2026-0119.html) | Same mDNS path; the mDNS encoder is compiled | Fixed in Hickory 0.26.1, outside mDNS 0.48's `^0.25.2` requirement. Application attacker reachability has not been established. |
| [RUSTSEC-2026-0161](https://rustsec.org/advisories/RUSTSEC-2026-0161.html), [0162](https://rustsec.org/advisories/RUSTSEC-2026-0162.html), [0163](https://rustsec.org/advisories/RUSTSEC-2026-0163.html) | Both `cryprq-crypto` and `node` depend on pqcrypto-mlkem and pqcrypto-traits; ML-KEM pulls in pqcrypto-internals | These packages are unmaintained, with no patched release. Replace the backend through a coordinated protocol review. |

The published compatible Hickory chain is libp2p 0.57 → libp2p-mdns 0.49 →
Hickory 0.26.1, requiring Rust 1.88. That migration also changes core/swarm
APIs and QUIC/TLS negotiation, including the provider and preferred hybrid
group. It is excluded from this scoped update and must be validated with the
authenticated protocol and VPN owner. Removing mDNS changes discovery behavior;
an independent backport would require separate maintenance and security review.

RustSec recommends RustCrypto `ml-kem` as a pqcrypto replacement. Version 0.3.4
declares Rust 1.87 support, but is not a drop-in dependency: both application
consumers need KEM, serialization, and RNG adaptations (`rand_core` 0.10 versus
the current 0.6). A migration must preserve ML-KEM-768 wire sizes, transcript
and KDF inputs, and persisted secret-key representation; explicitly enable
secret zeroization and verify known-answer vectors, malformed input handling,
and bidirectional interoperability before replacing either consumer. Its own
MSRV alone does not validate the resolved application graph. The backend has
not been integrated or independently audited by this project.

Frontend audits currently report 17 development-tool findings (10 high,
7 moderate), with zero frontend runtime findings and zero server findings.
No compatible changes within the existing ranges remove those tooling findings;
they require a separate tooling migration. Browser smoke success is independent
of these findings and is not evidence of authenticated VPN or file-transfer
success. Historical functional claims below must be revalidated on the final
combined revision before release.

## Implementation status against the v1.0.1 warnings

This section replaces the "MUST FIX before production" list written for the
v1.0.1 web preview. Each of those warnings was checked against the code in
this repository. The project is still **pre-production**: see "What is not
established" below and the open findings register at the top.

Evidence levels used here:

- **Implemented**: the code path exists.
- **Tested locally**: tests that exercise it pass on a developer machine.
- **Tested in hosted CI**: see the workflow runs of this repository. The
  statements below are "tested locally" unless a run shows otherwise.

### The four v1.0.1 warnings

| v1.0.1 warning | Status now | Evidence |
|---|---|---|
| Test-mode key direction hack: both sides decrypt with the initiator's keys (`Tunnel::recv_record`) | **Resolved.** A session derives separate keys per direction from its role (`epoch_keys` in `node/src/lib.rs`); the initiator sends with one pair and the responder with the other. The fixed-key test tunnel takes an explicit role as well | Tested locally: `traffic_keys_are_separated_by_direction`, `every_epoch_and_direction_has_its_own_key_and_iv` (`node/src/tests.rs`) |
| Hardcoded test keys in the CLI (`[0x01; 32]` and so on) | **Test-only and isolated.** The constants remain in `test_mode_tunnel` (`cli/src/main.rs`). It is reached only with `--test-mode` on the command line **and** `CRYPRQ_TEST_MODE=1` (or `true`) in the environment; `node::create_tunnel` refuses fixed credentials otherwise. Without both, file transfer and `--vpn` use the authenticated handshake and never fall back | Tested locally: "--test-mode alone is refused" and "the unauthenticated test-mode VPN is refused unless explicitly enabled" in the two-process scripts |
| Missing real handshake (placeholder X25519 with a TODO) | **Resolved.** Handshake v2 (`node/src/auth_handshake.rs`, specified in `docs/HANDSHAKE_V2.md`): ML-KEM-768 and X25519 shares, a transcript hash over every message, session secrets bound to that transcript, and key confirmation in both directions before any data is accepted | Tested locally: `pinned_peers_agree_on_transcript_bound_keys`, `man_in_the_middle_key_share_substitution_is_detected`, `every_single_bit_flip_in_any_message_is_rejected`, `wrong_key_confirmation_fails_and_yields_no_session`, `replayed_messages_from_an_earlier_session_are_rejected` |
| No peer identity verification (placeholder) | **Resolved.** Each node has a persistent Ed25519 identity; peers are pinned explicitly, an empty trust set refuses to start, and there is no trust on first use. Handshake messages are signed and the signatures are checked against the pinned key | Tested locally: `responder_rejects_unpinned_initiators`, `impersonating_a_pinned_initiator_fails_at_finish`, `responder_with_substituted_identity_key_is_rejected`; "unpinned sender was refused", "sender refused a receiver with another identity" in the scripts |

### Statements from the old "safe for testing" list that changed

- "u8 epoch with modulo 256 wrapping" is no longer how it works. Epochs never
  wrap: after epoch 255 the session expires and must be replaced, so a key and
  nonce pair is never reused. Tested locally:
  `epoch_exhaustion_expires_the_session_instead_of_wrapping`.
- "Sequence counter reset on epoch change" holds in the sense that every epoch
  has its own key and its own sequence space, shared by all message types.
- "SHA-256 hash verification" of received files was not enforced by the
  v1.0.1 receiver. It is now: the receiver publishes a file only after its
  size and SHA-256 match the announcement, atomically and without replacing
  an existing file. Tested locally: 26 receiver tests in
  `node/src/file_transfer.rs`.

### Unauthenticated paths that still exist

These are real execution paths. None of them is a fallback of the
authenticated ones.

| Path | What it is | Exposure |
|---|---|---|
| `cryprq --listen` / `--peer` without `--vpn` | The legacy libp2p/QUIC connection. It runs no CrypRQ handshake and pins nobody. It logs `encryption=libp2p-transport cryprq_handshake=none` | Carries no tunnel traffic and accepts no files: no file-transfer handler is installed and packet forwarding is only wired in test mode. Any node can connect and ping |
| Web preview (`web/`, `docker-compose.web.yml`) | Starts the node on that legacy path. Until this revision the UI stated that all traffic was encrypted with "ML-KEM (Kyber768) + X25519 hybrid"; that was false for this path and is corrected | For evaluation on an isolated machine only. The preview does not provision identities, so `--vpn` and `send-file` started from it stop with "cannot load the node identity" instead of running unauthenticated |
| `--test-mode` with `CRYPRQ_TEST_MODE=1` | Fixed keys, no peer authentication, for interoperability tests | Refused unless both switches are given; logs `INSECURE` |
| Legacy Docker E2E stack (`docker/e2e/`) | Two test-mode containers | CI and local testing only; publishes UDP 9999 on the host that runs it |

### What is not established

- **Hosted CI is not a release gate yet.** The results quoted here are from
  developer machines.
- **No independent review or audit** of the handshake, the record layer or
  the receiver.
- **Authentication is classical.** Ed25519 signatures are not post-quantum;
  only the confidentiality of recorded traffic has hybrid protection.
- **Identity keys are sent in the clear** during the handshake (no identity
  hiding).
- **`--vpn` is a point-to-point link, not a system VPN.** It creates a TUN
  device and assigns its address. It installs no routes, no DNS settings and
  no firewall rules, and has no leak protection. Tested locally between two
  processes and between Linux TUN devices in containers; macOS TUN devices,
  routing, DNS and leak behaviour are untested.
- **No local peer discovery.** mDNS is not compiled;
  `CRYPRQ_ENABLE_MDNS=1` only logs that it is unavailable.
- **File transfer has no acknowledgement or retransmission**, and one
  receiver process serves one session.
- **The open findings register** at the top of this file: `pqcrypto-*`
  unmaintained, `hickory-proto` in `Cargo.lock`, npm development advisories.

### Conditions for calling a release ready

1. Hosted CI executes and passes the jobs in `docs/CI_VPN_GATES.md`, and
   GitHub requires them before a merge.
2. The maintainer has decided each item of the open findings register: fix,
   schedule or accept, in writing.
3. An independent review of the handshake and record layer.
4. Host-level routing, DNS and leak tests, if the release claims to be a
   system VPN.

---

## References

- Protocol Specification: `cryp-rq-protocol-v1.md`
- Test Matrix: `docs/TEST_MATRIX.md`

