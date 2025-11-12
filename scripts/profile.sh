#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Profiling script for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Running Performance Profiling"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Build with profiling flags
echo "🔨 Building with profiling flags..."
cargo build --release --profile release-with-debug

# Check for perf (Linux) or Instruments (macOS)
if command -v perf &> /dev/null; then
    echo "📊 Using perf for profiling (Linux)..."
    echo "   Run: perf record ./target/release/cryprq"
    echo "   View: perf report"
elif command -v instruments &> /dev/null; then
    echo "📊 Using Instruments for profiling (macOS)..."
    echo "   Run: instruments -t 'Time Profiler' ./target/release/cryprq"
else
    echo "⚠️  No profiling tools found. Install perf (Linux) or use Instruments (macOS)"
fi

# Check for cargo-flamegraph
if command -v cargo-flamegraph &> /dev/null; then
    echo ""
    echo "🔥 Generating flamegraph..."
    cargo flamegraph --bin cryprq -- --help 2>&1 | head -5 || true
    echo "   Flamegraph saved to flamegraph.svg"
else
    echo ""
    echo "💡 Install cargo-flamegraph for visual profiling:"
    echo "   cargo install flamegraph"
fi

# Memory profiling with valgrind (if available)
if command -v valgrind &> /dev/null; then
    echo ""
    echo "💾 Running memory profiling with valgrind..."
    valgrind --leak-check=full --show-leak-kinds=all \
        ./target/release/cryprq --help 2>&1 | head -20 || true
else
    echo ""
    echo "💡 Install valgrind for memory profiling (Linux only)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Profiling tools ready"
echo ""

exit 0

