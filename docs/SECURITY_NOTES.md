# Security Notes for CrypRQ

Status: **research prototype, pre-production.** Originally written for the
v1.0.1 web preview; kept current for the authenticated protocol branch.

## Dependency findings: status

`cargo deny check`, `cargo audit` and `npm audit` (with development
dependencies) report no findings on this revision, and no advisory is ignored
in any configuration. What was open, and how each item was closed:

**1. Unmaintained ML-KEM packages** (RUSTSEC-2026-0161, -0162, -0163).
`pqcrypto-mlkem`, `pqcrypto-traits` and `pqcrypto-internals` are replaced by
the RustCrypto `ml-kem` crate behind `crypto/src/mlkem.rs`, which keeps the
FIPS 203 encodings the handshake puts on the wire. Evidence: the 80
ML-KEM-768 cases of the NIST ACVP vectors pass (key generation,
encapsulation, decapsulation including modified ciphertexts, both key
checks), and a build from before the change and one from after complete
authenticated file transfers and VPN sessions with each other in both
directions. A received encapsulation key is now validated as FIPS 203
requires; the previous implementation accepted any 1184 bytes. The new
backend has not been independently audited by this project.

**2. `hickory-proto` in `Cargo.lock`** (RUSTSEC-2026-0118, -0119, -0487,
-0489, -0490, -0491). The umbrella `libp2p` crate recorded its optional mDNS
and DNS packages in the lockfile although they were never built. `p2p` now
depends on the libp2p components it uses, at the same versions, and builds
its swarm itself (`p2p/src/libp2p.rs`). `hickory-proto`, `libp2p-mdns` and
`libp2p-dns` are no longer in the lockfile.

**3. npm development-dependency advisories** (17, in `web/`). Closed by
updating puppeteer to 25.17 and pinning the lint parser's `minimatch` to a
patched release through an npm override. The override is a pin of a
transitive dependency and should be removed once `typescript-eslint` depends
on a patched `minimatch` itself.

**4. The former exception for RUSTSEC-2025-0134** (`rustls-pemfile`) matched
no package any more and is removed from `cargo-deny.toml` and from the audit
command in CI.

**mDNS remains removed.** Local peer discovery does not exist in this build:
`CRYPRQ_ENABLE_MDNS=1` only logs that it is unavailable. It must not be
restored without a maintained, advisory-free implementation and a decision
that discovery is wanted.

A clean scanner result says that no *known* advisory applies to the locked
dependencies today. It is not a security review of them or of this project.

## Implementation status against the v1.0.1 warnings

This section replaces the "MUST FIX before production" list written for the
v1.0.1 web preview. Each of those warnings was checked against the code in
this repository. The project is still **pre-production**: see "What is not
established" below.

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
- **No audit of the dependencies.** The scanners report no known advisory
  (see the top of this file); that is not an assessment of the code they
  cover, including the ML-KEM implementation.

### Conditions for calling a release ready

1. Hosted CI executes and passes the jobs in `docs/CI_VPN_GATES.md`, and
   GitHub requires them before a merge.
2. `cargo deny`, `cargo audit` and `npm audit` pass without exceptions on the
   release revision.
3. An independent review of the handshake and record layer.
4. Host-level routing, DNS and leak tests, if the release claims to be a
   system VPN.

---

## References

- Protocol Specification: `cryp-rq-protocol-v1.md`
- Test Matrix: `docs/TEST_MATRIX.md`

