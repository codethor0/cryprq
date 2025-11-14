#!/bin/bash

# Copyright (c) 2025 Thor Thor
# Author: Thor Thor (GitHub: https://github.com/codethor0)
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# License: MIT (see LICENSE file for details)

# Web Smoke Test Helper Script
# This script helps automate the web smoke test process
# Usage: ./scripts/web-smoke-test.sh

set -euo pipefail

echo "=== CrypRQ Web Smoke Test Helper ==="
echo ""

# Check if test file exists
TEST_FILE="/tmp/testfile.bin"
EXPECTED_HASH="6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec"

if [ ! -f "$TEST_FILE" ]; then
    echo "⚠️  Test file not found: $TEST_FILE"
    echo "Creating test file..."
    echo "Test file for CrypRQ web v1.0.1" > "$TEST_FILE"
    echo "✅ Created $TEST_FILE"
fi

# Verify test file hash
ACTUAL_HASH=$(sha256sum "$TEST_FILE" | cut -d' ' -f1)
echo "Test file hash: $ACTUAL_HASH"

if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
    echo "⚠️  Warning: Test file hash doesn't match expected hash"
    echo "   Expected: $EXPECTED_HASH"
    echo "   Actual:   $ACTUAL_HASH"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted. Please create /tmp/testfile.bin with correct content."
        exit 1
    fi
else
    echo "✅ Test file hash matches expected"
fi

echo ""
echo "=== Starting Web Stack ==="
echo ""
echo "Starting Docker Compose..."
echo "Command: docker compose -f docker-compose.web.yml up --build"
echo ""
echo "⚠️  Manual steps required:"
echo "1. Wait for web stack to start (look for 'listening on port 8787' or similar)"
echo "2. Open browser: http://localhost:8787"
echo "3. Use Web UI to send: $TEST_FILE"
echo "4. Wait for transfer to complete"
echo ""

read -p "Press Enter when web stack is running and you're ready to test, or Ctrl+C to exit..."
echo ""

# Start docker compose in background
echo "Starting Docker Compose..."
docker compose -f docker-compose.web.yml up --build -d

echo ""
echo "✅ Web stack started in background"
echo ""
echo "Next steps:"
echo "1. Open browser: http://localhost:8787"
echo "2. Send test file via UI: $TEST_FILE"
echo "3. After transfer completes, run:"
echo ""
echo "   sha256sum $TEST_FILE /tmp/receive/testfile.bin"
echo ""
echo "   Expected hash: $EXPECTED_HASH"
echo ""
echo "4. If hashes match, run:"
echo ""
echo "   ./scripts/update-web-validation.sh \\"
echo "     WEB-1 PASS \"\$(date +%Y-%m-%d)\" \\"
echo "     \"testfile.bin\" \\"
echo "     \"$EXPECTED_HASH\" \\"
echo "     \"matches CLI minimal sanity\""
echo ""
echo "5. Then run: ./scripts/complete-release.sh"
echo ""
echo "To stop web stack: docker compose -f docker-compose.web.yml down"
echo ""

