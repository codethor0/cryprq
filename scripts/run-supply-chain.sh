#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Supply chain security checks
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/supply-chain}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Supply Chain Security Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# cargo-audit
echo "Running cargo-audit..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit 2>&1 | tee "$ARTIFACT_DIR/audit.log"; then
        echo "✅ cargo-audit passed"
    else
        echo "❌ cargo-audit found vulnerabilities"
        FAILED=1
    fi
else
    echo "⚠️ cargo-audit not installed - skipping"
    echo "Install with: cargo install cargo-audit --locked"
fi
echo ""

# cargo-deny
echo "Running cargo-deny..."
if command -v cargo-deny &> /dev/null; then
    if cargo deny check 2>&1 | tee "$ARTIFACT_DIR/deny.log"; then
        echo "✅ cargo-deny passed"
    else
        echo "❌ cargo-deny found issues"
        FAILED=1
    fi
else
    echo "⚠️ cargo-deny not installed - skipping"
    echo "Install with: cargo install cargo-deny --locked"
fi
echo ""

# cargo-vet (if configured)
if [ -f "supply-chain/audits.toml" ]; then
    echo "Running cargo-vet..."
    if cargo vet check 2>&1 | tee "$ARTIFACT_DIR/vet.log"; then
        echo "✅ cargo-vet passed"
    else
        echo "❌ cargo-vet found unaudited dependencies"
        FAILED=1
    fi
    echo ""
else
    echo "⚠️ cargo-vet not configured (supply-chain/audits.toml missing)"
    echo "Skipping cargo-vet check"
    echo ""
fi

# cargo-geiger (unsafe code audit)
echo "Running cargo-geiger..."
if command -v cargo-geiger &> /dev/null; then
    if cargo geiger --quiet 2>&1 | tee "$ARTIFACT_DIR/geiger.log"; then
        echo "✅ cargo-geiger completed"
    else
        echo "⚠️ cargo-geiger encountered issues (non-blocking)"
    fi
else
    echo "⚠️ cargo-geiger not installed - skipping"
    echo "Install with: cargo install cargo-geiger --locked"
fi
echo ""

# SBOM generation
echo "Generating SBOM with Syft..."
if command -v syft >/dev/null 2>&1; then
    syft packages dir:. -o spdx-json > "$ARTIFACT_DIR/sbom.spdx.json" 2>&1
    syft packages dir:. -o cyclonedx-json > "$ARTIFACT_DIR/sbom.cyclonedx.json" 2>&1
    echo "✅ SBOM generated"
else
    echo "⚠️ Syft not installed - skipping SBOM generation"
fi
echo ""

# Grype scan
echo "Running Grype vulnerability scan..."
if command -v grype >/dev/null 2>&1; then
    if grype dir:. -o json > "$ARTIFACT_DIR/grype.json" 2>&1; then
        # Check for high/critical vulnerabilities
        HIGH_CRITICAL=$(grep -E '"severity":\s*"(HIGH|CRITICAL)"' "$ARTIFACT_DIR/grype.json" | wc -l | tr -d ' ')
        if [ "$HIGH_CRITICAL" -gt 0 ]; then
            echo "❌ Grype found $HIGH_CRITICAL HIGH/CRITICAL vulnerabilities"
            FAILED=1
        else
            echo "✅ Grype scan passed (no HIGH/CRITICAL vulnerabilities)"
        fi
    else
        echo "❌ Grype scan failed"
        FAILED=1
    fi
else
    echo "⚠️ Grype not installed - skipping vulnerability scan"
fi
echo ""

if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ All supply chain checks passed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ Supply chain checks failed - check logs in $ARTIFACT_DIR"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

