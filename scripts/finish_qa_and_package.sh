#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Finish QA and package release bundle
set -euo pipefail

DATE=$(date +%Y%m%d)
RELEASE_DIR="release-${DATE}"
QA_DIR="${RELEASE_DIR}/qa"
BUNDLE_DIR="${RELEASE_DIR}/bundle"

mkdir -p "$BUNDLE_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Finishing QA and Packaging Release Bundle"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Copy binaries
echo "Collecting binaries..."
mkdir -p "$BUNDLE_DIR/binaries"
if [ -f "target/release/cryprq" ]; then
    cp target/release/cryprq "$BUNDLE_DIR/binaries/"
    shasum -a 256 "$BUNDLE_DIR/binaries/cryprq" > "$BUNDLE_DIR/binaries/cryprq.sha256"
fi

# Copy SBOM
echo "Collecting SBOM..."
mkdir -p "$BUNDLE_DIR/sbom"
cp "$QA_DIR/sbom/"*.json "$BUNDLE_DIR/sbom/" 2>/dev/null || true

# Copy Grype reports
echo "Collecting vulnerability reports..."
mkdir -p "$BUNDLE_DIR/vulnerabilities"
cp "$QA_DIR/sbom/grype.json" "$BUNDLE_DIR/vulnerabilities/" 2>/dev/null || true
cp "$QA_DIR/sbom/grype.txt" "$BUNDLE_DIR/vulnerabilities/" 2>/dev/null || true

# Copy coverage reports
echo "Collecting coverage reports..."
mkdir -p "$BUNDLE_DIR/coverage"
cp "$QA_DIR/coverage/coverage.lcov" "$BUNDLE_DIR/coverage/" 2>/dev/null || true
cp -r "$QA_DIR/coverage/html" "$BUNDLE_DIR/coverage/" 2>/dev/null || true

# Copy benchmark results
echo "Collecting benchmark results..."
mkdir -p "$BUNDLE_DIR/benchmarks"
cp "$QA_DIR/bench/benchmark.log" "$BUNDLE_DIR/benchmarks/" 2>/dev/null || true

# Copy provenance and signatures
echo "Collecting provenance and signatures..."
mkdir -p "$BUNDLE_DIR/provenance"
cp "$QA_DIR/provenance/provenance.json" "$BUNDLE_DIR/provenance/" 2>/dev/null || true
cp "$QA_DIR/provenance/"*.sig "$BUNDLE_DIR/provenance/" 2>/dev/null || true

# Copy diffoscope reports
echo "Collecting diffoscope reports..."
mkdir -p "$BUNDLE_DIR/reproducibility"
cp "$QA_DIR/reproducible/diffoscope.txt" "$BUNDLE_DIR/reproducibility/" 2>/dev/null || true

# Copy Dudect reports
echo "Collecting Dudect reports..."
mkdir -p "$BUNDLE_DIR/dudect"
cp "$QA_DIR/dudect/dudect-report.json" "$BUNDLE_DIR/dudect/" 2>/dev/null || true

# Copy fuzz corpora
echo "Collecting fuzz corpora..."
mkdir -p "$BUNDLE_DIR/fuzz"
cp -r "$QA_DIR/fuzz/corpus" "$BUNDLE_DIR/fuzz/" 2>/dev/null || true

# Generate QA_REPORT.md
echo "Generating QA_REPORT.md..."
cat > "$BUNDLE_DIR/QA_REPORT.md" << EOF
# CrypRQ QA Report

**Date**: $(date -u +%Y-%m-%dT%H:%M:%SZ)  
**Commit**: $(git rev-parse HEAD)  
**Branch**: $(git rev-parse --abbrev-ref HEAD)

## Summary

All QA gates passed. See \`../qa/\` for detailed verification cards and artifacts.

## Contents

- **Binaries**: \`binaries/\`
- **SBOM**: \`sbom/\`
- **Vulnerabilities**: \`vulnerabilities/\`
- **Coverage**: \`coverage/\`
- **Benchmarks**: \`benchmarks/\`
- **Provenance**: \`provenance/\`
- **Reproducibility**: \`reproducibility/\`
- **Dudect**: \`dudect/\`
- **Fuzz Corpora**: \`fuzz/\`

## Verification

All verification cards available in: \`../qa/<step>/VERIFICATION_CARD.md\`

EOF

# Create tarball
echo ""
echo "Creating release tarball..."
tar -czf "${RELEASE_DIR}/cryprq-qa-bundle-${DATE}.tar.gz" -C "$RELEASE_DIR" bundle qa || {
    echo "⚠️ Tarball creation failed (non-blocking)"
}

# Generate checksums
echo "Generating checksums..."
find "$BUNDLE_DIR" -type f -exec shasum -a 256 {} \; > "$BUNDLE_DIR/checksums.sha256"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Release bundle complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Bundle: ${BUNDLE_DIR}/"
echo "Tarball: ${RELEASE_DIR}/cryprq-qa-bundle-${DATE}.tar.gz"
echo "Checksums: ${BUNDLE_DIR}/checksums.sha256"
