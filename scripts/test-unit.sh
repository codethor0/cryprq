#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Unit test runner for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Running Unit Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Run unit tests for all packages
echo "📦 Running unit tests for all packages..."
echo ""

cargo test --lib --all --no-fail-fast 2>&1 | tee test-unit.log

# Check exit code
if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "✅ All unit tests passed!"
    echo ""
    echo "📊 Test Summary:"
    echo "  • Log file: test-unit.log"
    exit 0
else
    echo ""
    echo "❌ Some unit tests failed. Check test-unit.log for details."
    exit 1
fi

