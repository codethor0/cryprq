# CrypRQ: A Post-Quantum-Aware Encrypted Record and File Transfer Layer (Web-Only Preview v1.0.1)

**Version:** v1.0.1-web-preview  
**Code commit:** db0903f (implementation baseline)  
**Status:** Web-only, test-mode preview (static keys, no handshake, no peer authentication)

---

## 0. Preview / Test-Mode Disclaimer

This document describes the CrypRQ v1.0.1-web-preview implementation.

The current release is **not production-ready**:

- Uses static, pre-shared symmetric keys for record encryption.
- No interactive handshake is performed in this version.
- No peer authentication is enforced.
- Both ends of the tunnel are effectively configured as "initiators" for test convenience.
- All usage is expected to be local, single-operator, lab/test environments only.

Subsequent releases will introduce a full post-quantum hybrid handshake, identity binding, and a hardened production security profile.

---

## 1. Introduction

Modern encrypted transport protocols such as TLS 1.3 and QUIC provide strong security, but they carry considerable complexity and tight coupling to specific protocol stacks. CrypRQ explores a different axis:

**A record-layer-centric, post-quantum-aware encrypted transport, optimized for file transfer and tunnel-style networking, with a clear separation between:**

- Cryptographic record/traffic layer
- Node / tunnel logic
- Web UI and operator workflows

This whitepaper documents:

- The record layer and key schedule used to protect data in transit.
- The post-quantum-aware cryptographic design that underpins future handshake work.
- The file transfer protocol and buffering model implemented in the node crate.
- The web stack and operational workflows used in the v1.0.1 web-only preview.
- The security model and limitations of this test-mode release.
- The validation, CI, and release process that back the published artifacts.

Our aim is to provide a transparent, technically detailed description that can be cited, reviewed, and critiqued by practitioners and researchers.

---

## 2. System Overview

CrypRQ is implemented as a set of Rust crates and a small web stack:

### cryp-rq-core (core crate)

Defines the record format, constants (e.g., `MSG_TYPE_DATA`), header parsing/serialization, and record-level encryption interface.

### cryp-rq-crypto (crypto crate)

- Implements the key derivation functions (KDFs), use of HKDF, and integration with AEAD (e.g., `chacha20poly1305`).
- Hosts post-quantum ML-KEM (Kyber-768-class) primitives via `pqcrypto-mlkem`.
- Provides property and KAT tests for handshake-related key derivations.

### node crate

- Implements record-layer usage, file transfer, buffer pools, replay protections, and tunnel orchestration.
- Uses libp2p, QUIC, and associated transport components to build data paths.

### p2p crate

Encapsulates P2P behaviors: Kademlia, ping, connection limits, allow/block lists, etc.

### CLI (cli crate, cryprq)

Provides an operator-oriented command line interface for local testing and integration.

### Web stack (cryprq-web container)

A small TypeScript/React/Vite front-end, plus a lightweight backend that forwards file operations into the node/record layer. Exposes a localhost web UI on port 8787 for file transfer tests.

### Packaging & deployment

Dockerized via `docker-compose.web.yml` with two containers:

- `cryprq-web` (web UI + backend)
- `cryprq-vpn` / node container (tunnel & record layer)

In the v1.0.1-web-preview, the system is deliberately constrained:

- Single-operator workflows (local environment).
- Static keys injected from configuration / environment.
- No distributed handshake or identity assertions.

This allows us to validate the record layer, file transfer machinery, and web UX in isolation before layering on a full handshake and identity model.

---

## 3. Cryptographic Design

### 3.1 Goals

The cryptographic layer is designed with the following goals:

**Post-quantum awareness**

Integrate a modern ML-KEM (Kyber-class) KEM for forward-compatible key establishment.

**Record-oriented abstraction**

Separate "record encryption" from transport and routing, similar in spirit to TLS record layer, but adapted for our file-transfer/tunnel usage.

**Nonce safety and replay resistance**

Derive per-record nonces from epoch, sequence number, and a static IV, with tests ensuring:

- Determinism where required (for KATs).
- Uniqueness across the feasible space of sequence numbers.
- Correct overflow behavior when sequence counters wrap.

**Domain separation**

Distinct HKDF labels are used for handshake, traffic, and directional keys (e.g., Initiator→Responder vs Responder→Initiator), avoiding key/nonce re-use across contexts.

### 3.2 Primitives

From the implementation and tests:

**Symmetric AEAD: `chacha20poly1305`**

- Used for per-record authenticated encryption.
- 256-bit key, 96-bit nonce, 128-bit authentication tag.

**Key derivation: HKDF**

HKDF-Extract and HKDF-Expand with context-specific labels:

