# CrypRQ v1.0.1 — Web-Only Stack (Test Mode Preview)

**Release Type:** Technical Preview / Test-Only  
**Date:** 2025-11-14  
**Scope:** Web-only CrypRQ v1.0.1 stack (frontend + backend + record layer, test mode)

> ⚠️ **IMPORTANT:** This web-only stack is for **local testing, demos, and protocol exploration only.**  
> It is **NOT** a production-ready VPN or secure file transfer solution.

---

## 1. Overview

This release packages the **CrypRQ v1.0.1** protocol implementation into a **web-only stack**:

- Rust backend using the v1.0.1 record layer (20-byte header, epoch, nonce, HKDF).
- Web UI (React + TypeScript) for:
  - Initiating file transfers.
  - Observing real-time status/logs.
- Docker-based deployment for easy local bring-up.

The web stack reuses the same core **record layer + file transfer** implementation that has been validated via the CLI, but runs in a simplified **test mode** using static keys and no handshake.

---

## 2. What's Included in v1.0.1 (Web-Only)

### 2.1 Protocol-Level Features (Shared with CLI)

- **Record Layer**
  - 20-byte header:
    - Version (1 byte)
    - Message Type (1 byte)
    - Flags (1 byte)
    - Epoch (1 byte, `u8` modulo 256)
    - Stream ID (4 bytes)
    - Sequence Number (8 bytes)
    - Ciphertext Length (4 bytes)
  - AEAD with:
    - TLS 1.3–style nonce: `nonce = static_iv XOR seq_number`
    - Header as AAD.

- **Key Schedule & Rotation**
  - HKDF-based derivation with v1.0.1 labels:
    - `salt_hs = "cryp-rq v1.0 hs"`
    - Labeled `ir` / `ri` keys and IVs.
  - Epoch-scoped traffic keys:
    - Each epoch derives fresh directional keys.
    - Epoch is `u8`, wrapping at 256.

- **File Transfer**
  - `FILE_META`, `FILE_CHUNK`, `FILE_ACK` message types.
  - `FileTransferManager` with:
    - Stream ID allocation (VPN on 1, files ≥ 2).
    - Incoming transfer tracking.
    - Final SHA-256 verification.

---

### 2.2 Web Stack Features

- **Web UI**
  - File selection and send controls.
  - Progress / status view (wired to backend events).
  - Intended for local loopback testing (single host).

- **Web Backend**
  - Adapters that route web-originated file transfers through:
    - `Tunnel` → record layer → UDP.
  - Event/log streaming endpoint (SSE/WebSocket/HTTP polling; implementation-dependent).

- **Docker Compose**
  - `docker-compose.web.yml` for:
    - Backend container.
    - Frontend container.
  - Local-only configuration for test/demo.

---

## 3. Validation Status (Web-Only)

**Documents**

- `VALIDATION_RUN.md` — CLI & tunnel validation.
- `WEB_VALIDATION_RUN.md` — Web-only validation (this release).
- `TEST_MATRIX.md` — Matrix covering CLI + web.
- `SECURITY_NOTES.md` — Security posture and limitations.

**Current Status (Intended Use)**

- ✅ Local testing and demos.
- ✅ Protocol exploration and tooling integration.
- ❌ Not suitable for use on hostile or untrusted networks.
- ❌ Not a production VPN or secure file-transfer product.

Refer to `WEB_VALIDATION_RUN.md` for detailed PASS/FAIL status of:

- WEB-1: Minimal web loopback transfer.
- WEB-2: Medium file transfer.
- WEB-3: Concurrent transfers.
- WEB-5: Web log/event streaming.
- WEB-6/7: Protocol alignment + security posture checks.

---

## 4. Security Posture (Web-Only Test Mode)

The web-only stack **intentionally runs in a weakened security configuration** to simplify validation and prototyping.

### 4.1 Current Behavior (Test Mode)

- **Static Test Keys**
  - Shared symmetric keys are hardcoded / config-driven for test only.
  - No real key exchange occurs over the network.

- **No Handshake / No Peer Authentication**
  - The v1.0.1 handshake (`CRYPRQ_CLIENT_HELLO`, `CRYPRQ_SERVER_HELLO`, `CRYPRQ_CLIENT_FINISH`) is **not yet implemented** in this stack.
  - No cryptographic authentication of the remote peer.
  - No certificate / identity checking.

