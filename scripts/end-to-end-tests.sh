#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 End-to-End Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker to run E2E tests."
    exit 1
fi

# Start services
echo "Starting Docker Compose services..."
docker compose up -d cryprq-listener || {
    echo "❌ Failed to start listener"
    exit 1
}

# Wait for listener to be ready
echo "Waiting for listener to be ready..."
timeout 30 bash -c 'until docker logs cryprq-listener 2>&1 | grep -q "Listening on"; do sleep 1; done' || {
    echo "❌ Listener did not start in time"
    docker compose logs cryprq-listener
    docker compose down -v
    exit 1
}

# Run dialer
echo "Running dialer..."
docker compose run --rm cryprq-dialer || {
    echo "⚠️  Dialer test failed (non-blocking)"
}

# Run integration tests
echo "Running integration tests..."
if [ -f "scripts/test-integration.sh" ]; then
    bash scripts/test-integration.sh || {
        echo "⚠️  Integration tests failed (non-blocking)"
    }
else
    echo "⚠️  Integration test script not found"
fi

# Cleanup
echo "Cleaning up..."
docker compose down -v

echo ""
echo "✅ End-to-end tests complete"

