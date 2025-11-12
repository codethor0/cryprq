#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Test Docker container connection
# This script tests if Mac can connect to the Docker container

set -euo pipefail

CONTAINER_NAME="cryprq-vpn"
COMPOSE_FILE="docker-compose.vpn.yml"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "Testing Docker container connection..."

# Check if container is running
if ! docker ps | grep -q "$CONTAINER_NAME"; then
    log "Container is not running. Starting it..."
    ./scripts/docker-vpn-start.sh
    sleep 5
fi

# Get container IP
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$CONTAINER_NAME" 2>/dev/null || echo "")

if [ -z "$CONTAINER_IP" ]; then
    log "ERROR: Could not get container IP"
    exit 1
fi

log "Container IP: $CONTAINER_IP"
log "Container peer address: /ip4/$CONTAINER_IP/udp/9999/quic-v1"

# Check if container is listening
log "Checking if container is listening on port 9999..."
if docker exec "$CONTAINER_NAME" netstat -uln 2>/dev/null | grep -q ":9999"; then
    log "✅ Container is listening on port 9999"
else
    log "⚠️  Container may not be listening (netstat not available or port not bound)"
fi

# Test connectivity from Mac
log "Testing connectivity from Mac..."
if ping -c 1 "$CONTAINER_IP" >/dev/null 2>&1; then
    log "✅ Mac can ping container"
else
    log "⚠️  Mac cannot ping container (may be normal if using bridge network)"
fi

# Show container logs
log "Container logs (last 10 lines):"
docker logs --tail 10 "$CONTAINER_NAME" 2>&1

log ""
log "To connect from Mac:"
log "./target/release/cryprq --peer /ip4/$CONTAINER_IP/udp/9999/quic-v1"
log ""
log "Or use the web UI with Docker mode enabled"

