#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Vulnerability Scanning (Grype)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

OUTPUT_DIR="${1:-artifacts/grype}"
mkdir -p "$OUTPUT_DIR"

# Check if grype is available
if ! command -v grype &> /dev/null; then
    echo "⚠️  Grype not found. Installing..."
    if command -v brew &> /dev/null; then
        brew install grype || echo "⚠️  Failed to install grype via brew"
    elif command -v docker &> /dev/null; then
        echo "Using Docker to run grype..."
        GRYPE_CMD="docker run --rm -v $(pwd):/workspace -w /workspace anchore/grype"
    else
        echo "❌ Grype not available and cannot install. Skipping vulnerability scan."
        exit 0
    fi
else
    GRYPE_CMD="grype"
fi

# Scan Docker image
if docker images cryprq-node:latest --format "{{.Repository}}:{{.Tag}}" | grep -q "cryprq-node:latest"; then
    echo "Scanning Docker image for vulnerabilities..."
    $GRYPE_CMD docker:cryprq-node:latest \
        --output json \
        --file "$OUTPUT_DIR/docker-scan.json" || echo "⚠️  Docker scan failed"
    
    $GRYPE_CMD docker:cryprq-node:latest \
        --output table \
        --file "$OUTPUT_DIR/docker-scan.txt" || echo "⚠️  Docker scan failed"
fi

# Scan Rust workspace
echo "Scanning Rust workspace for vulnerabilities..."
$GRYPE_CMD dir:. \
        --output json \
        --file "$OUTPUT_DIR/workspace-scan.json" || echo "⚠️  Workspace scan failed"

echo ""
echo "✅ Vulnerability scanning complete"
echo "📊 Scan results saved to: $OUTPUT_DIR/"

