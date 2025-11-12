#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Final production readiness verification script
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Production Readiness Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# 1. Compilation Check
echo "📦 Step 1: Compilation Check"
echo "────────────────────────────────────────────────────────────────────────────────"
if cargo check --workspace 2>&1 | tee /tmp/cargo-check.log; then
    echo "✅ All crates compile successfully"
else
    echo "❌ Compilation failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 2. Release Build
echo "🔨 Step 2: Release Build"
echo "────────────────────────────────────────────────────────────────────────────────"
if cargo build --release 2>&1 | tee /tmp/cargo-build.log; then
    echo "✅ Release build successful"
    BINARY="./target/release/cryprq"
    if [ -f "$BINARY" ]; then
        SIZE=$(stat -f%z "$BINARY" 2>/dev/null || stat -c%s "$BINARY" 2>/dev/null)
        echo "   Binary size: $SIZE bytes"
    fi
else
    echo "❌ Release build failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 3. Unit Tests
echo "🧪 Step 3: Unit Tests"
echo "────────────────────────────────────────────────────────────────────────────────"
if cargo test --lib --all --no-fail-fast 2>&1 | tee /tmp/cargo-test.log; then
    echo "✅ Unit tests passed"
else
    echo "❌ Unit tests failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 4. Documentation
echo "📚 Step 4: Documentation"
echo "────────────────────────────────────────────────────────────────────────────────"
if cargo doc --no-deps --all 2>&1 | tee /tmp/cargo-doc.log; then
    echo "✅ Documentation builds successfully"
else
    echo "❌ Documentation build failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 5. Security Audit
echo "🔒 Step 5: Security Audit"
echo "────────────────────────────────────────────────────────────────────────────────"
if bash scripts/security-audit.sh 2>&1 | tee /tmp/security-audit.log; then
    echo "✅ Security audit passed"
else
    echo "❌ Security audit failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 6. Compliance Checks
echo "✅ Step 6: Compliance Checks"
echo "────────────────────────────────────────────────────────────────────────────────"
if bash scripts/compliance-checks.sh 2>&1 | tee /tmp/compliance.log; then
    echo "✅ Compliance checks passed"
else
    echo "❌ Compliance checks failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 7. Code Formatting
echo "🎨 Step 7: Code Formatting"
echo "────────────────────────────────────────────────────────────────────────────────"
if cargo fmt --all -- --check 2>&1 | tee /tmp/cargo-fmt.log; then
    echo "✅ Code is properly formatted"
else
    echo "❌ Code formatting issues found"
    FAILED=$((FAILED + 1))
fi
echo ""

# 8. Clippy
echo "🔍 Step 8: Clippy Lints"
echo "────────────────────────────────────────────────────────────────────────────────"
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/cargo-clippy.log; then
    echo "✅ Clippy checks passed"
else
    echo "❌ Clippy checks failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# 9. Docker Build
echo "🐳 Step 9: Docker Build"
echo "────────────────────────────────────────────────────────────────────────────────"
if docker build -t cryprq-node:test -f Dockerfile . 2>&1 | tee /tmp/docker-build.log; then
    echo "✅ Docker build successful"
else
    echo "❌ Docker build failed"
    FAILED=$((FAILED + 1))
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
    echo "✅ Production Readiness: PASSED"
    echo ""
    echo "All checks completed successfully. The application is ready for production."
    exit 0
else
    echo "❌ Production Readiness: FAILED"
    echo ""
    echo "$FAILED check(s) failed. Please review the logs above and fix the issues."
    echo ""
    echo "Log files:"
    echo "  • /tmp/cargo-check.log"
    echo "  • /tmp/cargo-build.log"
    echo "  • /tmp/cargo-test.log"
    echo "  • /tmp/cargo-doc.log"
    echo "  • /tmp/security-audit.log"
    echo "  • /tmp/compliance.log"
    echo "  • /tmp/cargo-fmt.log"
    echo "  • /tmp/cargo-clippy.log"
    echo "  • /tmp/docker-build.log"
    exit 1
fi

