#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Reproducible Build Validation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

OUTPUT_DIR="${1:-artifacts/reproducible}"
mkdir -p "$OUTPUT_DIR"

# Build 1
echo "Build 1: Initial build..."
cargo build --release -p cryprq 2>&1 | tee "$OUTPUT_DIR/build1.log"
BINARY1="target/release/cryprq"
HASH1=$(shasum -a 256 "$BINARY1" 2>/dev/null | awk '{print $1}' || md5sum "$BINARY1" 2>/dev/null | awk '{print $1}')
echo "Build 1 hash: $HASH1" | tee -a "$OUTPUT_DIR/build1.log"

# Clean
echo ""
echo "Cleaning build artifacts..."
cargo clean

# Build 2
echo ""
echo "Build 2: Reproducible build..."
cargo build --release -p cryprq 2>&1 | tee "$OUTPUT_DIR/build2.log"
BINARY2="target/release/cryprq"
HASH2=$(shasum -a 256 "$BINARY2" 2>/dev/null | awk '{print $1}' || md5sum "$BINARY2" 2>/dev/null | awk '{print $1}')
echo "Build 2 hash: $HASH2" | tee -a "$OUTPUT_DIR/build2.log"

# Compare
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$HASH1" = "$HASH2" ]; then
    echo "✅ REPRODUCIBLE: Builds match (hash: $HASH1)"
    echo "✅ Reproducible build validation PASSED"
    exit 0
else
    echo "❌ NOT REPRODUCIBLE: Builds differ"
    echo "Build 1 hash: $HASH1"
    echo "Build 2 hash: $HASH2"
    echo "⚠️  Reproducible build validation FAILED"
    echo ""
    echo "Note: Some non-determinism is expected (timestamps, build paths)."
    echo "For true reproducibility, use Docker or Nix builds."
    exit 1
fi

