# Performance Guide

## Overview

This guide covers performance optimization, benchmarking, and profiling for CrypRQ.

## Quick Start

### Run Performance Benchmarks

```bash
## Basic performance check
bash scripts/performance-benchmark.sh

## Detailed benchmarks (requires nightly)
cargo +nightly bench

## Performance optimization analysis
bash scripts/optimize-performance.sh
```

### Build Optimizations

The project is configured with optimized build settings:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit (better optimization)
strip = true        # Strip symbols for smaller binary
```

### Current Performance Metrics

- **Binary Size**: ~11-13MB (optimized)
- **Startup Time**: <500ms
- **Build Time**: ~25-30s (release)
- **Test Time**: <2s (unit tests)

## Benchmarking

### Running Benchmarks

```bash
## Run all benchmarks
bash scripts/benchmark.sh

## Or directly with cargo
cargo +nightly bench
```

### Benchmark Targets

- **Handshake Performance**: ML-KEM + X25519 hybrid handshake
- **Key Rotation**: Key generation and rotation overhead
- **Packet Processing**: Encryption/decryption throughput
- **Memory Usage**: Allocation patterns and peak memory

### Interpreting Results

- Compare against baseline
- Look for regressions
- Identify bottlenecks
- Track improvements over time

## Profiling

### CPU Profiling

```bash
## Generate flamegraph
cargo install flamegraph
cargo flamegraph --bin cryprq

## Use perf (Linux)
perf record ./target/release/cryprq
perf report

## Use Instruments (macOS)
instruments -t 'Time Profiler' ./target/release/cryprq
```

### Memory Profiling

```bash
## Valgrind (Linux)
valgrind --leak-check=full ./target/release/cryprq

## Heaptrack (Linux)
heaptrack ./target/release/cryprq

## Memory profiling script
bash scripts/profile.sh
```

## Performance Targets

### Handshake

- **Target**: < 100ms for ML-KEM + X25519 hybrid
- **Measurement**: Time from key generation to shared secret

### Key Rotation

- **Target**: < 50ms for key rotation
- **Measurement**: Time to generate new keys and rotate

### Packet Processing

- **Target**: > 10,000 packets/second
- **Measurement**: Encrypt/decrypt throughput

### Memory

- **Target**: < 50MB peak memory
- **Measurement**: Peak RSS during operation

## Optimization Techniques

### 1. Reduce Allocations

```rust
// Bad: Allocates on every call
fn process(data: Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    // Process
    result
}

// Good: Reuse buffer
fn process_in_place(data: &mut [u8]) {
    // Process in place
}
```

### 2. Use Efficient Data Structures

- Use `Vec` for dynamic arrays
- Use `HashMap` for key-value lookups
- Use `BTreeMap` for ordered maps
- Consider `SmallVec` for small arrays

### 3. Avoid Unnecessary Clones

```rust
// Bad: Unnecessary clone
fn process(data: Vec<u8>) {
    let copy = data.clone();
    // Use copy
}

// Good: Use reference
fn process(data: &[u8]) {
    // Use data directly
}
```

### 4. Optimize Hot Paths

- Profile to identify hot paths
- Optimize critical sections
- Consider SIMD for bulk operations
- Use unsafe only when necessary

## Monitoring

### Metrics to Track

- Connection handshake time
- Key rotation frequency
- Packet processing rate
- Memory usage
- Error rates

### Tools

- Prometheus metrics
- Application logs
- System monitoring
- Custom dashboards

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion Benchmarking](https://github.com/bheisler/criterion.rs)
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)

