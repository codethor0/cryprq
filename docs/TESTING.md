# Testing Guide

## Overview

CrypRQ includes comprehensive testing infrastructure covering unit tests, integration tests, and end-to-end tests.

## Test Types

### Unit Tests

Test individual components in isolation.

```bash
bash scripts/test-unit.sh
```

Or directly:

```bash
cargo test --lib --all
```

### Integration Tests

Test component interactions and Docker-based scenarios.

```bash
bash scripts/test-integration.sh
```

Or directly:

```bash
cargo test --test '*'
bash scripts/docker_vpn_test.sh
```

### End-to-End Tests

Test complete application workflows.

```bash
bash scripts/test-e2e.sh
```

Uses Docker Compose to spin up full test environment.

## Test Organization

```
tests/
├── unit_tests.rs          # Unit test examples
├── integration_tests.rs   # Integration test examples
└── e2e_tests.rs          # E2E test examples
```

## Writing Tests

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake() {
        // Test implementation
    }
}
```

### Integration Test Example

```rust
// tests/integration_test.rs
use cryprq_crypto::HybridHandshake;

#[test]
fn test_handshake_integration() {
    // Integration test implementation
}
```

## Test Coverage

### Generate Coverage Report

```bash
## Install cargo-tarpaulin
cargo install cargo-tarpaulin

## Generate coverage
cargo tarpaulin --out Html
```

### View Coverage

Open `tarpaulin-report.html` in a browser.

## Continuous Integration

### GitHub Actions

Tests run automatically on:
- Push to `main` branch
- Pull requests
- Manual workflow dispatch

### Local CI Simulation

```bash
## Run all tests
bash scripts/test-unit.sh
bash scripts/test-integration.sh
bash scripts/test-e2e.sh
```

## Mobile Testing

### Android

```bash
bash scripts/test-android.sh
```

Runs:
- Unit tests (`./gradlew test`)
- Instrumented tests (`./gradlew connectedAndroidTest`)
- Detox E2E tests (if configured)

### iOS

```bash
bash scripts/test-ios.sh
```

Runs:
- Jest unit tests
- Xcode unit tests
- Detox E2E tests (if configured)

## Performance Testing

```bash
bash scripts/performance-tests.sh
```

Measures:
- Connection handshake time
- Memory usage
- Binary size

## Test Data

### Mock Data

Create mock data in `tests/fixtures/`:

```
tests/
└── fixtures/
    ├── test_keys.json
    └── test_config.toml
```

### Test Utilities

Create shared test utilities in `tests/common/`:

```rust
// tests/common/mod.rs
pub mod helpers;
pub mod fixtures;
```

## Best Practices

1. **Isolate tests**: Each test should be independent
2. **Use fixtures**: Reuse test data and setup
3. **Clean up**: Remove temporary files and containers
4. **Document**: Add comments for complex test scenarios
5. **Fast tests**: Keep unit tests fast (< 1s each)
6. **Deterministic**: Tests should produce consistent results

## Troubleshooting

### Tests Fail Locally But Pass in CI

- Check environment differences
- Verify dependencies are installed
- Check for race conditions

### Docker Tests Fail

```bash
## Clean up containers
docker compose down -v
docker system prune -f

## Rebuild images
docker compose build --no-cache
```

### Flaky Tests

- Add retries for network-dependent tests
- Use timeouts appropriately
- Check for race conditions

## References

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

