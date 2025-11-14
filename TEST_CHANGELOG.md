# Test Changelog

This document tracks significant changes to the test infrastructure and test suite.

## 2025-11-13 - Dockerized Test Environment Setup

### Added
- **Dockerfile.test**: Test-specific Dockerfile optimized for running tests
  - Based on `rust:1.83-slim`
  - Includes cargo, rustc, and test dependencies
  - Optimized layer caching for faster builds

- **docker-compose.test.yml improvements**:
  - Renamed `cryprq-test-runner` to `test` for clarity
  - Added `test-integration` service for integration tests requiring listener
  - Added cargo cache volume for faster test runs
  - Separated unit tests (no dependencies) from integration tests (require listener)

- **TESTING_INVENTORY.md**: Comprehensive inventory of existing test infrastructure
- **TEST_STRATEGY.md**: Layered test strategy (unit/integration/E2E)

### Changed
- Test runner service now runs tests by default instead of `sleep infinity`
- Unit tests can run without listener service (faster, no port conflicts)
- Integration tests explicitly depend on listener service

### Test Execution
- **Unit Tests**: `docker compose -f docker-compose.test.yml --profile test run --rm test`
- **Integration Tests**: `docker compose -f docker-compose.test.yml --profile test-integration run --rm test-integration`
- **Local (no Docker)**: `cargo test --lib --all --no-fail-fast`

### Rationale
- Standardized Docker test environment for reproducible test runs
- Faster unit test execution (no need to wait for listener)
- Clear separation between unit and integration tests
- Better caching for faster CI runs

## Previous Test Infrastructure

### Existing Tests (Pre-2025-11-13)
- **Unit Tests**: ~39 tests embedded in source files
  - crypto: 15+ tests (KAT, property, basic)
  - node: 24+ tests (tunnel, replay, rate limiting)
  - p2p: 8+ tests (swarm, keys, concurrency)

- **Integration Tests**: Script-based Docker VPN tests
  - `scripts/docker_vpn_test.sh` - Main integration test
  - `scripts/docker-smoketest.sh` - Quick smoke test

- **E2E Tests**: Partial browser tests (Playwright)

### Test Commands
- `cargo test --lib --all --no-fail-fast` - Unit tests
- `cargo test --package cryprq-crypto kat_tests` - KAT tests
- `cargo test --package cryprq-crypto property_tests` - Property tests
- `./scripts/docker_vpn_test.sh` - Integration tests