- `LABEL_RI_KEY`, `LABEL_RI_IV`, etc. for derived traffic keys and IVs.
- Dedicated labels for epoch keys and handshake keys.

**Post-quantum KEM: `pqcrypto-mlkem` (ML-KEM-768-class)**

- Used in tests to validate round-trip encapsulation/decapsulation and deterministic KAT behavior.
- Forms the PQ half of a hybrid handshake in future work.

**Classical primitives:**

- X25519 via `x25519-dalek` for classical ECDH, and standard hash functions as required by HKDF and KEM binding.

**Auxiliary libraries:**

- `constant_time_eq` for timing-safe comparisons.
- `blake3` and related modern hash/cr hash utilities where appropriate.

### 3.3 Record Layer Overview

At the core of CrypRQ is a record abstraction:

**A record header that carries:**

- `epoch`: logical key epoch identifier.
- `stream_id`: multiplexing identifier for logical streams.
- `sequence_number`: monotonically increasing per-stream counter.
- `message_type`: e.g., `MSG_TYPE_DATA` (data), and reserved codes for control.
- `flags`: reserved for future features (e.g., end-of-stream).
- `length`: ciphertext payload length.

**A payload that is AEAD-encrypted under keys derived from:**

`master_secret` → epoch key → directional traffic keys / IVs.

The `cryp-rq-core` crate exposes functions such as:

- `RecordHeader::from_bytes` / `to_bytes`
- `Record::from_bytes` / `to_bytes`

An `encrypt` function that binds:

```rust
encrypt(
    version,
    message_type,
    flags,
    epoch,
    stream_id,
    seq,
    plaintext,
    static_iv
) -> io::Result<Record>
```

(The exact signature may differ, but this is the conceptual binding.)

Clippy and unit tests assert that:

- Record headers serialize/deserialize correctly.
- Records can be encrypted/decrypted round-trip with consistent headers.
- Epoch wrapping and sequence counter behavior is well-defined and tested.

### 3.4 Nonce Construction

The `node::crypto_utils` module contains tests like:

- `test_nonce_construction`
- `test_nonce_deterministic`
- `test_nonce_overflow`
- `test_max_nonce_value_constant`

The nonce strategy is:

1. Start from a static IV (per direction, derived from HKDF).
2. Mix in:
   - `epoch`
   - `sequence_number`
3. Construct a 96-bit AEAD nonce such that:
   - For all valid epoch, seq combinations in a deployment, no nonce repeats under the same key.
   - Overflow behavior is explicit and tested (e.g., when sequence counters reach their maximum).

This is implemented using `generic-array` types from `chacha20poly1305`, with deprecation warnings around `as_slice()` handled in code and tests.

### 3.5 KDF and Key Schedule

The `crypto::kdf` module exposes:

- `derive_handshake_keys(...)`
- `derive_traffic_keys(...)`
- `derive_epoch_keys(...)`

Each function:

- Accepts a base secret (`master_secret` or KEM output), plus context (e.g., role, direction).
- Uses HKDF with distinct labels:
  - Example: `"handshake_keys"`, `"traffic_keys"`, `LABEL_RI_KEY`, `LABEL_RI_IV`, etc.
- Produces:
  - AEAD keys (for both directions).
  - IVs or static IV components.
  - Epoch-specific key material.

Tests in this module ensure:

- Determinism for given inputs (KAT-style checks).
- Correct length and type of output keys and IVs.
- No panics when HKDF expand is used (clippy enforced by banning `unwrap()` and replacing with safe handling or `expect` with meaningful messages).

### 3.6 Post-Quantum Hybrid Handshake (Planned)

The `cryp-rq-crypto` crate includes:

**KAT tests:**

- `test_kyber768_keypair_kat`
- `test_kyber768_encaps_decaps_kat`
- `test_kyber768_roundtrip_correctness`
- `test_kyber768_wrong_key_rejection`

**Property tests:**

- `test_hybrid_handshake_symmetry`
- `test_handshake_idempotence`
- `test_key_sizes_consistent`

These indicate the intended handshake architecture:

**Hybrid construction:**

Combine a classical ECDH component (X25519) with ML-KEM-768 to derive a joint `master_secret`.

**Symmetry and idempotence:**

Both sides derive identical key material when transcripts match. Repeated runs with the same transcript yield the same keys (idempotence), as expected.

**Robustness:**

Wrong key usage leads to rejection as expected.

However, v1.0.1-web-preview does not expose this handshake on the wire. It is confined to tests and internal APIs. The live tunnel still uses static symmetric keys.

---

## 4. Record Format and File Transfer Protocol

### 4.1 Record Header

