#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Final Verification - Production Readiness"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

VERIFICATION_LOG="final-verification-$(date +%Y%m%d-%H%M%S).log"
PASSED=0
FAILED=0

run_check() {
    local name="$1"
    local command="$2"
    echo "▶️  $name..."
    if eval "$command" >> "$VERIFICATION_LOG" 2>&1; then
        echo "✅ $name passed"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo "❌ $name FAILED"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# 1. Build verification
echo "=== 1. Build Verification ==="
run_check "Release build" "cargo build --release -p cryprq"
run_check "All crates compile" "cargo check --workspace"
echo ""

# 2. Test verification
echo "=== 2. Test Verification ==="
run_check "Unit tests" "cargo test --lib --all --no-fail-fast"
run_check "Cryptographic tests" "bash scripts/crypto-validation.sh"
echo ""

# 3. Code quality
echo "=== 3. Code Quality ==="
run_check "Formatting" "cargo fmt --all -- --check"
run_check "Linting" "cargo clippy --all-targets --all-features -- -D warnings"
echo ""

# 4. Security
echo "=== 4. Security Verification ==="
run_check "Secret scanning" "bash scripts/secret-scan.sh" || true
run_check "Security audit" "bash scripts/security-audit.sh" || true
echo ""

# 5. Performance
echo "=== 5. Performance Verification ==="
BINARY_SIZE=$(stat -f%z target/release/cryprq 2>/dev/null || stat -c%s target/release/cryprq 2>/dev/null || echo "0")
if [ "$BINARY_SIZE" -lt 20000000 ]; then
    echo "✅ Binary size acceptable: $((BINARY_SIZE / 1024 / 1024))MB"
    PASSED=$((PASSED + 1))
else
    echo "⚠️  Binary size large: $((BINARY_SIZE / 1024 / 1024))MB"
    FAILED=$((FAILED + 1))
fi
echo ""

# 6. Documentation
echo "=== 6. Documentation Verification ==="
run_check "Documentation builds" "cargo doc --no-deps --workspace"
if [ -f "README.md" ] && [ -f "PRODUCTION_READY.md" ]; then
    echo "✅ Key documentation files present"
    PASSED=$((PASSED + 1))
else
    echo "❌ Missing documentation files"
    FAILED=$((FAILED + 1))
fi
echo ""

# 7. Docker (if available)
echo "=== 7. Docker Verification ==="
if command -v docker &> /dev/null; then
    if docker ps --filter "name=cryprq-listener" --format "{{.Names}}" | grep -q "cryprq-listener"; then
        echo "✅ Docker containers running"
        PASSED=$((PASSED + 1))
    else
        echo "⚠️  Docker containers not running (start with: docker compose up -d)"
        FAILED=$((FAILED + 1))
    fi
else
    echo "⚠️  Docker not available (skipping)"
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Final Verification Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total: $((PASSED + FAILED))"
echo ""
echo "📊 Verification log: $VERIFICATION_LOG"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All verifications passed!"
    echo "🚀 Application is ready for production!"
    exit 0
else
    echo "⚠️  Some verifications failed. Please review and fix."
    exit 1
fi

