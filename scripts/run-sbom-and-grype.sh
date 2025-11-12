#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# SBOM generation and Grype scanning
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/sbom}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SBOM Generation & Grype Scan"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# Install Syft if not present
if ! command -v syft >/dev/null 2>&1; then
    echo "Installing Syft..."
    curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin || {
        echo "⚠️ Syft installation failed - skipping SBOM generation"
        FAILED=1
    }
fi

# Generate SBOMs
if command -v syft >/dev/null 2>&1; then
    echo "Generating SBOM (SPDX JSON)..."
    syft packages dir:. -o spdx-json > "$ARTIFACT_DIR/sbom.spdx.json" 2>&1 || {
        echo "❌ SBOM generation failed"
        FAILED=1
    }
    
    echo "Generating SBOM (CycloneDX JSON)..."
    syft packages dir:. -o cyclonedx-json > "$ARTIFACT_DIR/sbom.cyclonedx.json" 2>&1 || {
        echo "❌ SBOM generation failed"
        FAILED=1
    }
    
    echo "✅ SBOMs generated"
else
    echo "⚠️ Syft not available - skipping SBOM generation"
fi

echo ""

# Install Grype if not present
if ! command -v grype >/dev/null 2>&1; then
    echo "Installing Grype..."
    curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin || {
        echo "⚠️ Grype installation failed - skipping vulnerability scan"
        FAILED=1
    }
fi

# Run Grype scan
if command -v grype >/dev/null 2>&1; then
    echo "Running Grype vulnerability scan..."
    if grype dir:. -o json > "$ARTIFACT_DIR/grype.json" 2>&1; then
        # Check for High/Critical vulnerabilities
        HIGH_CRITICAL=$(grep -E '"severity":\s*"(HIGH|CRITICAL)"' "$ARTIFACT_DIR/grype.json" 2>/dev/null | wc -l | tr -d ' ' || echo "0")
        
        if [ "$HIGH_CRITICAL" -gt 0 ]; then
            echo "❌ Grype found $HIGH_CRITICAL HIGH/CRITICAL vulnerabilities"
            FAILED=1
        else
            echo "✅ Grype scan passed (no HIGH/CRITICAL vulnerabilities)"
        fi
        
        # Generate text report
        grype dir:. -o table > "$ARTIFACT_DIR/grype.txt" 2>&1 || true
    else
        echo "❌ Grype scan failed"
        FAILED=1
    fi
else
    echo "⚠️ Grype not available - skipping vulnerability scan"
fi

echo ""

if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ SBOM and Grype scan complete"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ SBOM/Grype checks failed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

