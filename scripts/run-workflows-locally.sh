#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Running GitHub Workflows Locally"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

run_check() {
    local name="$1"
    local command="$2"
    echo "▶️  Running: $name..."
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $name passed${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ $name FAILED${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# CI Workflow Checks
echo "=== CI Workflow Checks ==="
run_check "Cargo fmt check" "cargo fmt --all -- --check"
run_check "Cargo clippy" "cargo clippy --all-targets --all-features -- -D warnings"
run_check "Cargo test (lib)" "cargo test --lib --all --no-fail-fast"
run_check "Cargo build (release)" "cargo build --release -p cryprq"
echo ""

# Docker Workflow Checks
echo "=== Docker Workflow Checks ==="
if command -v docker &> /dev/null; then
    run_check "Docker build" "docker build -t cryprq-test -f Dockerfile ."
    if [ $? -eq 0 ]; then
        run_check "Docker run (help)" "docker run --rm cryprq-test --help"
    fi
else
    echo -e "${YELLOW}⚠️  Docker not found, skipping Docker checks${NC}"
fi
echo ""

# Security Audit Checks
echo "=== Security Audit Checks ==="
if command -v cargo-audit &> /dev/null; then
    run_check "Cargo audit" "cargo audit --deny warnings" || true
else
    echo -e "${YELLOW}⚠️  cargo-audit not found, skipping${NC}"
fi

if command -v cargo-deny &> /dev/null; then
    run_check "Cargo deny" "cargo deny check" || true
else
    echo -e "${YELLOW}⚠️  cargo-deny not found, skipping${NC}"
fi
echo ""

# CodeQL Checks (simulated)
echo "=== CodeQL Checks (simulated) ==="
run_check "Cargo check (all targets)" "cargo check --all-targets --all-features"
run_check "Cargo doc" "cargo doc --no-deps --workspace"
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! ($PASSED/$PASSED)${NC}"
    echo "🚀 Workflows should pass on GitHub!"
    exit 0
else
    echo -e "${RED}❌ Some checks failed ($FAILED failed, $PASSED passed)${NC}"
    echo "⚠️  Please fix the issues above before pushing to GitHub"
    exit 1
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

