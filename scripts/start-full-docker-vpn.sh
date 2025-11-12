#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Start complete Docker VPN setup
# 1. Starts Docker container
# 2. Starts web server with Docker mode
# 3. Shows connection info

set -euo pipefail

cd "$(dirname "$0")/.."

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "🚀 Starting CrypRQ Docker VPN..."

# Check Docker
if ! docker ps > /dev/null 2>&1; then
    log "❌ Docker is not running. Please start Docker Desktop first."
    exit 1
fi

# Start container
log "📦 Starting Docker container..."
./scripts/docker-vpn-start.sh

# Wait for container
sleep 3

# Get container info
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn 2>/dev/null || echo "")

if [ -z "$CONTAINER_IP" ]; then
    log "❌ Could not get container IP. Check container logs:"
    docker-compose -f docker-compose.vpn.yml logs --tail=20
    exit 1
fi

log "✅ Container running at IP: $CONTAINER_IP"
log "✅ Container peer: /ip4/$CONTAINER_IP/udp/9999/quic-v1"

# Start web server
log "🌐 Starting web server with Docker mode..."
export USE_DOCKER=true
export BRIDGE_PORT=8787

log ""
log "=========================================="
log "✅ Docker VPN is ready!"
log "=========================================="
log ""
log "Container IP: $CONTAINER_IP"
log "Web UI: http://localhost:8787"
log "Container peer: /ip4/$CONTAINER_IP/udp/9999/quic-v1"
log ""
log "To connect from CLI:"
log "./target/release/cryprq --peer /ip4/$CONTAINER_IP/udp/9999/quic-v1"
log ""
log "Starting web server..."
log "Press Ctrl+C to stop"
log ""

cd web/server
node docker-bridge.mjs

