#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Check Docker VPN status and provide instructions

set -euo pipefail

cd "$(dirname "$0")/.."

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

echo "🔍 Checking CrypRQ Docker VPN Status..."
echo ""

# Check Docker
if docker ps > /dev/null 2>&1; then
    echo "✅ Docker is running"
    DOCKER_RUNNING=true
else
    echo "❌ Docker is NOT running"
    echo "   → Please start Docker Desktop"
    echo "   → Then run: ./scripts/start-full-docker-vpn.sh"
    DOCKER_RUNNING=false
    exit 1
fi

# Check container
if docker ps | grep -q cryprq-vpn; then
    echo "✅ Container 'cryprq-vpn' is running"
    CONTAINER_RUNNING=true
    
    # Get container info
    CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn 2>/dev/null || echo "")
    if [ -n "$CONTAINER_IP" ]; then
        echo "✅ Container IP: $CONTAINER_IP"
        echo "✅ Connect address: /ip4/$CONTAINER_IP/udp/9999/quic-v1"
    fi
    
    # Check container logs
    echo ""
    echo "📋 Container logs (last 5 lines):"
    docker logs --tail 5 cryprq-vpn 2>&1 | sed 's/^/   /'
else
    echo "⚠️  Container 'cryprq-vpn' is NOT running"
    echo "   → Start with: ./scripts/docker-vpn-start.sh"
    CONTAINER_RUNNING=false
fi

# Check web server port
if lsof -ti:8787 > /dev/null 2>&1; then
    PID=$(lsof -ti:8787 | head -1)
    echo "✅ Web server is running on port 8787 (PID: $PID)"
    WEB_RUNNING=true
else
    echo "⚠️  Web server is NOT running on port 8787"
    echo "   → Start with: ./scripts/start-web-docker.sh"
    WEB_RUNNING=false
fi

echo ""
echo "=========================================="
if [ "$DOCKER_RUNNING" = true ] && [ "$CONTAINER_RUNNING" = true ] && [ "$WEB_RUNNING" = true ]; then
    echo "✅ Everything is running!"
    echo "=========================================="
    echo ""
    echo "🌐 Web UI: http://localhost:8787"
    if [ -n "${CONTAINER_IP:-}" ]; then
        echo "🔗 Container: /ip4/$CONTAINER_IP/udp/9999/quic-v1"
    fi
    echo ""
    echo "Ready to connect!"
else
    echo "⚠️  Some components are not running"
    echo "=========================================="
    echo ""
    echo "To start everything:"
    echo "  ./scripts/start-full-docker-vpn.sh"
    echo ""
    echo "Or start individually:"
    echo "  ./scripts/docker-vpn-start.sh    # Start container"
    echo "  ./scripts/start-web-docker.sh     # Start web server"
fi

