# Test Strategy for CrypRQ

## Overview
CrypRQ is a post-quantum VPN application built as a Rust workspace. This document defines a layered test strategy covering unit, integration, and end-to-end testing.

## Test Layers

### 1. Unit Tests (Fast Path - Run on Every Commit)

**Purpose**: Test individual functions and modules in isolation.

**Location**: 
- Embedded in source files using `#[cfg(test)]` modules
- Files: `*_tests.rs` or `tests` modules in `lib.rs`

**Framework**: Rust standard library (`cargo test`)

**Coverage Targets**:
- **crypto crate**: Cryptographic primitives, key generation, KAT validation
- **node crate**: Tunnel logic, packet handling, replay protection, rate limiting
- **p2p crate**: Swarm initialization, key management, peer dialing
- **core crate**: Core utilities and helpers

**Naming Convention**: `test_*` functions

**Execution**:
```bash
# All unit tests
cargo test --lib --all --no-fail-fast

# Specific crate
cargo test --package cryprq-crypto --lib

# Specific test module
cargo test --package cryprq-crypto kat_tests
```

**Target Duration**: < 30 seconds for full suite

**Current Status**: ✅ 39+ tests passing

### 2. Integration Tests (Medium Path - Run on PRs)

**Purpose**: Test interactions between components and services.

**Location**: 
- `tests/` directory (Rust integration tests)
- `scripts/docker_vpn_test.sh` and related scripts

**Types**:
- **VPN Connection Tests**: Listener/dialer handshake, packet forwarding
- **Network Tests**: libp2p QUIC transport, peer discovery
- **Docker Integration**: Containerized service interactions

**Execution**:
```bash
# Rust integration tests
cargo test --test '*'

# Docker VPN integration test
./scripts/docker_vpn_test.sh

# Using docker-compose
docker compose -f docker-compose.test.yml run --rm cryprq-test-runner cargo test --all
```

**Target Duration**: < 2 minutes

**Dependencies**: Docker, network access

**Current Status**: ⚠️ Scripts exist but need standardization

### 3. End-to-End Tests (Heavy Path - Run on Main Branch/Nightly)

**Purpose**: Test complete user workflows and system behavior.

**Location**:
- `tests/browser-docker.spec.ts` - Browser E2E tests
- `tests/web.spec.ts` - Web UI tests
- Manual testing scripts

**Types**:
- **Browser E2E**: Web UI interactions (if applicable)
- **VPN E2E**: Complete VPN connection lifecycle
- **Performance Tests**: Benchmarks and load testing

**Execution**:
```bash
# Browser tests (requires Node.js)
npm test

# Full E2E suite
make local-validate
```

**Target Duration**: < 10 minutes

**Dependencies**: Docker, Node.js (for browser tests), network

**Current Status**: ⚠️ Partially implemented

## Test Framework Selection

### Rust Tests
- **Framework**: Built-in `cargo test`
- **Rationale**: Standard, no external dependencies, fast
- **Coverage**: Use `cargo-llvm-cov` or `cargo-tarpaulin` for coverage reports

### Integration Tests
- **Framework**: Docker Compose + Rust tests
- **Rationale**: Reproducible environment, matches production
- **Coverage**: Test critical paths and error scenarios

### E2E Tests
- **Framework**: Playwright (for browser), Docker scripts (for VPN)
- **Rationale**: Real browser testing, actual network conditions
- **Coverage**: User-facing workflows

## Test Organization

### Directory Structure
```
.
├── crypto/
│   ├── src/
│   │   ├── tests.rs          # Unit tests
│   │   ├── kat_tests.rs      # KAT tests
│   │   └── property_tests.rs # Property tests
│   └── tests/                 # Integration tests (if needed)
├── node/
│   └── src/
│       └── tests.rs           # Unit tests
├── p2p/
│   └── src/
│       └── tests.rs           # Unit tests
├── tests/                     # Integration/E2E tests
│   ├── browser-docker.spec.ts
│   └── web.spec.ts
└── scripts/                   # Test automation scripts
    ├── docker_vpn_test.sh
    └── test-*.sh
```

