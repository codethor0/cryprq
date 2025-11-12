#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Security audit script for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔒 Running Security Audit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

AUDIT_LOG="security-audit-$(date +%Y%m%d-%H%M%S).log"
ISSUES=0

# Check for cargo-audit
if ! command -v cargo-audit &> /dev/null; then
    echo "⚠️  cargo-audit not found. Installing..."
    cargo install cargo-audit --locked || {
        echo "❌ Failed to install cargo-audit"
        exit 1
    }
fi

# Run cargo audit
echo "📦 Running cargo-audit..."
if cargo audit --deny warnings 2>&1 | tee -a "$AUDIT_LOG"; then
    echo "✅ No known security vulnerabilities found"
else
    echo "❌ Security vulnerabilities detected!"
    ISSUES=$((ISSUES + 1))
fi

# Check for cargo-deny
if command -v cargo-deny &> /dev/null; then
    echo ""
    echo "📋 Running cargo-deny..."
    if cargo deny check 2>&1 | tee -a "$AUDIT_LOG"; then
        echo "✅ Cargo-deny checks passed"
    else
        echo "❌ Cargo-deny checks failed!"
        ISSUES=$((ISSUES + 1))
    fi
else
    echo "⚠️  cargo-deny not found. Skipping dependency checks."
fi

# Check for unsafe code
echo ""
echo "🔍 Checking for unsafe code..."
UNSAFE_COUNT=$(grep -r "unsafe" --include="*.rs" . | grep -v "//" | wc -l || echo "0")
if [ "$UNSAFE_COUNT" -gt 0 ]; then
    echo "⚠️  Found $UNSAFE_COUNT instances of 'unsafe' keyword"
    grep -r "unsafe" --include="*.rs" . | grep -v "//" | tee -a "$AUDIT_LOG"
else
    echo "✅ No unsafe code found"
fi

# Check for hardcoded secrets
echo ""
echo "🔍 Checking for potential hardcoded secrets..."
SECRET_PATTERNS=("password" "secret" "api_key" "private_key" "token")
for pattern in "${SECRET_PATTERNS[@]}"; do
    if grep -ri "$pattern" --include="*.rs" --include="*.toml" . | grep -v "test" | grep -v "example" | grep -v "//" > /dev/null; then
        echo "⚠️  Potential hardcoded $pattern found"
        grep -ri "$pattern" --include="*.rs" --include="*.toml" . | grep -v "test" | grep -v "example" | grep -v "//" | tee -a "$AUDIT_LOG"
        ISSUES=$((ISSUES + 1))
    fi
done

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ISSUES -eq 0 ]; then
    echo "✅ Security audit completed successfully"
    echo "📊 Audit log: $AUDIT_LOG"
    exit 0
else
    echo "❌ Security audit found $ISSUES issue(s)"
    echo "📊 Audit log: $AUDIT_LOG"
    echo "⚠️  Please review and fix the issues above"
    exit 1
fi

