# CrypRQ v1.0.1 — Web-Only Preview (Test Mode)

> ⚠️ **Security Notice**  
> This is a **web-only, test-mode preview** of CrypRQ v1.0.1.  
> It is intended for **development and lab testing only** and MUST NOT be used in production.

---

## Overview

CrypRQ is a post-quantum-hybrid secure tunnel and file-transfer protocol with:

- A custom record layer
- Epoch-based key rotation
- Chunked file transfer with SHA-256 verification
- VPN/TUN support at the protocol level

This release packages a **web-only preview** of the v1.0.1 stack:

- CLI stack validated end-to-end for file transfer
- Web backend wired to the v1.0.1 record layer
- Web UI for file transfer and log inspection
- Docker-based web deployment flow

The goal of this release is to make it easy to **stand up the web stack, run tests, and iterate**, *not* to provide a production-hardened secure tunnel.

---

## What's Included

### Protocol / Engine

- CrypRQ v1.0.1 **record layer**:
  - 20-byte header (version, type, flags, epoch, stream ID, sequence number, ciphertext length)
  - TLS 1.3–style nonce construction (static IV XOR sequence number)
  - HKDF-based key schedule with epoch-scoped keys

- Message types wired for file transfer:
  - `FILE_META`, `FILE_CHUNK`, `FILE_ACK`
  - `CONTROL` hooks in place for future key updates / control flows

- Epoch management:
  - `u8` epoch (0–255, wraps modulo 256)
  - Key rotation logic implemented (test-mode wiring)

### CLI

- `cryprq send-file` / `cryprq receive-file`:
  - File transfer over CrypRQ records
  - Chunked transfer with final SHA-256 verification
  - Validated in `VALIDATION_RUN.md` (minimal sanity test: PASS)

### Web Stack

- **Backend**:
  - Web stack integrated with CrypRQ record layer
  - Endpoints for initiating file transfers and streaming logs
  - Dockerized web deployment (`docker-compose.web.yml`)

- **Frontend**:
  - React/TypeScript Web UI
  - File selection + peer/endpoint configuration
  - Real-time status / log streaming panel

### Documentation

**Core docs for this release:**

- `docs/WEB_STACK_QUICK_START.md` — **Start here** (onboarding + "run this first")
- `docs/DOCKER_WEB_GUIDE.md` — Docker-based deployment and operations
- `docs/WEB_UI_GUIDE.md` — Web UI behavior, flows, and API expectations
- `docs/WEB_VALIDATION_RUN.md` — Web validation matrix and results tracker
- `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md` — Release-specific notes
- `docs/SECURITY_NOTES.md` — Security posture and limitations (MUST read)

**Master / automation prompts:**

- `docs/MASTER_VALIDATION_PROMPT.md` — Full-stack validation guidance
- `docs/MASTER_WEB_ALIGNMENT_PROMPT.md` — Web stack alignment / QA
- `docs/MASTER_WEB_RELEASE_PROMPT.md` — Web-only release engineering flow
- `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` — Future handshake & identity work

---

## Limitations and Security Posture (Test Mode)

This release is explicitly **NOT** production-ready. Key limitations include:

- **No real handshake yet**  
  - No CRYPRQ-defined `CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH` path implemented.
  - No transcript-bound authentication.

- **No peer identity / authentication**  
  - No Ed25519/X.509/libp2p ID integration.
  - Peers are not cryptographically authenticated.

- **Static / test-mode keys**  
  - Keys are currently fixed / test-mode derived.
  - A direction "hack" is used in test mode so both sides can decrypt with the same key material.

- **Not hardened for hostile environments**  
  - No systematic HTTP/Web API hardening.
  - No rate-limiting, authz, or multi-tenant isolation.
  - Intended to run in controlled lab/dev environments only.

See `docs/SECURITY_NOTES.md` for the full security posture and a clear list of **MUST-FIX** items before any production deployment.

---

## Quick Start

### 1. Build

```bash
cargo build --release -p cryprq
```

### 2. Web Stack (Docker)

From the repo root:

```bash
docker compose -f docker-compose.web.yml up --build
```

Check `docs/DOCKER_WEB_GUIDE.md` for ports, environment variables, and troubleshooting.

### 3. Web UI

- Open the Web UI in your browser (see port in `DOCKER_WEB_GUIDE.md` / `docker-compose.web.yml`).
- Follow `docs/WEB_UI_GUIDE.md` for:
  - Selecting a file
  - Configuring the peer/endpoint
  - Starting a transfer and watching logs

### 4. Validation

**Recommended:**

- Run the minimal web sanity test from `docs/WEB_VALIDATION_RUN.md`:
  - Send a small file via the Web UI
  - Verify it is received correctly and the SHA-256 hash matches
- Confirm CLI validation in `docs/VALIDATION_RUN.md`.

---

## Validation & Status

- **CLI path:** ✅ Minimal sanity test PASS
- **Web path:** 🔄 Validation matrix defined in `WEB_VALIDATION_RUN.md`

Use `docs/MASTER_WEB_RELEASE_PROMPT.md` to guide a systematic validation run.  
Update `WEB_VALIDATION_RUN.md` with PASS/WARN/BLOCK for each test ID.

---

## This Release Is Tagged As

- **Web-only preview / test-mode build**
- **Suitable for:** protocol development, record-layer testing, and web stack experimentation.
- **Not suitable for:** production traffic, real user data, or security-sensitive deployments.

---

## Roadmap / Next Steps

Planned future work (tracked via docs and issue templates):

- Implement the full CrypRQ handshake (`CRYPRQ_CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH`).
- Add peer identity and authentication (Ed25519/X.509/libp2p ID/PSK).
- Replace static test keys with dynamic, negotiated keys.
- Harden the web stack (auth, rate limiting, logging hygiene, secure deployment guidance).
- Expand validation:
  - Multi-file / concurrent transfers
  - VPN/TUN integration tests
  - Interop and fuzzing

---

## One-Line Verdict

CrypRQ v1.0.1 Web-Only Preview provides a fully wired, test-mode implementation of the record layer + file transfer path with a Dockerized web UI and comprehensive validation docs, suitable for development and protocol experimentation but not for production use.

