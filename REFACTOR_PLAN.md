# Web-Only Refactor Plan

## Goal
Focus CrypRQ exclusively on web experience: remove all non-web platforms, simplify repo structure, make it Docker-deployable.

## Architecture
- **Frontend**: `web/` (Vite + React + TypeScript)
- **Backend**: `web/server/` (Node.js Express that spawns Rust `cryprq` binary)
- **Core Rust**: `crypto/`, `p2p/`, `node/`, `core/`, `cli/` (needed by backend)

## Directory Categorization

### A. KEEP - Core Web Stack
- `web/` - Frontend (Vite + React + TS)
- `web/server/` - Node.js backend
- `crypto/` - Core crypto library (used by backend)
- `p2p/` - P2P networking (used by backend)
- `node/` - VPN node logic (used by backend)
- `core/` - Core utilities (used by backend)
- `cli/` - CLI binary (spawned by web backend)
- `third_party/` - Vendored dependencies
- `Dockerfile` - Docker build
- `docker-compose.yml` - Docker compose (will simplify)
- `docker-compose.vpn.yml` - VPN Docker compose (keep for now, simplify later)
- `docs/OPERATOR_LOGS.md` - Log reference
- `docs/DOCKER_VPN_LOGS.md` - Docker logs guide
- `docs/WEB_VERSION_STATUS.md` - Web status
- `scripts/test-cli-logs.sh` - Test script

### B. KEEP BUT DE-EMPHASIZE - Shared Core
- `tests/` - Integration tests (if they test core logic)
- `fuzz/` - Fuzzing (keep but de-emphasize)
- `benches/` - Benchmarks (keep but de-emphasize)
- `xtask/` - Build tooling (if still needed)

### C. REMOVE - Non-Web Platforms
- `android/` - Android platform
- `apple/` - Apple platform
- `macos/` - macOS platform
- `windows/` - Windows platform
- `mobile/` - Mobile platform
- `gui/` - Desktop GUI (not web)
- `packaging/` - OS-specific packaging
- `release-*/` - Old release artifacts
- `artifacts/` - Build artifacts
- `dist/` - Distribution files (except web/dist)

### D. REMOVE - Legacy/Dead Code
- `be/` - Unknown/legacy
- `for/` - Unknown/legacy
- `your/` - Unknown/legacy
- `you'll/` - Unknown/legacy
- `prompted/` - Unknown/legacy
- `password/` - Unknown/legacy
- `config/` - If not used by web
- `monitoring/` - If not web-related
- `cryprq/` - Duplicate/legacy
- `store/` - If not web-related
- `store-assets/` - If not web-related
- `branding/` - If not web-related
- `supply-chain/` - If not actively used
- `dependency-reports/` - Old reports
- `logs/` - Old log files
- `result/` - Old test results
- `qa-*/` - Old QA directories
- `qa-chaos/` - Old QA
- `qa-abort-*/` - Old QA
- `*.log` files in root - Old logs
- `*.md` files that are old reports (keep only essential docs)

### E. REMOVE - Nix/Other Build Systems
- `flake.nix` - Nix config
- `flake.lock` - Nix lock
- `shell.nix` - Nix shell
- `Makefile` - If not used by web
- `cbindgen.toml` - C bindings (not needed for web)

### F. REMOVE - GitHub Workflows (Non-Web)
- `.github/workflows/gui-ci.yml` - GUI builds
- `.github/workflows/mobile-*.yml` - Mobile builds
- `.github/workflows/mobile-release.yml` - Mobile releases
- `.github/workflows/release.yml` - General releases (if not web)
- `.github/workflows/release-verify.yml` - Release verification
- `.github/workflows/icons.yml` - Icon enforcement
- `.github/workflows/icon-enforcement.yml` - Icon enforcement
- `.github/workflows/pr-cheat-sheet.yml` - PR automation
- `.github/workflows/qa-*.yml` - QA workflows
- `.github/workflows/nightly-hard-fail.yml` - Nightly builds
- `.github/workflows/extended-testing.yml` - Extended tests
- `.github/workflows/fuzz.yml` - Fuzzing (keep if we keep fuzz/)
- `.github/workflows/docs-ci.yml` - Docs CI (keep if we have docs)
- `.github/workflows/docker-test.yml` - Docker tests (keep if we use Docker)
- `.github/workflows/web-preview.yml` - Web preview (KEEP)
- `.github/workflows/ci.yml` - Main CI (KEEP but simplify)
- `.github/workflows/security-audit.yml` - Security (KEEP)
- `.github/workflows/security-checks.yml` - Security (KEEP)
- `.github/workflows/codeql.yml` - Security (KEEP)
- `.github/workflows/local-validate-mirror.yml` - Validation (KEEP)

## Target Structure

```
CrypRQ/
├── web/                    # Frontend (Vite + React + TS)
│   ├── src/
│   ├── server/            # Node.js backend
│   ├── package.json
│   └── vite.config.ts
├── crypto/                # Core crypto (Rust)
├── p2p/                   # P2P networking (Rust)
├── node/                  # VPN node (Rust)
├── core/                  # Core utilities (Rust)
├── cli/                   # CLI binary (Rust, spawned by web backend)
├── third_party/           # Vendored dependencies
├── tests/                 # Integration tests (if any)
├── fuzz/                  # Fuzzing (de-emphasized)
├── benches/               # Benchmarks (de-emphasized)
├── docs/                  # Documentation
│   ├── OPERATOR_LOGS.md
│   ├── DOCKER_VPN_LOGS.md
│   └── WEB_VERSION_STATUS.md
├── scripts/               # Build/test scripts
│   └── test-cli-logs.sh
├── Dockerfile             # Docker build
├── docker-compose.yml     # Docker compose (simplified)
├── docker-compose.vpn.yml # VPN Docker compose (simplified)
├── Cargo.toml             # Rust workspace
├── Cargo.lock
├── rust-toolchain.toml
├── README.md              # Updated for web-only
├── LICENSE
├── LICENSE-APACHE
├── LICENSE-MIT
├── .github/
│   └── workflows/
│       ├── ci.yml         # Simplified CI
│       ├── security-audit.yml
│       ├── security-checks.yml
│       ├── codeql.yml
│       ├── web-preview.yml
│       └── docker-test.yml
└── .gitignore
```

## Next Steps
1. Remove categorized directories
2. Update Cargo.toml workspace
3. Simplify GitHub workflows
4. Update docker-compose files
5. Update README.md
6. Test web build and Docker deploy

