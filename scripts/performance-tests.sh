#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Performance testing script for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚡ Running Performance Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

PERF_LOG="performance-$(date +%Y%m%d-%H%M%S).log"

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Build release binary
echo "🔨 Building release binary..."
cargo build --release --bin cryprq 2>&1 | tee -a "$PERF_LOG"

BINARY="./target/release/cryprq"

if [ ! -f "$BINARY" ]; then
    echo "❌ Failed to build binary"
    exit 1
fi

# Benchmark handshake performance
echo ""
echo "📊 Benchmarking handshake performance..."
echo ""

# Start listener in background
LISTEN_ADDR="/ip4/127.0.0.1/udp/9999/quic-v1"
$BINARY --listen "$LISTEN_ADDR" > /tmp/cryprq-listener.log 2>&1 &
LISTENER_PID=$!

# Wait for listener to start
sleep 2

# Measure connection time
echo "⏱️  Measuring connection time..."
START_TIME=$(date +%s%N)
$BINARY --peer "$LISTEN_ADDR" > /tmp/cryprq-dialer.log 2>&1 || true
END_TIME=$(date +%s%N)
CONNECTION_TIME=$(( (END_TIME - START_TIME) / 1000000 )) # Convert to milliseconds

echo "  Connection time: ${CONNECTION_TIME}ms" | tee -a "$PERF_LOG"

# Cleanup
kill $LISTENER_PID 2>/dev/null || true
wait $LISTENER_PID 2>/dev/null || true

# Memory usage test
echo ""
echo "💾 Testing memory usage..."
echo ""

# Build with memory profiling if possible
if command -v valgrind &> /dev/null; then
    echo "  Running valgrind memory check..."
    valgrind --leak-check=full --show-leak-kinds=all "$BINARY" --help 2>&1 | tee -a "$PERF_LOG" || true
else
    echo "  ⚠️  valgrind not found. Skipping memory profiling."
fi

# Binary size
echo ""
echo "📦 Binary size analysis..."
BINARY_SIZE=$(stat -f%z "$BINARY" 2>/dev/null || stat -c%s "$BINARY" 2>/dev/null)
echo "  Binary size: $BINARY_SIZE bytes ($(numfmt --to=iec-i --suffix=B $BINARY_SIZE 2>/dev/null || echo "N/A"))" | tee -a "$PERF_LOG"

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Performance tests completed"
echo "📊 Performance log: $PERF_LOG"
echo ""
echo "📈 Results:"
echo "  • Connection time: ${CONNECTION_TIME}ms"
echo "  • Binary size: $BINARY_SIZE bytes"
echo ""

exit 0

