#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Integration test runner for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔗 Running Integration Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "❌ ERROR: docker not found. Please install Docker."
    exit 1
fi

# Build Docker image if needed
echo "🐳 Building Docker image..."
docker build -t cryprq-node:test -f Dockerfile . > /dev/null 2>&1 || {
    echo "❌ Failed to build Docker image"
    exit 1
}

# Run integration tests
echo "📦 Running integration tests..."
echo ""

# Run tests in tests/ directory if it exists
if [ -d "tests" ]; then
    cargo test --test '*' --no-fail-fast 2>&1 | tee test-integration.log
else
    echo "⚠️  No tests/ directory found. Running component integration tests..."
    cargo test --all --no-fail-fast 2>&1 | tee test-integration.log
fi

# Run Docker-based integration tests
echo ""
echo "🐳 Running Docker integration tests..."
if [ -f "scripts/docker_vpn_test.sh" ]; then
    bash scripts/docker_vpn_test.sh 2>&1 | tee -a test-integration.log
fi

# Check exit code
if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "✅ All integration tests passed!"
    echo ""
    echo "📊 Test Summary:"
    echo "  • Log file: test-integration.log"
    exit 0
else
    echo ""
    echo "❌ Some integration tests failed. Check test-integration.log for details."
    exit 1
fi

