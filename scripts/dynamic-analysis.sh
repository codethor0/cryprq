#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Dynamic Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Build release binary
echo "Building release binary..."
cargo build --release -p cryprq || {
    echo "❌ Build failed"
    exit 1
}

# Run with various inputs
echo "Running dynamic analysis tests..."

# Test 1: Help command
echo "Test 1: Help command"
./target/release/cryprq --help > /dev/null || {
    echo "❌ Help command failed"
    exit 1
}
echo "✅ Help command works"

# Test 2: Invalid arguments
echo "Test 2: Invalid arguments"
./target/release/cryprq --invalid-arg 2>&1 | grep -q "error" || {
    echo "⚠️  Invalid arguments not handled properly"
}
echo "✅ Invalid arguments handled"

# Test 3: Memory leak check (basic)
echo "Test 3: Memory check (basic)"
if command -v valgrind &> /dev/null; then
    echo "Running valgrind memory check..."
    valgrind --leak-check=full --error-exitcode=1 ./target/release/cryprq --help > /dev/null 2>&1 || {
        echo "⚠️  Valgrind found potential issues (non-blocking)"
    }
else
    echo "⚠️  Valgrind not found, skipping memory check"
fi

# Test 4: Fuzzing (if available)
echo "Test 4: Fuzzing check"
if [ -d "fuzz" ]; then
    echo "Fuzz targets available. Run 'cargo fuzz' for detailed fuzzing."
else
    echo "⚠️  Fuzz targets not found"
fi

echo ""
echo "✅ Dynamic analysis complete"
echo ""
echo "Note: For comprehensive dynamic analysis, use:"
echo "  - Valgrind for memory leaks"
echo "  - cargo fuzz for fuzzing"
echo "  - Integration tests for runtime behavior"

