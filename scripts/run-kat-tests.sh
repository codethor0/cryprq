#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Run all KAT (Known Answer Test) suites
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/kat}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "KAT (Known Answer Test) Suite"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# FIPS 203 ML-KEM (Kyber768) KATs
echo "Running FIPS 203 ML-KEM (Kyber768) KATs..."
if cargo test --package cryprq-crypto --lib kat_tests 2>&1 | tee "$ARTIFACT_DIR/fips203-kat.log"; then
    echo "✅ FIPS 203 KATs passed"
else
    echo "❌ FIPS 203 KATs failed"
    FAILED=1
fi
echo ""

# RFC 8439 ChaCha20-Poly1305 KATs
echo "Running RFC 8439 ChaCha20-Poly1305 KATs..."
if cargo test --package cryprq-crypto --lib rfc8439_kat 2>&1 | tee "$ARTIFACT_DIR/rfc8439-kat.log"; then
    echo "✅ RFC 8439 KATs passed"
else
    echo "❌ RFC 8439 KATs failed"
    FAILED=1
fi
echo ""

# RFC 7748 X25519 KATs
echo "Running RFC 7748 X25519 KATs..."
if cargo test --package cryprq-crypto --lib rfc7748_kat 2>&1 | tee "$ARTIFACT_DIR/rfc7748-kat.log"; then
    echo "✅ RFC 7748 KATs passed"
else
    echo "❌ RFC 7748 KATs failed"
    FAILED=1
fi
echo ""

if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ All KAT suites passed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ Some KAT suites failed - check logs in $ARTIFACT_DIR"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

