#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Build script for Linux (musl static binary)
# Produces a static binary that works on any Linux distribution

echo "=== CrypRQ Linux Build ==="
echo ""

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install Rust toolchain."; exit 1; }

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Rust version: $RUST_VERSION"

# Add musl target if not present
echo "Adding musl target..."
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

echo ""
echo "Building for x86_64-unknown-linux-musl..."
echo ""

# Build with musl target for static binary
cargo build --release --target x86_64-unknown-linux-musl

BINARY="target/x86_64-unknown-linux-musl/release/cryprq"

# Strip symbols to reduce binary size
if command -v strip >/dev/null 2>&1; then
    echo "Stripping symbols..."
    strip "$BINARY"
fi

echo ""
echo "=== Build Complete ==="
echo "Binary: $BINARY"
ls -lh "$BINARY"
echo ""
echo "To run:"
echo "  $BINARY --help"

