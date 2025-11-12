#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Run sanitizer builds (ASan + UBSan)
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/sanitizers}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Sanitizer Builds (ASan + UBSan)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if sanitizers are supported
if ! rustup show | grep -q nightly; then
    echo "⚠️ Nightly toolchain required for sanitizers"
    echo "Installing nightly..."
    rustup toolchain install nightly
fi

rustup override set nightly || true

FAILED=0

# ASan build and test
echo "Building with AddressSanitizer..."
export RUSTFLAGS="-Zsanitizer=address"
if cargo +nightly test --all --lib 2>&1 | tee "$ARTIFACT_DIR/asan.log"; then
    echo "✅ ASan tests passed"
else
    echo "❌ ASan tests failed"
    FAILED=1
fi
unset RUSTFLAGS
echo ""

# UBSan build and test
echo "Building with UndefinedBehaviorSanitizer..."
export RUSTFLAGS="-Zsanitizer=undefined"
if cargo +nightly test --all --lib 2>&1 | tee "$ARTIFACT_DIR/ubsan.log"; then
    echo "✅ UBSan tests passed"
else
    echo "❌ UBSan tests failed"
    FAILED=1
fi
unset RUSTFLAGS
echo ""

# Restore default toolchain
rustup override set 1.83.0 || true

if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ All sanitizer tests passed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ Sanitizer tests failed - check logs in $ARTIFACT_DIR"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

