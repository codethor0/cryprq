#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Documentation linting (rustdoc + clippy doc lints)
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/docs}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Documentation Linting"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# Check for missing docs
echo "Checking for missing documentation..."
if cargo clippy --all-targets --all-features -- -D missing_docs 2>&1 | tee "$ARTIFACT_DIR/missing-docs.log"; then
    echo "✅ No missing documentation"
else
    echo "❌ Missing documentation found"
    FAILED=1
fi

# Check for missing examples
echo ""
echo "Checking for missing examples..."
if cargo clippy --all-targets --all-features -- -D missing_doc_examples 2>&1 | tee "$ARTIFACT_DIR/missing-examples.log"; then
    echo "✅ No missing examples"
else
    echo "⚠️ Missing examples found (non-blocking)"
fi

# Build docs to check for doc errors
echo ""
echo "Building documentation..."
if cargo doc --no-deps --all-features 2>&1 | tee "$ARTIFACT_DIR/doc-build.log"; then
    echo "✅ Documentation builds successfully"
else
    echo "❌ Documentation build failed"
    FAILED=1
fi

# Optional: cargo-spellcheck
if command -v cargo-spellcheck >/dev/null 2>&1; then
    echo ""
    echo "Running cargo-spellcheck..."
    cargo spellcheck 2>&1 | tee "$ARTIFACT_DIR/spellcheck.log" || {
        echo "⚠️ Spellcheck found issues (non-blocking)"
    }
fi

echo ""
if [ $FAILED -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ Documentation linting passed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ Documentation linting failed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi

