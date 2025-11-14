[![CI](https://github.com/codethor0/cryprq/actions/workflows/ci.yml/badge.svg)](https://github.com/codethor0/cryprq/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# CrypRQ: Post-Quantum, Zero-Trust VPN

<img width="512" height="512" alt="CrypRQ Logo" src="logo.png" />

> Post-quantum, zero-trust VPN with five-minute ephemeral key rotation.

## Overview

CrypRQ is a post-quantum VPN built for the web. It provides secure peer-to-peer connections using hybrid ML-KEM (Kyber768-compatible) + X25519 encryption over libp2p QUIC, with automatic five-minute key rotation.

**Web-First Architecture**: Modern React + TypeScript frontend with Node.js backend, deployable via Docker Compose.

## Features

- **Hybrid ML-KEM (Kyber768-compatible) + X25519** handshake over libp2p QUIC
- **Five-minute key rotation** with secure zeroization of prior keys
- **Userspace WireGuard prototype** using ChaCha20-Poly1305 and BLAKE3 KDF
- **Docker-ready**: Single-command deployment with `docker compose`
- **Real-time observability**: Structured logs for handshake, rotation, and connection events
- **Core Rust crates**: `crypto` (`no_std` ML-KEM), `p2p` (libp2p swarm), `node` (tunnel), `cli`
- **Supply-chain hardening**: Vendored dependencies, `cargo audit`, `cargo deny`, `CodeQL`

## Quickstart

### Web UI (Recommended)

**Option 1: Docker Compose (Easiest)**

```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq

# Build and start web stack
docker compose -f docker-compose.web.yml up --build

# Open http://localhost:5173 in your browser
```

**Option 2: Local Development**

```bash
# Terminal 1: Build Rust backend
cargo build --release -p cryprq

# Terminal 2: Start web server
cd web
npm install
node server/server.mjs

# Terminal 3: Start frontend dev server
cd web
npm run dev

# Open http://localhost:5173 in your browser
```

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

## Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `--listen <multiaddr>` | Listener mode multiaddr. | None |
| `--peer <multiaddr>` | Dialer mode multiaddr (optionally `/p2p/<peer-id>`). | None |
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
