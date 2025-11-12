#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Start web server with Docker mode enabled
# This connects the web UI to the Docker container instead of local binary

set -euo pipefail

cd "$(dirname "$0")/.."

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "Starting web server with Docker mode..."

# Ensure Docker container is running
log "Checking Docker container..."
if ! docker ps | grep -q cryprq-vpn; then
    log "Starting Docker container..."
    ./scripts/docker-vpn-start.sh
    sleep 3
fi

# Start web server with Docker mode
log "Starting web server..."
export USE_DOCKER=true
export BRIDGE_PORT=8787
export DOCKER_BRIDGE_PORT=8788

cd web/server
node server.mjs

