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

# Set proptest environment variables for minimum cases and shrinking
export PROPTEST_CASES=1000
export PROPTEST_SHRINK_TIME=1000

# Run property tests
if cargo test --package cryprq-crypto property_tests property_expanded 2>&1 | tee "$ARTIFACT_DIR/property-tests.log"; then
    echo "✅ Property tests passed"
    exit 0
else
    echo "❌ Property tests failed"
    
    # Save failure seeds if any
    if [ -d "tests/prop-failures" ]; then
        cp -r tests/prop-failures "$ARTIFACT_DIR/" || true
    fi
    
    exit 1
fi