### Test Naming Conventions
- **Unit tests**: `test_<functionality>` (e.g., `test_tunnel_creation`)
- **Integration tests**: `test_<component>_<interaction>` (e.g., `test_vpn_handshake`)
- **E2E tests**: Descriptive names matching user workflows

## Canonical Test Commands

### Local Development (Without Docker)
```bash
# Fast path - unit tests only
cargo test --lib --all --no-fail-fast

# With formatting and linting
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --lib --all
```

### Docker Environment (Recommended)
```bash
# Standardized Docker test command
docker compose -f docker-compose.test.yml run --rm test cargo test --lib --all --no-fail-fast

# Integration tests
docker compose -f docker-compose.test.yml run --rm test ./scripts/docker_vpn_test.sh
```

### CI/CD
```bash
# CI runs the same commands as local
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --all --no-fail-fast
cargo build --release -p cryprq
```

## Test Data and Fixtures

### Current Approach
- Tests use minimal, deterministic data
- No external test fixtures (preferred for unit tests)
- Docker containers provide isolated environments for integration tests

### Recommendations
- Keep unit tests deterministic (no random data unless testing randomness)
- Use constants for test data (e.g., `[1u8; 32]` for keys)
- Integration tests can use Docker networks for isolation

## Test Stability and Flakiness

### Known Issues
1. **Network-dependent tests**: May fail if network is unavailable
   - **Mitigation**: Use Docker networks, add retries with timeouts
   
2. **Timing-dependent tests**: Rate limiting, timeouts
   - **Mitigation**: Use deterministic time sources where possible, generous timeouts

3. **Concurrent tests**: May interfere with each other
   - **Mitigation**: Use isolated test data, avoid shared state

### Best Practices
- Use `#[tokio::test]` for async tests
- Avoid `sleep()` in tests; use proper async waiting
- Use `--no-fail-fast` to see all failures
- Mark flaky tests with `#[ignore]` and document why

## Coverage Goals

### Current Coverage
- **Unit Tests**: ~39 tests across crates
- **Integration Tests**: Script-based, needs standardization
- **E2E Tests**: Partial browser tests

### Target Coverage
- **Unit Tests**: 80%+ of critical paths
- **Integration Tests**: All major component interactions
- **E2E Tests**: Core user workflows

### Coverage Tools
- `cargo-llvm-cov` or `cargo-tarpaulin` for Rust
- Generate reports: `cargo llvm-cov --all-features -- --lib --all`

## Test Execution Strategy

### On Every Commit (Fast Path)
- Formatting check
- Clippy linting
- Unit tests (`cargo test --lib --all`)

### On Pull Requests (Medium Path)
- Fast path +
- Integration tests
- Build verification
- Security audits

### On Main Branch (Full Path)
- Medium path +
- E2E tests
- Performance benchmarks
- Extended fuzzing (nightly)

## Docker Test Environment

### Current State
- `docker-compose.test.yml` exists but test runner uses `sleep infinity`
- No standardized test command
- Manual scripts for integration tests

### Target State
- Standardized `test` service in docker-compose
- Single command to run all tests: `docker compose run --rm test`
- Test service includes all dependencies (cargo, rustc, etc.)
- Proper cleanup and isolation

## Documentation Requirements

### README Updates Needed
- Add "Testing" section
- Document Docker test workflow
- Link to this strategy document

### CONTRIBUTING.md Updates Needed
- Standardize test commands
- Document Docker test setup
- Explain test layers

## Success Criteria

✅ **Phase 1 Complete When**:
- Unit tests run reliably in Docker
- Integration tests standardized
- Single command runs all tests
- Documentation updated

✅ **Phase 2 Complete When**:
- E2E tests integrated
- Coverage reporting working
- CI passes consistently
- Test strategy documented

