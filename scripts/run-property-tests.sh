#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Property tests with proptest (minimum 1k cases, shrinking enabled, seed recording)
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/prop}"
mkdir -p "$ARTIFACT_DIR"
mkdir -p tests/prop-failures

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Property Tests (proptest)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Parse command line arguments
CASES="${1:-1000}"
SEED_RECORD="${2:-record}"

# Set proptest environment variables for minimum cases and shrinking
export PROPTEST_CASES="$CASES"
export PROPTEST_SHRINK_TIME=1000

# Create seeds directory
mkdir -p tests/prop-failures

# Run property tests
if cargo test --package cryprq-crypto property_tests property_expanded 2>&1 | tee "$ARTIFACT_DIR/property-tests.log"; then
    echo "✅ Property tests passed"
    
    # Generate JUnit XML if possible
    if command -v cargo-test-junit >/dev/null 2>&1; then
        cargo test-junit --package cryprq-crypto > "$ARTIFACT_DIR/property-tests.xml" 2>/dev/null || true
    fi
    
    exit 0
else
    echo "❌ Property tests failed"
    
    # Save failure seeds if any
    if [ -d "tests/prop-failures" ]; then
        cp -r tests/prop-failures "$ARTIFACT_DIR/" || true
        echo "📋 Failure seeds saved to: $ARTIFACT_DIR/prop-failures/"
    fi
    
    exit 1
fi

