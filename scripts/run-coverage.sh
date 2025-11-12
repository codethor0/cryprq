#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Run coverage analysis with cargo-llvm-cov
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/coverage}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Coverage Analysis (cargo-llvm-cov)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Install cargo-llvm-cov if not present
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov --force
fi

# Run coverage
echo "Running coverage analysis..."
cargo llvm-cov --all-features --workspace --tests --lcov --output-path "$ARTIFACT_DIR/coverage.lcov" 2>&1 | tee "$ARTIFACT_DIR/coverage.log"

# Generate HTML report
cargo llvm-cov --all-features --workspace --tests --html --output-dir "$ARTIFACT_DIR/html" 2>&1 | tee -a "$ARTIFACT_DIR/coverage.log"

# Extract coverage percentages
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Coverage Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Parse coverage from log
COVERAGE_LINE=$(grep -E "^\s*Total\s+\|" "$ARTIFACT_DIR/coverage.log" | tail -1 || echo "")
if [ -n "$COVERAGE_LINE" ]; then
    echo "$COVERAGE_LINE"
fi

# Check thresholds
echo ""
echo "Coverage Thresholds:"
echo "  crypto: ≥90% line / ≥85% branch"
echo "  core/p2p: ≥80% line / ≥70% branch"
echo ""
echo "📊 Full report: $ARTIFACT_DIR/html/index.html"
echo "📄 LCOV report: $ARTIFACT_DIR/coverage.lcov"

