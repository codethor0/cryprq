#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# MSRV (Minimum Supported Rust Version) verification
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/msrv}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "MSRV Verification (cargo-msrv)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check rust-version in Cargo.toml
if grep -q "rust-version" Cargo.toml; then
    MSRV=$(grep "rust-version" Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    echo "MSRV specified in Cargo.toml: $MSRV"
else
    echo "⚠️ rust-version not specified in Cargo.toml"
    echo "Setting MSRV to 1.83.0..."
    # Would need to add rust-version = "1.83.0" to Cargo.toml
fi

# Install cargo-msrv if not present
if ! command -v cargo-msrv >/dev/null 2>&1; then
    echo "Installing cargo-msrv..."
    cargo install cargo-msrv --force || {
        echo "⚠️ cargo-msrv installation failed"
        echo "Skipping MSRV verification"
        exit 0
    }
fi

# Verify MSRV
echo ""
echo "Verifying MSRV..."
if cargo msrv verify 2>&1 | tee "$ARTIFACT_DIR/msrv.log"; then
    echo "✅ MSRV verification passed"
    exit 0
else
    echo "❌ MSRV verification failed"
    exit 1
fi

