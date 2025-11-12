#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Setup Mac routing to send traffic through Docker VPN container
# This script configures routing tables to route traffic through the container

set -euo pipefail

CONTAINER_NAME="cryprq-vpn"
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$CONTAINER_NAME" 2>/dev/null || echo "")

if [ -z "$CONTAINER_IP" ]; then
    echo "❌ Container $CONTAINER_NAME is not running"
    exit 1
fi

echo "✅ Container IP: $CONTAINER_IP"
echo ""
echo "⚠️  Note: Full system-wide VPN routing requires:"
echo "   1. TUN interface configured in container"
echo "   2. Routing configured in container"
echo "   3. Mac routing configured (this script)"
echo "   4. DNS configuration (optional)"
echo ""
echo "Current limitations:"
echo "   - macOS requires Network Extension framework for true VPN"
echo "   - Docker-based VPN can route specific traffic only"
echo ""
echo "To route traffic through container:"
echo "  sudo route add -net 0.0.0.0/1 $CONTAINER_IP"
echo "  sudo route add -net 128.0.0.0/1 $CONTAINER_IP"
echo ""
echo "⚠️  WARNING: This will route ALL traffic through the container!"
echo "   Make sure the container is properly configured first."
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# Add routes (requires sudo)
sudo route add -net 0.0.0.0/1 "$CONTAINER_IP" 2>/dev/null || true
sudo route add -net 128.0.0.0/1 "$CONTAINER_IP" 2>/dev/null || true

echo "✅ Routes added"
echo ""
echo "To remove routes later:"
echo "  sudo route delete -net 0.0.0.0/1 $CONTAINER_IP"
echo "  sudo route delete -net 128.0.0.0/1 $CONTAINER_IP"

