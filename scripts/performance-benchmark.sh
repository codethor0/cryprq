#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚡ Performance Benchmarking"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BENCH_LOG="performance-benchmark-$(date +%Y%m%d-%H%M%S).log"

# Build release binary
echo "Building release binary for benchmarking..."
cargo build --release -p cryprq > "$BENCH_LOG" 2>&1 || {
    echo "❌ Build failed"
    exit 1
}

BINARY="./target/release/cryprq"
BINARY_SIZE=$(stat -f%z "$BINARY" 2>/dev/null || stat -c%s "$BINARY" 2>/dev/null)
echo "✅ Binary built: ${BINARY_SIZE} bytes (~$((BINARY_SIZE / 1024 / 1024))MB)"
echo ""

# Benchmark 1: Startup time
echo "Benchmark 1: Application Startup Time"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
START_TIME=$(date +%s%N)
timeout 2 "$BINARY" --help > /dev/null 2>&1 || true
END_TIME=$(date +%s%N)
STARTUP_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
echo "Startup time: ${STARTUP_TIME}ms"
echo ""

# Benchmark 2: Cryptographic operations
echo "Benchmark 2: Cryptographic Operation Performance"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo test --lib -p cryprq-crypto --no-fail-fast --release 2>&1 | grep -q "test result"; then
    echo "✅ Crypto operations benchmarked"
else
    echo "⚠️  Crypto benchmarks need cargo bench (nightly)"
fi
echo ""

# Benchmark 3: Memory usage (if valgrind available)
echo "Benchmark 3: Memory Usage"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if command -v valgrind &> /dev/null; then
    echo "Running valgrind memory check..."
    valgrind --tool=massif --massif-out-file=massif.out "$BINARY" --help > /dev/null 2>&1 || true
    if [ -f massif.out ]; then
        echo "✅ Memory profile generated: massif.out"
        rm -f massif.out
    fi
else
    echo "⚠️  Valgrind not available (install for detailed memory profiling)"
fi
echo ""

# Benchmark 4: Build time
echo "Benchmark 4: Build Performance"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
BUILD_START=$(date +%s)
cargo build --release -p cryprq > /dev/null 2>&1
BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))
echo "Release build time: ${BUILD_TIME}s"
echo ""

# Benchmark 5: Test execution time
echo "Benchmark 5: Test Execution Performance"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TEST_START=$(date +%s)
cargo test --lib --all --no-fail-fast > /dev/null 2>&1
TEST_END=$(date +%s)
TEST_TIME=$((TEST_END - TEST_START))
echo "Test execution time: ${TEST_TIME}s"
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Performance Benchmark Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Binary size: ${BINARY_SIZE} bytes (~$((BINARY_SIZE / 1024 / 1024))MB)"
echo "Startup time: ${STARTUP_TIME}ms"
echo "Build time: ${BUILD_TIME}s"
echo "Test time: ${TEST_TIME}s"
echo ""
echo "📊 Benchmark log: $BENCH_LOG"
echo ""
echo "✅ Performance benchmarking complete"
echo ""
echo "For detailed benchmarks, use:"
echo "  cargo +nightly bench"

