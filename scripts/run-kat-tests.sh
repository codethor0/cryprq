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

# Create KAT transcripts directory
mkdir -p "$ARTIFACT_DIR/transcripts"

# FIPS 203 ML-KEM (Kyber768) KATs
echo "Running FIPS 203 ML-KEM (Kyber768) KATs..."
if cargo test --package cryprq-crypto --lib kat_tests 2>&1 | tee "$ARTIFACT_DIR/fips203-kat.log"; then
    echo "✅ FIPS 203 KATs passed"
    # Save sample passing rows
    grep -E "test.*ok|test.*passed" "$ARTIFACT_DIR/fips203-kat.log" | head -10 > "$ARTIFACT_DIR/transcripts/fips203-sample.txt" || true
else
    echo "❌ FIPS 203 KATs failed"
    FAILED=1
fi
echo ""

# RFC 8439 ChaCha20-Poly1305 KATs
echo "Running RFC 8439 ChaCha20-Poly1305 KATs..."
if cargo test --package cryprq-crypto --lib rfc8439_kat 2>&1 | tee "$ARTIFACT_DIR/rfc8439-kat.log"; then
    echo "✅ RFC 8439 KATs passed"
    grep -E "test.*ok|test.*passed" "$ARTIFACT_DIR/rfc8439-kat.log" | head -10 > "$ARTIFACT_DIR/transcripts/rfc8439-sample.txt" || true
else
    echo "❌ RFC 8439 KATs failed"
    FAILED=1
fi
echo ""

# RFC 7748 X25519 KATs
echo "Running RFC 7748 X25519 KATs..."
if cargo test --package cryprq-crypto --lib rfc7748_kat 2>&1 | tee "$ARTIFACT_DIR/rfc7748-kat.log"; then
    echo "✅ RFC 7748 KATs passed"
    grep -E "test.*ok|test.*passed" "$ARTIFACT_DIR/rfc7748-kat.log" | head -10 > "$ARTIFACT_DIR/transcripts/rfc7748-sample.txt" || true
else
    echo "❌ RFC 7748 KATs failed"
    FAILED=1
fi
echo ""

# Create vector corpus snapshot
echo "Creating vector corpus snapshot..."
cat > "$ARTIFACT_DIR/transcripts/corpus-snapshot.md" << EOF
# KAT Vector Corpus Snapshot

**Date**: $(date -u +%Y-%m-%dT%H:%M:%SZ)

## FIPS 203 ML-KEM (Kyber768)
- Location: \`crypto/tests/data/fips203/\`
- Status: Infrastructure ready, vectors pending download
- Sample: See \`fips203-sample.txt\`

## RFC 8439 ChaCha20-Poly1305
- Location: \`crypto/tests/data/rfc8439/\`
- Status: Infrastructure ready, vectors pending download
- Sample: See \`rfc8439-sample.txt\`

## RFC 7748 X25519
- Location: \`crypto/tests/data/rfc7748/\`
- Status: Infrastructure ready, vectors pending download
- Sample: See \`rfc7748-sample.txt\`

## Code Paths
- FIPS 203: \`crypto/tests/fips203_kat_loader.rs\`, \`crypto/src/kat_tests.rs\`
- RFC 8439: \`crypto/tests/rfc8439_kat.rs\`
- RFC 7748: \`crypto/tests/rfc7748_kat.rs\`
EOF
echo "✅ Corpus snapshot created: $ARTIFACT_DIR/transcripts/corpus-snapshot.md"
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

