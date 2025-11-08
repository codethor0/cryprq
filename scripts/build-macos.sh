#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Build script for macOS (native binary)
# Produces a native macOS binary

echo "=== CrypRQ macOS Build ==="
echo ""

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install Rust toolchain."; exit 1; }

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Rust version: $RUST_VERSION"

# Check if we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "ERROR: This script is for macOS only"
    exit 1
fi

echo ""
echo "Building for macOS (x86_64-apple-darwin or aarch64-apple-darwin)..."
echo ""

# Build release binary
cargo build --release

# Determine architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
    TARGET="aarch64-apple-darwin"
else
    TARGET="x86_64-apple-darwin"
fi

BINARY="target/release/cryprq"

# Strip symbols to reduce binary size
if command -v strip >/dev/null 2>&1; then
    echo "Stripping symbols..."
    strip "$BINARY"
fi

echo ""
echo "=== Build Complete ==="
echo "Target: $TARGET"
echo "Binary: $BINARY"
ls -lh "$BINARY"
echo ""
echo "To run:"
echo "  $BINARY --help"