- **Key Direction Hack (Test Mode Only)**
  - For local testing, both peers effectively act as "initiator," and the receiver uses the same directional keys as the sender (`keys_outbound`) for decryption.
  - This is explicitly documented in `SECURITY_NOTES.md` and must be removed for production.

- **Logging**
  - Logs are designed **not** to include:
    - Private keys.
    - Raw traffic keys.
    - Nonces.
    - Plaintext data.
  - Logs may include high-level events (stream IDs, message types, file names, sizes, statuses).

### 4.2 Explicit Non-Goals for This Release

- ❌ No anonymity.
- ❌ No censorship resistance.
- ❌ No protection against active MITM in real-world networks.
- ❌ No guarantee of correct identity binding or multi-peer trust model.

---

## 5. MUST-FIX Items Before Production Use

These items **must be addressed** before any production or high-risk deployment:

1. **Full Handshake Implementation**
   - Implement:
     - `CRYPRQ_CLIENT_HELLO`
     - `CRYPRQ_SERVER_HELLO`
     - `CRYPRQ_CLIENT_FINISH`
   - Use ML-KEM + X25519 hybrid key exchange as specified in v1.0.1.
   - Derive handshake keys and traffic keys from the negotiated secrets.

2. **Peer Identity & Authentication**
   - Choose and implement identity schemes (e.g., Ed25519, X.509, libp2p-style peer IDs, or PSK).
   - Validate identity during handshake.
   - Bind identity to session keys.

3. **Directional Keys (Correct Initiator/Responder Model)**
   - Remove test-mode key-direction hack.
   - Enforce proper use of:
     - `key_ir` / `iv_ir` (initiator → responder).
     - `key_ri` / `iv_ri` (responder → initiator).

4. **Configurable Key Management**
   - Replace static test keys with:
     - Proper key derivation from handshake.
     - Configurable identity material (keys, certs).

5. **Hardening & Defense-in-Depth**
   - Strict error handling and protocol validation.
   - Rate limiting, timeouts, and resource caps.
   - Additional validation for web inputs (file size limits, path handling, etc.).

6. **Security Review & Threat Modeling**
   - Re-run a full threat modeling exercise for the web stack.
   - Perform internal security review and, ideally, external audit.

---

## 6. Quick Start (Web-Only, Test Mode)

> ⚠️ This is for **local testing only**, with the assumptions and limitations above.

### 6.1 Build

```bash
cargo build --release -p cryprq
```

### 6.2 Run Web Stack

```bash
docker compose -f docker-compose.web.yml up --build
```

Then open the web UI:

```
http://localhost:<frontend_port>
```

(Refer to `DOCKER_WEB_GUIDE.md` for the actual port and configuration.)

### 6.3 Minimal Test

Create a small test file:

```bash
echo "Test file for CrypRQ web v1.0.1" > test-web-minimal.bin
sha256sum test-web-minimal.bin
```

Use the web UI to send the file to the local peer.

Compare the SHA-256 of the received file to ensure a correct transfer.

For detailed steps, see `WEB_VALIDATION_RUN.md` (WEB-1).

---

## 7. Upgrade / Migration Notes

When moving from this test-mode web stack to a more secure or production-ready build:

- Do not reuse the static keys or test-mode config in production.
- Plan a migration path that includes:
  - Handshake-based key establishment.
  - Stable identity scheme (keys/certs, rotation, storage).
  - Updated web UI flows for authenticated peer selection (if needed).
- Re-run:
  - `VALIDATION_RUN.md`
  - `WEB_VALIDATION_RUN.md`
  - Any additional penetration testing.

---

## 8. References

- `PROTOCOL_SPEC_v1.0.1.md` (or equivalent) — Core CrypRQ protocol specification.
- `PROTOCOL_ALIGNMENT_REPORT.md` — Code vs spec alignment.
- `TEST_MATRIX.md` — Complete test matrix.
- `VALIDATION_RUN.md` — CLI/tunnel validation.
- `WEB_VALIDATION_RUN.md` — This release's web-only validation.
- `SECURITY_NOTES.md` — Security posture and limitations.
- `MASTER_VALIDATION_PROMPT.md` — AI/dev-assistant helper for validation.
- `MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` — Next-phase handshake & identity design helper.

---

## 9. One-Line Verdict

**CrypRQ v1.0.1 Web-Only Stack** is a protocol-aligned, test-mode implementation suitable for local demos and validation, but explicitly not for production or use on untrusted networks.