The record header has a fixed size and is designed for:

- Efficient parsing.
- Clear separation of:
  - `epoch`
  - `stream_id`
  - `sequence_number`
  - `message_type`
  - `flags`
  - `length`

Unit tests:

- `test_header_serialization`
- `test_record_serialization`
- `test_epoch_wrapping`

validate correctness. When decrypting, `RecordHeader::from_bytes` is used, and clippy enforcement ensures that error conditions are handled without unchecked `unwrap()` in tests and library code.

### 4.2 Fragmentation and Padding

The `node/file-transfer` layer treats records as transport units for file chunks:

- Large files are split into fixed-size chunks (`CHUNK_SIZE`).
- Each chunk is encapsulated in a single record (or a small sequence of records if necessary).

Padding helpers in `node::padding`:

- `test_pad_packet`
- `test_unpad_packet`

ensure that padding and de-padding are correct and safe for future traffic-shaping features.

### 4.3 File Transfer State Machines

In `node::file_transfer`:

**OutgoingTransfer structure holds:**

- `metadata`: `FileMetadata` (name, size, hash).
- `file_path`: `PathBuf`.
- `chunks_sent`: `u32`.
- `total_chunks`: `u32` (computed using safe `div_ceil` semantics).

**IncomingTransfer is stored in a map keyed by `stream_id`:**

- Tracks file metadata.
- Accumulates chunks.
- Reconstructs the final file in a destination directory.

**Concurrency primitives:**

Incoming/outgoing maps are protected via `Mutex`/`RwLock`. Clippy disallows bare `unwrap()`, so state is accessed either with:

- Explicit `expect("...")` with actionable messages, or
- Pattern matching and early returns.

This protocol supports:

- Start-transfer control messages containing metadata (filename, size, hash).
- Chunk messages carrying encrypted file data.
- Completion detection when all `total_chunks` are received.

Tests such as:

- `test_large_packet`
- `test_tunnel_send_packet`
- `test_buffer_pool_basic`
- `test_buffer_pool_reuse`
- `test_buffer_pool_clear_on_return`

validate buffering behavior and clean state handling.

### 4.4 Replay and Overflow Protections

Nonce and sequence counters are managed in `seq_counters` and replay window logic:

- `test_nonce_overflow_protection`
- `test_replay_window_sequential`
- `test_replay_window_out_of_order`
- `test_replay_window_old_nonces`
- `test_replay_window_large_gap`

These tests ensure:

- The replay window rejects stale or duplicated nonces/sequence numbers.
- Overflow scenarios are explicitly handled instead of silently wrapping.
- Buffer reuse does not leak data across streams or peers.

---

## 5. Networking and Web Stack

### 5.1 Node and P2P Layer

The `node` and `p2p` crates use:

**libp2p and its components:**

- QUIC transports.
- Noise and TLS wrappers.
- Kademlia DHT (`libp2p-kad`).
- `libp2p-mdns` for local discovery (future).
- `libp2p-ping`, connection limits, and allow/block lists.

**tun** for TUN/TAP integration (VPN-style tunneling).

**reqwest** for HTTP client operations where needed.

In v1.0.1-web-preview:

- The topology is intentionally simple:
  - Local node plus web UI.
  - No public bootstrap network.
- The VPN/TUN features are present but not fully exercised in the web preview.

### 5.2 Web Stack Architecture

The web deployment uses `docker-compose.web.yml` with:

**cryprq-web container:**

- Vite/React front-end.
- HTTP API on port 8787 (inside container, forwarded to host).
- Visual UI for selecting a file and initiating transfers.

**cryprq-vpn or equivalent node container:**

- Runs the node binary.
- Manages encrypted file transfer and the tunnel.

**A typical web smoke test for v1.0.1:**

1. Start web stack:
   ```bash
   docker compose -f docker-compose.web.yml up --build
   ```

2. Open browser:
   ```
   http://localhost:8787
   ```

3. Upload `/tmp/testfile.bin` using the UI.

4. Verify integrity:
   ```bash
   sha256sum /tmp/testfile.bin /tmp/receive/testfile.bin
   # Expected:
   # 6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec
   ```

5. Record the result in `docs/WEB_VALIDATION_RUN.md`.

This ties the web UI behavior to the same hash-verified file transfer that is validated by CLI tests, ensuring end-to-end correctness.

---

## 6. Security Model and Limitations

### 6.1 Intended Security Properties (Design Level)

At the design level (including planned handshake work), CrypRQ aims to provide:

**Confidentiality**

Per-record AEAD encryption with unique nonce/key use.

**Integrity and authenticity of records**

