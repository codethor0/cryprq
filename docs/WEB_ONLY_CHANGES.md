# Web-Only Refactor Summary

## Overview
This branch (`web-only-refactor`) focuses CrypRQ exclusively on the web experience, removing all non-web platforms and simplifying the repository structure.

## What Was Removed

### Platform-Specific Directories
- `android/` - Android platform code
- `apple/` - Apple platform code  
- `macos/` - macOS platform code
- `windows/` - Windows platform code
- `mobile/` - Mobile platform code
- `gui/` - Desktop GUI code
- `packaging/` - OS-specific packaging

### Build Systems & Configs
- `flake.nix`, `flake.lock`, `shell.nix` - Nix configuration
- `cbindgen.toml` - C bindings config
- `Makefile` - Make build system

### GitHub Workflows (Non-Web)
- `.github/workflows/gui-ci.yml` - GUI builds
- `.github/workflows/mobile-*.yml` - Mobile builds
- `.github/workflows/mobile-release.yml` - Mobile releases
- `.github/workflows/release.yml` - General releases
- `.github/workflows/release-verify.yml` - Release verification
- `.github/workflows/icons.yml` - Icon enforcement
- `.github/workflows/icon-enforcement.yml` - Icon enforcement
- `.github/workflows/pr-cheat-sheet.yml` - PR automation
- `.github/workflows/qa-*.yml` - QA workflows
- `.github/workflows/nightly-hard-fail.yml` - Nightly builds
- `.github/workflows/extended-testing.yml` - Extended tests

### Legacy/Dead Code
- Old report files (`*_REPORT.md`, `*_TEST*.md`, etc.)
- Old log files (`*.log`)
- Old QA directories (`qa-*`)
- Old release artifacts (`release-*`, `artifacts/`)
- Certificate files (`.certSigningRequest`, `.cer`)

## What Was Kept

### Core Web Stack
- `web/` - Frontend (Vite + React + TypeScript)
- `web/server/` - Node.js Express backend
- `crypto/` - Core crypto library (Rust)
- `p2p/` - P2P networking (Rust)
- `node/` - VPN node logic (Rust)
- `core/` - Core utilities (Rust)
- `cli/` - CLI binary (spawned by web backend)
- `third_party/` - Vendored dependencies

### Docker & Deployment
- `Dockerfile` - Rust backend build
- `Dockerfile.web` - **NEW** - Multi-stage web stack build
- `docker-compose.yml` - Simplified compose
- `docker-compose.vpn.yml` - VPN compose (kept)
- `docker-compose.web.yml` - **NEW** - Web-focused compose

### Documentation
- `docs/OPERATOR_LOGS.md` - Log reference
- `docs/DOCKER_VPN_LOGS.md` - Docker logs guide
- `docs/WEB_VERSION_STATUS.md` - Web status
- `README.md` - Updated for web-only

### CI/CD (Simplified)
- `.github/workflows/ci.yml` - Simplified (removed iOS/Android steps)
- `.github/workflows/security-audit.yml` - Security audits
- `.github/workflows/security-checks.yml` - Security checks
- `.github/workflows/codeql.yml` - CodeQL analysis
- `.github/workflows/web-preview.yml` - Web preview
- `.github/workflows/docker-test.yml` - Docker tests

### Testing & Development
- `scripts/test-cli-logs.sh` - Log verification script
- `tests/` - Integration tests
- `fuzz/` - Fuzzing (de-emphasized)
- `benches/` - Benchmarks (de-emphasized)

## New Files

- `Dockerfile.web` - Multi-stage Dockerfile for web stack
- `docker-compose.web.yml` - Web-focused Docker Compose
- `WEB_ONLY_CHANGES.md` - This file

## Changes Made

### Cargo.toml
- Kept all workspace members (crypto, p2p, node, cli, core, fuzz, benches)
- Added comment noting fuzz/benches are de-emphasized

### CI Workflow
- Removed iOS/Android icon validation steps
- Kept core Rust build/test steps
- Kept security and code quality checks

### README.md
- Updated to emphasize web-first architecture
- Added Docker Compose quickstart
- Added note about archived platforms

## How to Recover Archived Code

The pre-refactor state is preserved in:
- **Tag**: `pre-web-split-20251113`
- **Branch**: `main` (before this refactor)

To recover:
```bash
git checkout pre-web-split-20251113
# or
git checkout main  # before merge
```

## Next Steps

1. Test web build: `cd web && npm run build`
2. Test Docker: `docker compose -f docker-compose.web.yml up --build`
3. Verify CI passes: Push to GitHub and check workflows
4. Update deployment docs if needed
5. Merge to `main` when ready

## Architecture

```
CrypRQ Web Stack:
├── Frontend (web/)
│   ├── Vite + React + TypeScript
│   └── Real-time UI with EventSource
├── Backend (web/server/)
│   ├── Node.js Express server
│   └── Spawns Rust cryprq binary
└── Core Rust
    ├── crypto/ - ML-KEM + X25519
    ├── p2p/ - libp2p QUIC
    ├── node/ - VPN tunnel logic
    └── cli/ - Binary executable
```

