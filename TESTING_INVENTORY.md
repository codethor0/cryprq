# Testing Inventory for CrypRQ Repository

## Repository Overview
- **Type**: Post-quantum VPN application (Rust workspace)
- **Primary Language**: Rust (2021 edition)
- **Toolchain**: Rust 1.83.0
- **Architecture**: Multi-crate workspace (crypto, p2p, node, cli, core, fuzz, benches)

## Languages and Frameworks Detected

### Primary Stack
- **Rust**: Main application language
  - Edition: 2021
  - Workspace with 7 crates
  - Uses tokio for async runtime
  - Uses libp2p for peer-to-peer networking

### Secondary Components
- **TypeScript/JavaScript**: Web UI and mobile apps (gui/, mobile/, web/)
- **Shell Scripts**: Extensive bash scripts for testing and automation
- **Docker**: Containerization for deployment and testing

## Existing Test Infrastructure

### Test Frameworks
1. **Rust Built-in Testing**
   - Framework: `cargo test` (Rust standard library)
   - Location: Tests embedded in source files (`#[cfg(test)]` modules)
   - Test files found:
     - `crypto/src/tests.rs` - Basic crypto tests
     - `crypto/src/kat_tests.rs` - Known Answer Tests (KAT)
     - `crypto/src/property_tests.rs` - Property-based tests
     - `node/src/tests.rs` - Tunnel and node tests
     - `p2p/src/tests.rs` - P2P networking tests
     - Various `#[cfg(test)]` modules in other source files

2. **Integration Tests**
   - Location: `tests/` directory
   - Files:
     - `tests/browser-docker.spec.ts` - Browser tests (Playwright)
     - `tests/web.spec.ts` - Web UI tests

3. **Docker-based Integration Tests**
   - Scripts:
     - `scripts/docker_vpn_test.sh` - Main VPN integration test
     - `scripts/docker-smoketest.sh` - Quick smoke test
     - `scripts/docker_vpn_pairtest.sh` - Pair testing
     - `scripts/docker_vpn_idtest.sh` - Identity testing

### Test Commands Identified

#### From CI Workflow (`.github/workflows/ci.yml`)
```bash
# Formatting
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Unit Tests
cargo test --lib --all --no-fail-fast

# KAT Tests
cargo test --package cryprq-crypto kat_tests

# Property Tests
cargo test --package cryprq-crypto property_tests

# Build
cargo build --release -p cryprq

# Security
cargo audit
cargo deny check
```

#### From CONTRIBUTING.md
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --release
cargo audit --deny warnings
cargo deny check advisories bans sources licenses --deny vulnerability --deny unmaintained --deny unsound --warn notice
./scripts/docker_vpn_test.sh
```

#### From Makefile
```bash
make quick-smoke      # Fast local sanity (30s)
make local-validate   # Full validation
make validate         # Alias for quick-smoke
```

### Test Statistics
- **Unit Tests**: ~50+ tests across multiple crates
  - crypto: 15+ tests (KAT, property, basic)
  - node: 24+ tests (tunnel, replay window, rate limiting, buffer pool)
  - p2p: 8+ tests (swarm, keys, concurrency)
  - Other modules: Various tests in lib.rs files

- **Integration Tests**: 
  - Docker VPN tests (listener/dialer)
  - Browser tests (Playwright)

## Docker Infrastructure

### Existing Docker Files
1. **Dockerfile**
   - Base: `rust:1.83` (builder) + `debian:bookworm-slim` (runtime)
   - Purpose: Production binary build
   - Multi-stage build for optimization

2. **docker-compose.yml**
   - Services:
     - `cryprq-listener` - Main listener node
     - `cryprq-dialer` - Dialer node (test profile)
     - `cryprq-test-runner` - Test runner service (test profile)
   - Network: `cryprq-network` (bridge)
   - Health checks configured

3. **docker-compose.test.yml**
   - Purpose: Test-specific configuration
   - Status: Exists but needs review

4. **docker-compose.vpn.yml**
   - Purpose: VPN-specific setup
   - Status: Exists

### Docker Test Usage
- Current approach: Manual scripts (`docker_vpn_test.sh`)
- Test runner service exists but uses `sleep infinity` (needs improvement)
- No standardized Docker test command

## CI/CD Configuration

### GitHub Actions Workflows
1. **`.github/workflows/ci.yml`** - Main CI
   - Runs: fmt, clippy, tests, build, security audits
   - Environment: ubuntu-latest
   - Toolchain: Rust 1.83.0

2. **`.github/workflows/qa-vnext.yml`** - Extended QA
   - Comprehensive testing including fuzzing, Miri, sanitizers

3. **`.github/workflows/fuzz.yml`** - Fuzz testing
   - Uses nightly toolchain
   - Runs cargo-fuzz

## Gaps and Opportunities

### Missing Test Infrastructure
1. **No standardized Docker test command**
   - Test runner service exists but not properly configured
   - No `docker compose run test` equivalent

2. **Limited integration test coverage**
   - Docker tests exist but are script-based
   - No standardized integration test suite

3. **No test-specific Dockerfile**
   - Current Dockerfile is production-focused
   - Test environment needs cargo, rustc, etc.

4. **Missing E2E test framework**
   - Some browser tests exist but not integrated into Docker workflow
   - No comprehensive E2E test suite

### Documentation Gaps
1. **No clear testing documentation**
   - Testing approach not documented in README
   - No guide for running tests in Docker

2. **Test strategy not documented**
   - No clear separation of unit/integration/E2E
   - No test coverage reporting

### Test Quality Issues
1. **Some tests may be flaky**
   - Network-dependent tests (Docker VPN tests)
   - Timing-dependent tests (rate limiting, timeouts)

2. **Missing test fixtures**
   - No standardized test data
   - No mock services for external dependencies

## Current Test Execution

### Local (Without Docker)
```bash
# Unit tests
cargo test --lib --all --no-fail-fast

# Specific test suites
cargo test --package cryprq-crypto kat_tests
cargo test --package cryprq-crypto property_tests

# Integration tests (requires Docker)
./scripts/docker_vpn_test.sh
```

### Docker (Current)
```bash
# Build image
docker build -t cryprq-node .

# Run listener
docker run --rm -p 9999:9999/udp cryprq-node --listen /ip4/0.0.0.0/udp/9999/quic-v1

# Run dialer (separate container)
docker run --rm cryprq-node --peer /ip4/<listener-ip>/udp/9999/quic-v1

# Integration test script
./scripts/docker_vpn_test.sh
```

## Environment Requirements

### Required
- Rust toolchain 1.83.0
- Docker and docker-compose
- Bash (for scripts)

### Optional
- Node.js/npm (for web/mobile tests)
- ImageMagick (for icon generation)
- jq (for iOS validation)

## Next Steps

1. Create standardized Docker test environment
2. Document test strategy (unit/integration/E2E)
3. Improve test runner service in docker-compose.yml
4. Add test coverage reporting
5. Create comprehensive test documentation

