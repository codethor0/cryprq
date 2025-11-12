#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# End-to-End test runner for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Running End-to-End Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "❌ ERROR: docker not found. Please install Docker."
    exit 1
fi

# Check for docker-compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ ERROR: docker-compose not found. Please install docker-compose."
    exit 1
fi

# Use docker compose (v2) if available, otherwise docker-compose (v1)
COMPOSE_CMD="docker compose"
if ! docker compose version &> /dev/null; then
    COMPOSE_CMD="docker-compose"
fi

# Clean up any existing containers
echo "🧹 Cleaning up existing containers..."
$COMPOSE_CMD down -v 2>/dev/null || true
docker rm -f cryprq-listener cryprq-dialer 2>/dev/null || true

# Build images
echo "🐳 Building Docker images..."
$COMPOSE_CMD build --no-cache > /dev/null 2>&1 || {
    echo "❌ Failed to build Docker images"
    exit 1
}

# Start services
echo "🚀 Starting services..."
$COMPOSE_CMD up -d cryprq-listener

# Wait for listener to be ready
echo "⏳ Waiting for listener to be ready..."
timeout=30
elapsed=0
while [ $elapsed -lt $timeout ]; do
    if docker logs cryprq-listener 2>&1 | grep -q "Listening on"; then
        echo "✅ Listener is ready"
        break
    fi
    sleep 1
    elapsed=$((elapsed + 1))
done

if [ $elapsed -eq $timeout ]; then
    echo "❌ Listener failed to start"
    docker logs cryprq-listener
    exit 1
fi

# Run dialer test
echo "🔗 Testing connection..."
$COMPOSE_CMD run --rm cryprq-dialer 2>&1 | tee test-e2e.log

# Check for successful connection
if grep -q "Connected\|Handshake completed" test-e2e.log; then
    echo ""
    echo "✅ End-to-end test passed!"
    echo ""
    echo "📊 Test Summary:"
    echo "  • Log file: test-e2e.log"
    echo "  • Listener logs: docker logs cryprq-listener"
    
    # Cleanup
    $COMPOSE_CMD down -v
    exit 0
else
    echo ""
    echo "❌ End-to-end test failed. Check test-e2e.log for details."
    docker logs cryprq-listener >> test-e2e.log
    $COMPOSE_CMD down -v
    exit 1
fi

