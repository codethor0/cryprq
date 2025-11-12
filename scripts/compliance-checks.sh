#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Compliance check script for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Running Compliance Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

COMPLIANCE_LOG="compliance-$(date +%Y%m%d-%H%M%S).log"
ISSUES=0

# Check for required files
echo "📋 Checking required files..."

REQUIRED_FILES=(
    "LICENSE"
    "README.md"
    "SECURITY.md"
    "CONTRIBUTING.md"
    "Cargo.toml"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file exists"
    else
        echo "  ❌ $file missing!"
        ISSUES=$((ISSUES + 1))
    fi
done | tee -a "$COMPLIANCE_LOG"

# Check license headers
echo ""
echo "📜 Checking license headers..."
if [ -f "scripts/add-headers.sh" ]; then
    echo "  ✅ License header script exists"
else
    echo "  ⚠️  License header script not found"
fi

# Check for SPDX license identifier
if grep -r "SPDX-License-Identifier" --include="*.rs" --include="*.toml" . > /dev/null; then
    echo "  ✅ SPDX license identifiers found"
else
    echo "  ⚠️  SPDX license identifiers not found in all files"
    ISSUES=$((ISSUES + 1))
fi

# Check Rust edition
echo ""
echo "🦀 Checking Rust edition..."
if grep -q 'edition = "2021"' Cargo.toml; then
    echo "  ✅ Using Rust 2021 edition"
else
    echo "  ⚠️  Not using Rust 2021 edition"
    ISSUES=$((ISSUES + 1))
fi

# Check for unsafe code (should be minimal)
echo ""
echo "🔒 Checking unsafe code usage..."
UNSAFE_COUNT=$(grep -r "unsafe" --include="*.rs" . | grep -v "//" | grep -v "test" | wc -l || echo "0")
if [ "$UNSAFE_COUNT" -eq 0 ]; then
    echo "  ✅ No unsafe code found"
else
    echo "  ⚠️  Found $UNSAFE_COUNT instances of unsafe code"
fi

# Check code formatting
echo ""
echo "🎨 Checking code formatting..."
if cargo fmt --all -- --check 2>&1 | tee -a "$COMPLIANCE_LOG"; then
    echo "  ✅ Code is properly formatted"
else
    echo "  ❌ Code formatting issues found"
    ISSUES=$((ISSUES + 1))
fi

# Check clippy
echo ""
echo "🔍 Running clippy checks..."
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee -a "$COMPLIANCE_LOG"; then
    echo "  ✅ Clippy checks passed"
else
    echo "  ❌ Clippy checks failed"
    ISSUES=$((ISSUES + 1))
fi

# Check documentation
echo ""
echo "📚 Checking documentation..."
if cargo doc --no-deps --all 2>&1 | tee -a "$COMPLIANCE_LOG"; then
    echo "  ✅ Documentation builds successfully"
else
    echo "  ⚠️  Documentation build issues"
    ISSUES=$((ISSUES + 1))
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ISSUES -eq 0 ]; then
    echo "✅ Compliance checks passed"
    echo "📊 Compliance log: $COMPLIANCE_LOG"
    exit 0
else
    echo "❌ Compliance checks found $ISSUES issue(s)"
    echo "📊 Compliance log: $COMPLIANCE_LOG"
    echo "⚠️  Please review and fix the issues above"
    exit 1
fi

