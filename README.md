[![CI](https://github.com/codethor0/cryprq/actions/workflows/ci.yml/badge.svg)](https://github.com/codethor0/cryprq/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# CrypRQ: Post-Quantum, Zero-Trust VPN

<img width="512" height="512" alt="CrypRQ Logo" src="logo.png" />

> Post-quantum, zero-trust VPN with five-minute ephemeral key rotation.

## Overview

CrypRQ is a post-quantum VPN built for the web. It provides secure peer-to-peer connections using hybrid ML-KEM (Kyber768-compatible) + X25519 encryption over libp2p QUIC, with automatic five-minute key rotation.

**Web-First Architecture**: Modern React + TypeScript frontend with Node.js backend, deployable via Docker Compose.

> ⚠️ **Important:** The current v1.0.1 web-only preview uses **test-mode** configuration (static keys, no handshake, no peer authentication). This configuration is for **testing/lab use only** and MUST NOT be used in production. See [`docs/SECURITY_NOTES.md`](docs/SECURITY_NOTES.md) for details.

## Features

- **Hybrid ML-KEM (Kyber768-compatible) + X25519** handshake over libp2p QUIC
- **Five-minute key rotation** with secure zeroization of prior keys
- **Userspace WireGuard prototype** using ChaCha20-Poly1305 and BLAKE3 KDF
- **Secure file transfer** over encrypted tunnel with integrity verification
- **Web UI**: Modern React + TypeScript interface for connection management
- **Docker-ready**: Single-command deployment with `docker compose`
- **Real-time observability**: Structured logs for handshake, rotation, and connection events
- **Core Rust crates**: `crypto` (`no_std` ML-KEM), `p2p` (libp2p swarm), `node` (tunnel), `cli`
- **Supply-chain hardening**: Vendored dependencies, `cargo audit`, `cargo deny`, `CodeQL`

## Quickstart

### Web UI + File Transfer (Recommended)

**Option 1: Docker Compose (Easiest)**

```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq

# Build and start web stack
docker compose -f docker-compose.web.yml up --build

# Open http://localhost:8787 in your browser
```

The web UI provides:
- Real-time connection management (listener/dialer modes)
- Live log streaming with structured event parsing
- **Secure file transfer** via web interface
- Connection status and key rotation monitoring

**Option 2: Local Development**

```bash
# Terminal 1: Build Rust backend
cargo build --release -p cryprq

# Terminal 2: Start web server
cd web
npm install
node server/server.mjs

# Terminal 3: Start frontend dev server (optional, for development)
cd web
npm run dev

# Open http://localhost:8787 in your browser (web server serves built frontend)
# Or http://localhost:5173 if using dev server
```

**File Transfer via Web UI:**
1. Establish connection (start listener and dialer via UI)
2. Click "Send File Securely" button
3. Select file to upload
4. Monitor transfer progress in real-time logs
5. Verify file received successfully

### CLI (For Testing/Development)

```bash
# Build
cargo build --release -p cryprq

# Listener (Terminal 1)
RUST_LOG=info ./target/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

# Dialer (Terminal 2)
RUST_LOG=info ./target/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1
```

**Expected Logs:**

Listener should show:
```
event=listener_starting peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 transport=QUIC/libp2p
Local peer id: 12D3KooW...
event=listener_ready peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 status=accepting_connections
event=rotation_task_started interval_secs=300
event=handshake_complete peer_id=... direction=inbound encryption=ML-KEM+X25519 status=ready
event=connection_established peer_id=... transport=QUIC/libp2p encryption_active=true
```

Dialer should show:
```
event=dialer_starting peer_id=12D3KooW... target_addr=/ip4/127.0.0.1/udp/9999/quic-v1 transport=QUIC/libp2p
event=handshake_complete peer_id=... direction=outbound encryption=ML-KEM+X25519 status=ready
event=connection_established peer_id=... transport=QUIC/libp2p encryption_active=true
```

**Key Rotation:** Every 5 minutes (or `CRYPRQ_ROTATE_SECS`), you'll see:
```
event=key_rotation status=success epoch=<N> duration_ms=<MS> interval_secs=300
```

### Encrypted File Transfer (CLI)

CrypRQ supports secure file transfer over the encrypted tunnel with end-to-end encryption and integrity verification:

**Terminal 1 – Receiver:**
```bash
./target/release/cryprq receive-file \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1 \
  --output-dir ./received
```

**Terminal 2 – Sender:**
```bash
# Replace <PEER_ID> with the peer ID shown by the receiver
./target/release/cryprq send-file \
  --peer /ip4/127.0.0.1/udp/9999/quic-v1/p2p/<PEER_ID> \
  --file sample.txt
```

**Features:**
- **End-to-end encryption**: Files are transferred over the ML-KEM + X25519 encrypted tunnel
- **Chunked transfer**: Large files are automatically split into chunks for reliable transfer
- **SHA-256 verification**: Automatic hash verification ensures data integrity
- **Real-time progress**: Logs show transfer progress and completion status

**Expected Logs:**

Receiver:
```
INFO  cryprq Receiving files on: /ip4/0.0.0.0/udp/9999/quic-v1, output directory: "./received"
Local peer id: 12D3KooW...
INFO  cryprq Receiving file: sample.txt (1024 bytes) from peer 12D3KooW...
INFO  cryprq File received successfully: sample.txt (1024 bytes) from peer 12D3KooW...
```

Sender:
```
INFO  cryprq Connected to peer: 12D3KooW...
INFO  p2p Sent file metadata to 12D3KooW...: sample.txt (1024 bytes)
INFO  cryprq All responses received (3/3) - exiting
```

**Verification:**
```bash
# Verify file integrity
shasum -a 256 sample.txt
shasum -a 256 ./received/sample.txt
# Hashes should match
```

## Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `--listen <multiaddr>` | Listener mode multiaddr. | None |
| `--peer <multiaddr>` | Dialer mode multiaddr (optionally `/p2p/<peer-id>`). | None |
| `--vpn` | Enable VPN mode (TUN interface for system-wide routing). | Disabled |
| `--tun-name <name>` | TUN interface name (VPN mode). | `cryprq0` |
| `--tun-address <ip>` | TUN interface IP address (VPN mode). | `10.0.0.1` |
| `send-file --peer <addr> --file <path>` | Send file over encrypted tunnel. | None |
| `receive-file --listen <addr> --output-dir <dir>` | Receive files over encrypted tunnel. | None |
| `--allow-peer <peer-id>` | Allowlist specific peer IDs (repeatable). **Enforces explicit peer allowlist.** | Allow all |
| `--metrics-addr <addr>` | Bind Prometheus metrics/health server. | `127.0.0.1:9464` |
| `--rotate-secs <seconds>` | Override rotation interval in seconds. | `300` (5 minutes) |
| `RUST_LOG` | Log level (`error`…`trace`). | `info` |
| `CRYPRQ_ROTATE_SECS` | Rotation interval in seconds. **Controls key rotation cadence.** | `300` |
| `CRYPRQ_MAX_INBOUND` | Max pending/established inbound handshakes. | `64` |
| `CRYPRQ_BACKOFF_BASE_MS` | Initial inbound backoff (ms) after failures. | `500` |
| `CRYPRQ_BACKOFF_MAX_MS` | Max inbound backoff (ms). | `30000` |

**Peer flow**: Listener logs a peer ID, dialer connects using the multiaddr, libp2p ping events confirm liveness.

## Security Model

**Assets**: Hybrid handshake material and (future) tunnel keys. All peers authenticate via libp2p identity keys.

**Post-quantum intent**: ML-KEM mitigates store-now-decrypt-later risk. Rotation limits exposure window.

**Hardened deployments**: Disable mDNS discovery. Current limitations and dependency review are documented.

**Limitations**:
- Data-plane encryption still in development.
- mDNS discovery disabled in hardened deployments.
- No automated peer revocation or ACL enforcement yet.
- Dependency `pqcrypto-mlkem` under active review.

**Responsible disclosure**: codethor@gmail.com

**Supply-chain checks**: `cargo audit`, `cargo deny`, `CodeQL`.

## Development

### Prerequisites

- Rust 1.83.0+
- Node.js 18+
- Docker and Docker Compose (for Docker deployment)

### Build

```bash
# Build Rust backend
cargo build --release -p cryprq

# Build web frontend
cd web
npm install
npm run build
```

### Testing

```bash
# Run unit tests
cargo test --lib --all --no-fail-fast

# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Security audits
cargo audit --deny warnings
cargo deny check
```

### Code Quality

- **Format**: `cargo fmt --all`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Tests**: `cargo test --lib --all --no-fail-fast`
- **Security**: `cargo audit --deny warnings && cargo deny check`

## Protocol Specification

The complete CrypRQ v1.0 protocol specification is available in [`cryp-rq-protocol-v1.md`](cryp-rq-protocol-v1.md). This document provides:

- Comprehensive protocol specification in RFC-style format
- Cryptographic handshake details (ML-KEM + X25519 hybrid)
- Key schedule and rotation mechanisms
- Record layer and wire format specifications
- Message types and semantics
- Security considerations and threat model
- Implementation guidelines and test vector structure

The specification is implementation-independent and serves as the canonical reference for building compatible CrypRQ implementations.

## Documentation

### Validation & Testing

- **[`docs/VALIDATION_RUN.md`](docs/VALIDATION_RUN.md)** — CLI and tunnel validation results
- **[`docs/WEB_VALIDATION_RUN.md`](docs/WEB_VALIDATION_RUN.md)** — Web-only stack validation matrix
- **[`docs/TEST_MATRIX.md`](docs/TEST_MATRIX.md)** — Complete test matrix (CLI + web)

### Security & Release Notes

- **[`docs/SECURITY_NOTES.md`](docs/SECURITY_NOTES.md)** — Security posture, limitations, and test-mode warnings
- **[`docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`](docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md)** — Web-only stack release summary

### Web Stack Guides

- **[`docs/DOCKER_WEB_GUIDE.md`](docs/DOCKER_WEB_GUIDE.md)** — Docker deployment guide for web-only stack
- **[`docs/WEB_UI_GUIDE.md`](docs/WEB_UI_GUIDE.md)** — Web UI usage and API guide
- **[`docs/WEB_STACK_QUICK_START.md`](docs/WEB_STACK_QUICK_START.md)** — Quick start for new contributors (web stack focus)

### Publications & Citation

- **[`docs/WHITEPAPER.md`](docs/WHITEPAPER.md)** — Technical whitepaper (v1.0.1-web-preview)
- **[`docs/BLOG_OVERVIEW.md`](docs/BLOG_OVERVIEW.md)** — Blog-style overview for public sharing
- **[`CITATION.cff`](CITATION.cff)** — Citation metadata for academic/project citation

### Developer Resources

- **[`docs/MASTER_VALIDATION_PROMPT.md`](docs/MASTER_VALIDATION_PROMPT.md)** — Full-stack validation assistant prompt
- **[`docs/MASTER_WEB_RELEASE_PROMPT.md`](docs/MASTER_WEB_RELEASE_PROMPT.md)** — Web-only release alignment prompt
- **[`docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`](docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md)** — Next-phase handshake & identity implementation guide

## Development

### Pre-Flight Checks (Before Pushing)

Before pushing code, run these commands locally to match CI:

```bash
# Format code
cargo fmt --all

# Run clippy with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all
```

**CI Pipeline:** All pushes to `main` and `feature/**` branches are automatically validated via GitHub Actions:

**Quick Check Job** (runs in parallel for faster feedback):
- ✅ `cargo fmt --all -- --check`
- ✅ `cargo clippy --all-targets --all-features -- -D warnings`

**Full Build Job**:
- ✅ `cargo build --release --workspace`
- ✅ `cargo test --workspace`
- ✅ `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ `cargo fmt --all -- --check`

**Nightly Job** (runs daily at 2 AM UTC):
- ✅ Documentation generation (`cargo doc`)
- ✅ Build and test with all features enabled
- ✅ Documentation warning checks

Branch protection ensures these checks must pass before merging to `main`.

## Architecture

### Core Crates

- **`crypto`**: Post-quantum cryptography (`no_std` ML-KEM implementation)
- **`p2p`**: libp2p swarm management and QUIC transport
- **`node`**: Tunnel management, encryption, and key rotation
- **`cli`**: Command-line interface
- **`core`**: Core utilities

### Web Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Node.js Express server (spawns Rust binary)
- **Deployment**: Docker Compose with multi-stage builds

## License

CrypRQ is licensed under the [MIT License](LICENSE).

## Support

If this project helps you, consider supporting ongoing maintenance:

- **One-time tip:** https://buy.stripe.com/00w6oA7kM4wc4co5RB3Nm01  
- **Monthly support:** https://buy.stripe.com/7sY3cobB2bYEdMYa7R3Nm00

**What you fund:** maintenance, docs, roadmap experiments, and new features.
