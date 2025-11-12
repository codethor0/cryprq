#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Test web server with Docker mode

set -euo pipefail

cd "$(dirname "$0")/.."

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "🧪 Testing Web Server with Docker Mode..."

# Check Docker
if ! docker ps > /dev/null 2>&1; then
    log "❌ Docker is not running"
    exit 1
fi

# Check container
if ! docker ps | grep -q cryprq-listener; then
    log "⚠️  Container not running, starting it..."
    ./scripts/docker-vpn-start.sh
    sleep 3
fi

CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-listener 2>/dev/null || echo "")
if [ -z "$CONTAINER_IP" ]; then
    log "❌ Could not get container IP"
    exit 1
fi

log "✅ Container IP: $CONTAINER_IP"

# Start web server
log "🌐 Starting web server with Docker mode..."
export USE_DOCKER=true
export BRIDGE_PORT=8787

cd web/server
node server.mjs &
WEB_PID=$!

sleep 2

log "✅ Web server started (PID: $WEB_PID)"
log ""
log "Testing endpoints..."

# Test listener endpoint
log "Testing listener endpoint..."
LISTENER_RESPONSE=$(curl -s -X POST http://localhost:8787/connect \
    -H "Content-Type: application/json" \
    -d '{"mode":"listener","port":9999,"vpn":false}')

if echo "$LISTENER_RESPONSE" | grep -q "dockerMode"; then
    log "✅ Listener endpoint working (Docker mode)"
else
    log "⚠️  Listener endpoint response: $LISTENER_RESPONSE"
fi

sleep 1

# Test dialer endpoint
log "Testing dialer endpoint..."
DIALER_RESPONSE=$(curl -s -X POST http://localhost:8787/connect \
    -H "Content-Type: application/json" \
    -d "{\"mode\":\"dialer\",\"port\":9999,\"peer\":\"/ip4/$CONTAINER_IP/udp/9999/quic-v1\",\"vpn\":false}")

if echo "$DIALER_RESPONSE" | grep -q "dockerMode"; then
    log "✅ Dialer endpoint working (Docker mode)"
else
    log "⚠️  Dialer endpoint response: $DIALER_RESPONSE"
fi

log ""
log "=========================================="
log "✅ Testing complete!"
log "=========================================="
log ""
log "Web UI: http://localhost:8787"
log "Container: $CONTAINER_IP"
log ""
log "To stop web server: kill $WEB_PID"

