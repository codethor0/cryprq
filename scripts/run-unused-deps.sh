#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Unused dependencies check
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/unused-deps}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Unused Dependencies Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# Try cargo-udeps first
if command -v cargo-udeps >/dev/null 2>&1; then
    echo "Running cargo-udeps..."
    if cargo udeps --all-targets --all-features 2>&1 | tee "$ARTIFACT_DIR/udeps.log"; then
        echo "✅ No unused dependencies found"
    else
        echo "❌ Unused dependencies found"
        FAILED=1
    fi
# Fallback to cargo-machete
elif command -v cargo-machete >/dev/null 2>&1; then
    echo "Running cargo-machete..."
    if cargo machete 2>&1 | tee "$ARTIFACT_DIR/machete.log"; then
        echo "✅ No unused dependencies found"
    else
        echo "❌ Unused dependencies found"
        FAILED=1
    fi
# Fallback to cargo check with unused-crate-dependencies
else
    echo "Using cargo check with unused-crate-dependencies lint..."
    if cargo check --all-targets --all-features 2>&1 | grep -E "unused.*dependency" | tee "$ARTIFACT_DIR/unused-check.log"; then
        echo "❌ Unused dependencies found"
        FAILED=1
    else
        echo "✅ No unused dependencies found"
    fi
fi

echo ""
if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ Unused dependencies check passed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ Unused dependencies check failed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