AEAD tags; handshake-derived keys bound to peer identities in future versions.

**Forward secrecy and post-quantum robustness**

Hybrid ECDH + ML-KEM handshake so that compromising classical algorithms alone is insufficient.

**Replay protection**

Sequence counters with replay windows and nonce uniqueness.

**Traffic shaping hooks**

Padding and shaping logic in the node layer to support future traffic obfuscation.

### 6.2 Limitations of v1.0.1-web-preview

The current release does not enforce the full security model:

**Static symmetric keys**

- Keys are provisioned out-of-band.
- Every session may reuse the same keys.

**No live handshake**

- PQ and hybrid KEM logic exists only in tests and KDF design.
- No negotiated parameters on the wire.

**No peer authentication**

- There is no binding between keys and an identity (certificates, signatures, etc.).
- The system assumes a test/lab environment where both endpoints are under a single operator's control.

**"Both sides initiator" test hack**

- For simplicity, handshake roles are not distinguished in live traffic.
- This simplification will be removed once the full handshake is implemented.

**No hardened DoS / resource controls yet**

- While rate limiting tests exist (e.g., `test_rate_limiter_basic`, `test_rate_limiter_burst_then_sustained`), these are not yet treated as a complete DoS hardening story.

### 6.3 Threat Model for v1.0.1-web-preview

Given the limitations, the preview is only appropriate for:

- Localhost / lab environments.
- Single-operator demos and testing of:
  - Record format and encryption.
  - End-to-end file transfer behavior.
  - Web stack UX and operational flow.

It is **not appropriate for**:

- Internet-exposed use.
- Multi-tenant scenarios.
- Protecting high-value or regulated data.

---

## 7. Implementation, Testing, and Validation

### 7.1 Crate-Level Testing

Each Rust crate (`core`, `crypto`, `node`, `p2p`, `cli`) includes:

**Unit tests for low-level primitives:**

- Record serialization.
- Nonce construction.
- KDF behavior.
- Tunnel state machines and buffer pools.

**KAT tests for cryptographic components:**

- ML-KEM keypair/encaps/decaps vectors.
- KDF output determinism.

**Property-based tests with `proptest`:**

- Key size consistency.
- Handshake symmetry.
- Idempotence properties.

All tests are wired into a unified `cargo test` invocation, and CI ensures they pass across the crates.

### 7.2 Clippy and Formatting

The CI pipeline enforces:

- `cargo clippy --all-targets --all-features -- -D warnings` for:
  - `core`
  - `crypto`
  - `node`
  - `p2p`
  - `cli`
- `cargo fmt --all -- --check` for consistent formatting.

**Specific clippy rules that were addressed:**

- `too_many_arguments` for record encryption APIs (suppressed where semantically justified).
- `disallowed-methods` to remove `unwrap()` from library code and tests where inappropriate.
- `expect_used` and `slow_vector_initialization` in `crypto::kdf` refactored to:
  - Use `vec![0; len]` allocations.
  - Replace `expect` with either:
    - Clearly labeled failure messages, or
    - Pattern matches with explicit error propagation.

Warnings in third-party crates are not treated as fatal; the focus is on keeping project code clippy-clean or explicitly justified via `#[allow(...)]` annotations.

### 7.3 Validation Runs

Two key validation documents anchor v1.0.1-web-preview:

**`docs/VALIDATION_RUN.md` (CLI / minimal sanity)**

Describes a minimal file transfer using the CLI. Records hash:

```
6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec
```

as the canonical test vector for `testfile.bin`.

**`docs/WEB_VALIDATION_RUN.md` (web smoke tests)**

Records that WEB-1 (basic web transfer) passed with the same hash. Demonstrates that the web UI faithfully drives the same underlying record and file transfer logic as the CLI.

**Scripts:**

- `scripts/web-smoke-test.sh`
- `scripts/update-web-validation.sh`
- `scripts/preflight-and-tag.sh`
- `scripts/complete-release.sh`

support repeatable validation, tagging, and release creation with minimal operator effort.

---

## 8. Release and Deployment Model

### 8.1 Release v1.0.1-web-preview

The v1.0.1-web-preview release is anchored by:

**Git tag:** `v1.0.1-web-preview` (immutable)

**Spec and docs:**

- Protocol spec (v1.0.1)
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- `SECURITY_NOTES.md`
- `WEB_STACK_QUICK_START.md`
- `RELEASE_EXECUTION_SUMMARY.md` and `FINAL_RELEASE_STEPS.md`

**Operator-level runbooks:**

- `CUT_THE_RELEASE.md`
- `OPERATOR_GUIDE.md`
- `RELEASE_NOW.md`

**Whitepaper and publication docs:**

