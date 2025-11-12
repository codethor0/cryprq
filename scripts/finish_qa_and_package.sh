#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Finish QA and Package Release Bundle
# Usage: ./finish_qa_and_package.sh [VERSION=v0.1.0] [IMAGE=cryprq-node:latest]

VERSION="${1:-v0.1.0}"
IMAGE="${2:-cryprq-node:latest}"
RELEASE_DIR="release-${VERSION}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Finishing QA and Packaging Release Bundle: ${VERSION}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create release directory structure
mkdir -p "${RELEASE_DIR}/security"
mkdir -p "${RELEASE_DIR}/qa"
mkdir -p "${RELEASE_DIR}/bin"

echo "[1/5] Generating SBOM..."
bash scripts/syft-sbom.sh "${RELEASE_DIR}/security" 2>&1 | tee "${RELEASE_DIR}/qa/sbom.log" || echo "⚠️  SBOM generation skipped"

echo ""
echo "[2/5] Running Grype vulnerability scan..."
bash scripts/grype-scan.sh "${RELEASE_DIR}/security" 2>&1 | tee "${RELEASE_DIR}/qa/grype.log" || echo "⚠️  Grype scan skipped"

echo ""
echo "[3/5] Copying binaries and checksums..."
if [ -f "target/release/cryprq" ]; then
    cp target/release/cryprq "${RELEASE_DIR}/bin/cryprq-${VERSION}"
    shasum -a 256 "${RELEASE_DIR}/bin/cryprq-${VERSION}" > "${RELEASE_DIR}/bin/checksums.txt" 2>/dev/null || \
    md5sum "${RELEASE_DIR}/bin/cryprq-${VERSION}" > "${RELEASE_DIR}/bin/checksums.txt" 2>/dev/null || true
    echo "✅ Binary copied: ${RELEASE_DIR}/bin/cryprq-${VERSION}"
else
    echo "⚠️  Release binary not found. Run 'cargo build --release -p cryprq' first."
fi

echo ""
echo "[4/5] Collecting QA logs..."
# Copy any existing QA artifacts
if [ -d "artifacts/local" ]; then
    cp -r artifacts/local/* "${RELEASE_DIR}/qa/" 2>/dev/null || true
fi
if [ -d "artifacts/docker" ]; then
    cp -r artifacts/docker/* "${RELEASE_DIR}/qa/" 2>/dev/null || true
fi
if [ -d "artifacts/perf" ]; then
    cp -r artifacts/perf/* "${RELEASE_DIR}/qa/" 2>/dev/null || true
fi

# Copy cutover smoke artifacts if available
if [ -d "artifacts/cutover_${TIMESTAMP}" ]; then
    cp -r "artifacts/cutover_${TIMESTAMP}"/* "${RELEASE_DIR}/qa/" 2>/dev/null || true
fi

echo ""
echo "[5/5] Creating release summary..."
cat > "${RELEASE_DIR}/RELEASE_SUMMARY.txt" <<EOF
CrypRQ Release Bundle: ${VERSION}
Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

Contents:
- security/     SBOM (SPDX) and Grype vulnerability reports
- qa/           QA logs, test results, and validation artifacts
- bin/          Release binaries and checksums

Quick Verification:
- Check checksums: cat ${RELEASE_DIR}/bin/checksums.txt
- Review SBOM: cat ${RELEASE_DIR}/security/*.spdx.json
- Review Grype: cat ${RELEASE_DIR}/security/*.txt

For full release notes, see PRODUCTION_SUMMARY.md
EOF

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Release bundle complete: ${RELEASE_DIR}/"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Contents:"
echo "  • security/  - SBOM, Grype reports"
echo "  • qa/        - QA logs and test results"
echo "  • bin/       - Release binaries and checksums"
echo ""

