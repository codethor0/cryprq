#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Reproducible build verification with diffoscope
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/reproducible}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Reproducible Build Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Set deterministic build environment
export SOURCE_DATE_EPOCH=0
export TZ=UTC
export LANG=C
export LC_ALL=C

# Build 1
echo "Build 1: Clean build..."
cargo clean -p cryprq
cargo build --release -p cryprq
BUILD1_BINARY="target/release/cryprq"
BUILD1_CHECKSUM=$(shasum -a 256 "$BUILD1_BINARY" 2>/dev/null | cut -d' ' -f1 || echo "checksum_failed")
echo "Build 1 checksum: $BUILD1_CHECKSUM"
echo ""

# Build 2
echo "Build 2: Clean rebuild..."
cargo clean -p cryprq
cargo build --release -p cryprq
BUILD2_BINARY="target/release/cryprq"
BUILD2_CHECKSUM=$(shasum -a 256 "$BUILD2_BINARY" 2>/dev/null | cut -d' ' -f1 || echo "checksum_failed")
echo "Build 2 checksum: $BUILD2_CHECKSUM"
echo ""

# Compare checksums
if [ "$BUILD1_CHECKSUM" = "$BUILD2_CHECKSUM" ] && [ "$BUILD1_CHECKSUM" != "checksum_failed" ]; then
    echo "✅ Checksums match - builds are reproducible"
else
    echo "❌ Checksums differ - builds are not reproducible"
    echo ""
    
    # Use diffoscope if available
    if command -v diffoscope >/dev/null 2>&1; then
        echo "Running diffoscope analysis..."
        diffoscope "$BUILD1_BINARY" "$BUILD2_BINARY" > "$ARTIFACT_DIR/diffoscope.txt" 2>&1 || true
        echo "📄 Diffoscope report: $ARTIFACT_DIR/diffoscope.txt"
    else
        echo "⚠️ diffoscope not installed - install with: pip install diffoscope"
        echo "Comparing binaries with diff..."
        diff "$BUILD1_BINARY" "$BUILD2_BINARY" > "$ARTIFACT_DIR/diff.txt" 2>&1 || true
    fi
    
    exit 1
fi

# Musl build (if available)
if rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
    echo ""
    echo "Musl build verification..."
    cargo clean -p cryprq
    cargo build --release --target x86_64-unknown-linux-musl -p cryprq
    MUSL_BINARY="target/x86_64-unknown-linux-musl/release/cryprq"
    MUSL_CHECKSUM=$(shasum -a 256 "$MUSL_BINARY" 2>/dev/null | cut -d' ' -f1 || echo "checksum_failed")
    echo "Musl checksum: $MUSL_CHECKSUM"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Reproducible build verification complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

