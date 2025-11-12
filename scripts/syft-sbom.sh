#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SBOM Generation (Syft)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

OUTPUT_DIR="${1:-artifacts/sbom}"
mkdir -p "$OUTPUT_DIR"

# Check if syft is available
if ! command -v syft &> /dev/null; then
    echo "⚠️  Syft not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install syft || echo "⚠️  Failed to install syft via brew"
    elif command -v docker &> /dev/null; then
        echo "Using Docker to run syft..."
        SYFT_CMD="docker run --rm -v $(pwd):/workspace -w /workspace anchore/syft"
    else
        echo "❌ Syft not available and cannot install. Skipping SBOM generation."
        exit 0
    fi
else
    SYFT_CMD="syft"
fi

# Generate SBOM for binary
if [ -f "target/release/cryprq" ]; then
    echo "Generating SBOM for release binary..."
    $SYFT_CMD packages dir:target/release/cryprq \
        --output spdx-json \
        --file "$OUTPUT_DIR/binary-sbom.spdx.json" || echo "⚠️  Binary SBOM generation failed"
fi

# Generate SBOM for Docker image
if docker images cryprq-node:latest --format "{{.Repository}}:{{.Tag}}" | grep -q "cryprq-node:latest"; then
    echo "Generating SBOM for Docker image..."
    $SYFT_CMD packages docker:cryprq-node:latest \
        --output spdx-json \
        --file "$OUTPUT_DIR/docker-sbom.spdx.json" || echo "⚠️  Docker SBOM generation failed"
fi

# Generate SBOM for Rust workspace
echo "Generating SBOM for Rust workspace..."
$SYFT_CMD packages dir:. \
        --output spdx-json \
        --file "$OUTPUT_DIR/workspace-sbom.spdx.json" || echo "⚠️  Workspace SBOM generation failed"

echo ""
echo "✅ SBOM generation complete"
echo "📊 SBOMs saved to: $OUTPUT_DIR/"

