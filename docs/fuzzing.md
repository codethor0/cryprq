# Fuzz Testing Guide

## Overview

CrypRQ uses `cargo-fuzz` for automated fuzz testing of cryptographic operations and protocol parsing. This helps identify edge cases and potential security vulnerabilities.

## Fuzz Targets

### 1. `hybrid_handshake.rs`
Fuzz tests the hybrid ML-KEM + X25519 handshake creation and keypair generation.

**What it tests:**
- Hybrid handshake initialization
- ML-KEM keypair generation
- Key validation (non-zero checks)

### 2. `ppk_derivation.rs`
Fuzz tests Post-Quantum Pre-Shared Key (PPK) derivation.

**What it tests:**
- PPK derivation from ML-KEM shared secrets
- Expiration logic
- Key properties (non-zero, correct peer ID)

### 3. `key_rotation.rs`
Fuzz tests key rotation and PPK cleanup operations.

**What it tests:**
- PPK storage and retrieval
- Expired PPK cleanup
- Peer removal operations

### 4. `protocol_parse.rs`
Fuzz tests protocol parsing (multiaddr, peer IDs).

**What it tests:**
- String parsing edge cases
- Bounds checking
- Input validation

## Running Fuzz Tests Locally

### Prerequisites

```bash
cargo install cargo-fuzz
```

### Run All Targets

```bash
cd fuzz
cargo fuzz run fuzz_target_1
cargo fuzz run fuzz_target_2
cargo fuzz run fuzz_target_3
cargo fuzz run fuzz_target_4
```

### Run with Time Limit

```bash
cd fuzz
cargo fuzz run fuzz_target_1 -- -max_total_time=300  # 5 minutes
```

### Run with Corpus

```bash
cd fuzz
# Create corpus directory
mkdir -p corpus/fuzz_target_1

# Run with corpus
cargo fuzz run fuzz_target_1 -- -max_total_time=300 corpus/fuzz_target_1
```

## CI Integration

Fuzz tests run automatically on:
- Pull requests to `main`
- Weekly schedule (Sundays)
- Manual workflow dispatch

See `.github/workflows/fuzz.yml` for configuration.

## Interpreting Results

- **No crashes**: Target passes fuzz testing
- **Crash found**: Investigate the crash and fix the issue
- **Timeout**: Increase timeout or optimize the code

## Adding New Fuzz Targets

1. Create a new file in `fuzz/fuzz_targets/`
2. Add the target to `fuzz/Cargo.toml` `[[bin]]` section
3. Update `.github/workflows/fuzz.yml` to include the new target

Example:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzz test logic
});
```

## Best Practices

1. **Keep targets focused**: Each target should test one specific component
2. **Use bounds checking**: Always validate input sizes before processing
3. **Avoid panics**: Use `Result` types and proper error handling
4. **Test edge cases**: Include tests for empty inputs, maximum sizes, etc.

## Resources

- [cargo-fuzz documentation](https://github.com/rust-fuzz/cargo-fuzz)
- [libfuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)

