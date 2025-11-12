#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# API semver stability checks
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/semver}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "API SemVer Stability Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Install cargo-semver-checks if not present
if ! command -v cargo-semver-checks >/dev/null 2>&1; then
    echo "Installing cargo-semver-checks..."
    cargo install cargo-semver-checks --force || {
        echo "⚠️ cargo-semver-checks installation failed"
        echo "Skipping semver checks"
        exit 0
    }
fi

# Get last tag or baseline
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [ -z "$LAST_TAG" ]; then
    echo "⚠️ No tags found - skipping semver checks"
    echo "Note: Run after first release tag"
    exit 0
fi

echo "Checking against baseline: $LAST_TAG"
echo ""

FAILED=0

# Run semver checks for each crate
for crate in crypto core p2p node cli; do
    if [ -d "$crate" ] && [ -f "$crate/Cargo.toml" ]; then
        echo "Checking $crate..."
        if cargo semver-checks check-release --baseline "$LAST_TAG" --package "cryprq-$crate" 2>&1 | tee "$ARTIFACT_DIR/${crate}-semver.log"; then
            echo "✅ $crate: No breaking changes"
        else
            echo "❌ $crate: Potential breaking changes detected"
            FAILED=1
        fi
    fi
done

echo ""
if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ SemVer checks passed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ SemVer checks failed - breaking changes detected"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

