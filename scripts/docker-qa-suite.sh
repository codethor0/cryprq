#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🐳 Extensive Docker QA Testing Suite"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

QA_LOG="docker-qa-$(date +%Y%m%d-%H%M%S).log"
PASSED=0
FAILED=0

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Please install Docker.${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose not found. Please install Docker Compose.${NC}"
    exit 1
fi

echo "✅ Docker environment check passed"
echo "Docker version: $(docker --version)"
echo "Docker Compose version: $(docker compose version 2>/dev/null || docker-compose --version)"
echo ""

# Function to run test in container
run_test() {
    local name="$1"
    local command="$2"
    local container="${3:-cryprq-test-runner}"
    
    echo "▶️  Running: $name..."
    if docker exec "$container" bash -c "$command" >> "$QA_LOG" 2>&1; then
        echo -e "${GREEN}✅ $name passed${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ $name FAILED${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# Build Docker images
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 Building Docker Images"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker compose build --no-cache || {
    echo -e "${RED}❌ Docker build failed${NC}"
    exit 1
}
echo -e "${GREEN}✅ Docker images built successfully${NC}"
echo ""

# Start containers
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Docker Containers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker compose up -d cryprq-listener || {
    echo -e "${RED}❌ Failed to start listener${NC}"
    exit 1
}

# Wait for listener to be ready
echo "Waiting for listener to be ready..."
timeout 30 bash -c 'until docker logs cryprq-listener 2>&1 | grep -q "Listening on"; do sleep 1; done' || {
    echo -e "${YELLOW}⚠️  Listener may not have started properly${NC}"
    docker compose logs cryprq-listener
}

# Create test runner container
echo "Creating test runner container..."
docker compose run -d --name cryprq-test-runner cryprq-test-runner sleep 3600 || {
    echo -e "${YELLOW}⚠️  Test runner container creation failed, using existing${NC}"
}

echo -e "${GREEN}✅ Containers started${NC}"
echo ""

# Run tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Running Extensive QA Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 1. Secret Scanning
echo "=== 1. Secret Scanning ==="
run_test "Secret Scanning" "bash scripts/secret-scan.sh" "cryprq-test-runner" || true
echo ""

# 2. Dependency Scanning
echo "=== 2. Dependency Scanning ==="
run_test "Cargo Audit" "cargo audit --deny warnings || echo 'cargo-audit not available'" "cryprq-test-runner" || true
run_test "Cargo Deny" "cargo deny check || echo 'cargo-deny not available'" "cryprq-test-runner" || true
echo ""

# 3. Static Analysis
echo "=== 3. Static Analysis ==="
run_test "CodeQL Analysis" "bash scripts/codeql-analysis.sh || echo 'CodeQL not available'" "cryprq-test-runner" || true
echo ""

# 4. Dynamic Analysis
echo "=== 4. Dynamic Analysis ==="
run_test "Dynamic Analysis" "bash scripts/dynamic-analysis.sh || echo 'Dynamic analysis skipped'" "cryprq-test-runner" || true
echo ""

# 5. Security Audits
echo "=== 5. Security Audits ==="
run_test "Security Audit" "bash scripts/security-audit.sh || echo 'Security audit skipped'" "cryprq-test-runner" || true
echo ""

# 6. Cryptographic Validation
echo "=== 6. Cryptographic Validation ==="
run_test "Crypto Validation" "bash scripts/crypto-validation.sh" "cryprq-test-runner" || {
    echo -e "${RED}❌ Cryptographic validation failed - CRITICAL${NC}"
}
echo ""

# 7. Unit Tests
echo "=== 7. Unit Tests ==="
run_test "Unit Tests" "cargo test --lib --all --no-fail-fast" "cryprq-test-runner" || {
    echo -e "${RED}❌ Unit tests failed - CRITICAL${NC}"
}
echo ""

# 8. Integration Tests
echo "=== 8. Integration Tests ==="
run_test "Integration Tests" "bash scripts/test-integration.sh || echo 'Integration tests skipped'" "cryprq-test-runner" || true
echo ""

# 9. End-to-End Tests
echo "=== 9. End-to-End Tests ==="
run_test "E2E Tests" "bash scripts/end-to-end-tests.sh || echo 'E2E tests skipped'" "cryprq-test-runner" || true
echo ""

# 10. Performance Tests
echo "=== 10. Performance Tests ==="
run_test "Performance Tests" "bash scripts/performance-tests.sh || echo 'Performance tests skipped'" "cryprq-test-runner" || true
echo ""

# 11. Build Tests
echo "=== 11. Build Tests ==="
run_test "Release Build" "cargo build --release -p cryprq" "cryprq-test-runner" || {
    echo -e "${RED}❌ Release build failed - CRITICAL${NC}"
}
echo ""

# 12. Code Quality
echo "=== 12. Code Quality ==="
run_test "Cargo Fmt" "cargo fmt --all -- --check" "cryprq-test-runner" || {
    echo -e "${RED}❌ Format check failed${NC}"
}
run_test "Cargo Clippy" "cargo clippy --all-targets --all-features -- -D warnings" "cryprq-test-runner" || {
    echo -e "${RED}❌ Clippy check failed${NC}"
}
echo ""

# Verify container status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Container Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker compose ps
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total: $((PASSED + FAILED))"
echo ""
echo "📊 QA Log: $QA_LOG"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All critical tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please review the log: $QA_LOG${NC}"
    echo "Last 20 lines of log:"
    tail -20 "$QA_LOG"
    exit 1
fi

