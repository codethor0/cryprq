# Test Plan for CrypRQ Repository

## Repository Overview
- **Language**: Rust (workspace with multiple crates)
- **Toolchain**: Rust 1.83.0 (stable)
- **Project Type**: Post-quantum VPN with cryptographic libraries
- **Workspace Members**: crypto, p2p, node, cli, core, fuzz, benches

## Detected Languages and Frameworks
- **Primary**: Rust (2021 edition)
- **Secondary**: 
  - Shell scripts (bash)
  - TypeScript/JavaScript (web/gui/mobile)
  - Markdown documentation
  - Docker/containerization

## Test Commands

### Core Rust Tests (Primary)
1. **Formatting Check**
   ```bash
   cargo fmt --all -- --check
   ```
   - Enforces consistent code style
   - Must pass (non-negotiable)

2. **Linting**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
   - Catches common mistakes and enforces best practices
   - Must pass (non-negotiable)

3. **Unit Tests**
   ```bash
   cargo test --lib --all --no-fail-fast
   ```
   - Runs all library unit tests across workspace
   - Expected: All tests pass

4. **KAT Tests (Known Answer Tests)**
   ```bash
   cargo test --package cryprq-crypto kat_tests
   ```
   - Cryptographic validation tests
   - Must pass

5. **Property Tests**
   ```bash
   cargo test --package cryprq-crypto property_tests
   ```
   - Property-based testing for cryptographic operations
   - Must pass

6. **Build (Release)**
   ```bash
   cargo build --release -p cryprq
   ```
   - Ensures release build succeeds
   - Must pass

### Security Checks
1. **Cargo Audit**
   ```bash
   cargo audit
   ```
   - Checks for known vulnerabilities
   - Must pass (non-negotiable)

2. **Cargo Deny**
   ```bash
   cargo deny check
   ```
   - License compliance and dependency checks
   - Should pass (may have warnings)

### Integration Tests
1. **Docker VPN Test**
   ```bash
   bash scripts/docker_vpn_test.sh
   ```
   - End-to-end VPN connection test
   - Requires Docker

2. **Exploratory Testing**
   ```bash
   bash scripts/exploratory-testing.sh
   ```
   - Comprehensive test suite (14 categories)
   - May require Docker/services

### Documentation Checks
1. **Markdown Linting**
   ```bash
   npx --yes markdownlint-cli2 "**/*.md" "!**/node_modules/**" "!**/target/**"
   ```
   - Checks markdown formatting
   - Non-blocking (continue-on-error: true)

2. **Link Checking**
   ```bash
   bash scripts/check-doc-links.sh
   ```
   - Validates markdown links
   - Non-blocking

3. **Emoji Gate**
   ```bash
   bash scripts/no-emoji-gate.sh
   ```
   - Ensures no emojis in documentation
   - Non-blocking

### Icon Generation/Validation
1. **Icon Generation**
   ```bash
   bash scripts/generate-icons.sh
   ```
   - Generates platform icons
   - Requires ImageMagick (optional)

2. **Icon Verification**
   ```bash
   bash scripts/verify-icons-min.sh
   ```
   - Fast icon coverage check
   - Non-blocking

## Environment Requirements

### Required
- Rust toolchain 1.83.0
- Cargo
- Git

### Optional (for full CI)
- Docker (for integration tests)
- ImageMagick (for icon generation)
- Node.js/npm (for web/gui tests)
- jq (for iOS validation)

## CI Workflow Reference
Primary CI workflow: `.github/workflows/ci.yml`
- Runs on: push to main, pull requests
- Steps match commands above
- Some steps are non-blocking (continue-on-error: true)

## Test Execution Order
1. Formatting check (fastest, catches style issues early)
2. Clippy (catches code quality issues)
3. Build (ensures compilation)
4. Unit tests (fast feedback)
5. KAT tests (cryptographic validation)
6. Property tests (cryptographic properties)
7. Security audits (cargo audit, cargo deny)
8. Integration tests (if Docker available)
9. Documentation checks (non-blocking)

## Success Criteria
- All required checks (fmt, clippy, build, tests, audit) must pass
- Optional checks (docs, icons) may have warnings but should not fail catastrophically
- CI badge should show green status

