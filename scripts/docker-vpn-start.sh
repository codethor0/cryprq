#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Start CrypRQ VPN container for system-wide routing
# Mac connects to container, container handles encryption and routing

set -euo pipefail

COMPOSE_FILE="docker-compose.vpn.yml"
SERVICE_NAME="cryprq-vpn"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "Starting CrypRQ VPN container..."

# Build image if needed
if ! docker images | grep -q "cryprq-vpn"; then
    log "Building Docker image..."
    docker-compose -f "$COMPOSE_FILE" build
fi

# Start container
log "Starting container..."
docker-compose -f "$COMPOSE_FILE" up -d

# Wait for container to be ready
log "Waiting for container to be ready..."
sleep 3

# Get container IP
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$SERVICE_NAME" 2>/dev/null || echo "")

if [ -z "$CONTAINER_IP" ]; then
    log "Warning: Could not get container IP. Container may still be starting."
    log "Container logs:"
    docker-compose -f "$COMPOSE_FILE" logs --tail=20 "$SERVICE_NAME"
else
    log "✅ Container started successfully!"
    log "Container IP: $CONTAINER_IP"
    log "Connect to container at: /ip4/$CONTAINER_IP/udp/9999/quic-v1"
fi

# Show container status
log "Container status:"
docker-compose -f "$COMPOSE_FILE" ps

# Show logs
log "Container logs (last 20 lines):"
docker-compose -f "$COMPOSE_FILE" logs --tail=20 "$SERVICE_NAME"

log ""
log "To view logs: docker-compose -f $COMPOSE_FILE logs -f $SERVICE_NAME"
log "To stop: docker-compose -f $COMPOSE_FILE down"

