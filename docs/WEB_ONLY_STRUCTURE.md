# CrypRQ Web-Only Repository Structure

## Overview

This document describes the clean, web-focused structure of the CrypRQ repository after the web-only refactor.

## Directory Structure

```
CrypRQ/
├── 📦 Core Rust Crates
│   ├── cli/          # CLI binary (spawned by web backend)
│   ├── core/         # Core utilities
│   ├── crypto/       # ML-KEM + X25519 cryptography
│   ├── node/         # VPN tunnel logic
│   └── p2p/          # libp2p QUIC networking
│
├── 🌐 Web Stack
│   └── web/          # Frontend (Vite + React + TS) + Backend (Node.js)
│       ├── src/      # React frontend source
│       ├── server/   # Node.js Express backend
│       ├── dist/     # Built frontend (gitignored)
│       └── package.json
│
├── 🔧 Supporting
│   ├── tests/        # Integration tests
│   ├── fuzz/         # Fuzzing (de-emphasized)
│   ├── benches/      # Benchmarks (de-emphasized)
│   ├── third_party/  # Vendored dependencies
│   ├── scripts/      # Build/test scripts
│   ├── docs/         # Documentation
│   └── xtask/        # Build tooling
│
├── ⚙️ Configuration
│   ├── Cargo.toml    # Rust workspace
│   ├── Cargo.lock    # Rust dependency lock
│   ├── rust-toolchain.toml  # Rust version
│   ├── package.json  # Node.js workspace
│   ├── cargo-deny.toml  # Dependency checks
│   └── clippy.toml   # Linting config
│
├── 🐳 Docker
│   ├── Dockerfile              # Rust backend build
│   ├── Dockerfile.web          # Web stack (multi-stage)
│   ├── Dockerfile.reproducible # Reproducible builds
│   ├── Dockerfile.test         # Test environment
│   ├── docker-compose.yml      # Basic compose
│   ├── docker-compose.vpn.yml  # VPN compose
│   └── docker-compose.web.yml  # Web-focused compose
│
└── 📄 Documentation
    ├── README.md              # Main README (web-focused)
    ├── README_DOCKER_VPN.md   # Docker VPN guide
    ├── README_RELEASE.md      # Release process
    ├── SECURITY.md            # Security policy
    ├── SUPPORT.md             # Support information
    ├── REPRODUCIBLE.md        # Reproducible builds
    ├── WEB_ONLY_CHANGES.md    # Refactor changelog
    ├── OPERATOR_CHEAT_SHEET.txt  # Operator commands
    └── docs/                  # Additional documentation
        ├── OPERATOR_LOGS.md
        ├── DOCKER_VPN_LOGS.md
        └── WEB_VERSION_STATUS.md
```

## File Count

- **14 directories** (core crates, web, supporting)
- **33 files** (configs, docs, Docker files)
- **Total: 47 items** (excluding gitignored build artifacts)

## Build Artifacts (Gitignored)

- `target/` - Rust build output
- `node_modules/` - Node.js dependencies
- `dist/` - Web build output
- `*.log` - Log files
- `web/received_files/` - File transfer storage

## Key Files

### Core Configuration
- `Cargo.toml` - Rust workspace with 7 members (crypto, p2p, node, cli, core, fuzz, benches)
- `package.json` - Node.js workspace (if used)
- `rust-toolchain.toml` - Rust 1.83.0

### Docker Deployment
- `Dockerfile.web` - Multi-stage build for web stack
- `docker-compose.web.yml` - Web-focused Docker Compose

### Documentation
- `README.md` - Web-first quickstart and architecture
- `docs/OPERATOR_LOGS.md` - Log event reference
- `docs/DOCKER_VPN_LOGS.md` - Docker logging guide

## What Was Removed

- ❌ Platform directories: `android/`, `apple/`, `macos/`, `windows/`, `mobile/`, `gui/`
- ❌ Build systems: Nix (`flake.nix`, `shell.nix`), `Makefile`
- ❌ 14 GitHub workflows for non-web platforms
- ❌ Legacy reports, logs, QA directories, release artifacts
- ❌ Total: 420+ files removed

## What Was Kept

- ✅ Core Rust crates (needed by backend)
- ✅ Web stack (`web/` frontend + backend)
- ✅ Docker support
- ✅ Essential documentation
- ✅ Testing infrastructure (tests, fuzz, benches)

## Architecture

```
┌─────────────────────────────────────────┐
│         Web Frontend (React)           │
│         http://localhost:5173          │
└──────────────┬────────────────────────┘
               │ HTTP/EventSource
┌──────────────▼────────────────────────┐
│    Node.js Backend (Express)          │
│    web/server/server.mjs              │
│    Port: 8787                          │
└──────────────┬────────────────────────┘
               │ Spawns
┌──────────────▼────────────────────────┐
│    Rust Binary (cryprq)                │
│    ./target/release/cryprq             │
│    Uses: crypto, p2p, node, cli        │
└────────────────────────────────────────┘
```

## Quick Start

```bash
# Build Rust backend
cargo build --release -p cryprq

# Start web server
cd web && npm install && node server/server.mjs

# Start frontend dev server (another terminal)
cd web && npm run dev

# Or use Docker
docker compose -f docker-compose.web.yml up --build
```

## Recovery

Pre-refactor state preserved in tag: `pre-web-split-20251113`

```bash
git checkout pre-web-split-20251113
```