- `WHITEPAPER.md` (this document)
- `BLOG_OVERVIEW.md` (high-level blog variant)
- `CITATION.cff` (citation metadata)

**GitHub Release:**

- Title: CrypRQ v1.0.1 — Web-Only Preview (Test Mode)
- Body: `GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
- Prominently displays test-mode / preview security warnings.

### 8.2 Docker and Build Strategy

To decouple source control tags from runtime images, the project uses:

**Immutable git tags for releases:**

- `v1.0.1-web-preview` never moves.

**Rolling Docker builds from `main` for:**

- Demo and preview environments.
- Internal testing of new CI and clippy fixes.

For example:

- Build from the version-bound commit (`db0903f`) on `main`:
  ```bash
  git checkout db0903f
  docker build -t cryprq-web:latest -f docker/Dockerfile.web .
  ```

- Optionally tag images for registries:
  ```bash
  docker tag cryprq-web:latest your-registry/cryprq-web:preview
  docker push your-registry/cryprq-web:preview
  ```

Future releases (e.g., with handshake/identity) will get new git tags such as:

- `v1.1.0-web-preview` or
- `v2.0.0` for the first production-grade release.

### 8.3 Next-Phase Branch

Handshake and identity work is staged on:

- **Branch:** `feature/handshake-and-identity`
- **Driver prompt:** `MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`

**Work items include:**

- Implement real CrypRQ handshake (`CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH`).
- Integrate PQ + classical KEM/ECDH into a hybrid key schedule.
- Bind keys to peer identities (via signatures or certificates).
- Remove static test keys and "both sides initiator" hacks.
- Harden `SECURITY_NOTES.md` for production recommendations.

---

## 9. Positioning and Related Work

CrypRQ is not intended to "replace" TLS or QUIC. Instead, it:

- **Treats the record layer as a first-class object**, enabling:
  - Custom tunnels.
  - Specialized file transfer.
  - Integration in contexts where standard TLS/QUIC stacks are heavy or inflexible.

- **Emphasizes post-quantum preparedness:**
  - ML-KEM hybridization is part of the design from the beginning.

- **Embraces Rust safety and tooling:**
  - Clippy, rustfmt, proptest, and KATs are core to how correctness and security properties are enforced.

The project can be seen as:

- A research and prototyping platform for PQ-aware tunneling and record layers.
- A foundation that could be embedded in larger systems once handshake and identity are fully realized.

---

## 10. Conclusion and Future Work

This whitepaper has described the v1.0.1-web-preview of CrypRQ:

- A record-centric encrypted transport layer.
- With a post-quantum-aware cryptographic design.
- Validated by unit tests, property tests, and deterministic KATs.
- Exposed through a web-only preview with a clear test-mode stance.

**Near-term future work:**

**Handshake and Identity**

- Implement the hybrid PQ + classical handshake on wire.
- Bind peers to stable identities and attest them cryptographically.

**Production Security Profile**

- Eliminate static keys entirely.
- Harden DoS defenses and resource limits.
- Add configurable cipher/KEM suites.

**Operational Hardening**

- Integrate richer observability (metrics, tracing).
- Provide production-oriented deployment guides (Kubernetes, etc.).

**Formalization**

- Capture the record protocol and handshake as a formal specification (TLA+, ProVerif, or similar).
- Pursue third-party cryptographic review.

The v1.0.1 preview is the first public snapshot of this architecture. It demonstrates that:

- The record layer, KDF, and file transfer stack are coherent and testable.
- The web UI can drive real encrypted transfers, with deterministic, hash-verified outputs.
- The project is ready for external review, collaboration, and iteration toward a production-grade system.

---

## Implementation References

- [`docs/VALIDATION_RUN.md`](VALIDATION_RUN.md) – CLI validation run and reference hash
- [`docs/WEB_VALIDATION_RUN.md`](WEB_VALIDATION_RUN.md) – WEB-1 validation (hash-verified file transfer)
- [`docs/DOCKER_WEB_GUIDE.md`](DOCKER_WEB_GUIDE.md) – Web deployment and image/tag strategy
- [`docs/SECURITY_NOTES.md`](SECURITY_NOTES.md) – Security posture and limitations
- [`docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`](WEB_ONLY_RELEASE_NOTES_v1.0.1.md) – Web-only release summary
- [`cryp-rq-protocol-v1.md`](../cryp-rq-protocol-v1.md) – Complete protocol specification (v1.0.1)
- [`docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`](MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md) – Roadmap for handshake and identity

---

**License:** MIT  
**Repository:** https://github.com/codethor0/cryprq  
**Contact:** codethor@gmail.com
