#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Stop CrypRQ VPN container

set -euo pipefail

COMPOSE_FILE="docker-compose.vpn.yml"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "Stopping CrypRQ VPN container..."

docker-compose -f "$COMPOSE_FILE" down

log "✅ Container stopped"

