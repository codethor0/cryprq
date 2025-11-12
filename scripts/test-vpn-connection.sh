#!/bin/bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# SPDX-License-Identifier: MIT

# Test VPN connection and packet forwarding

set -e

CONTAINER_NAME="cryprq-vpn"
TIMEOUT=30

echo "🧪 Testing CrypRQ VPN Connection"
echo ""

# Check if container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "❌ Container ${CONTAINER_NAME} is not running"
    echo "   Start it with: docker-compose -f docker-compose.vpn.yml up -d"
    exit 1
fi

echo "✅ Container ${CONTAINER_NAME} is running"

# Get container IP
CONTAINER_IP=$(docker inspect ${CONTAINER_NAME} --format='{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' 2>/dev/null)
if [ -z "$CONTAINER_IP" ]; then
    echo "❌ Could not get container IP"
    exit 1
fi

echo "✅ Container IP: ${CONTAINER_IP}"

# Check TUN interface
echo ""
echo "📡 Checking TUN interface..."
if docker exec ${CONTAINER_NAME} ip addr show cryprq0 > /dev/null 2>&1; then
    TUN_IP=$(docker exec ${CONTAINER_NAME} ip addr show cryprq0 | grep "inet " | awk '{print $2}' | cut -d/ -f1)
    echo "✅ TUN interface cryprq0 exists"
    echo "   TUN IP: ${TUN_IP}"
else
    echo "⚠️  TUN interface cryprq0 not found"
fi

# Check listener status
echo ""
echo "🔍 Checking listener status..."
if docker logs ${CONTAINER_NAME} 2>&1 | grep -q "Listening on"; then
    echo "✅ Listener is active"
    docker logs ${CONTAINER_NAME} 2>&1 | grep "Listening on" | tail -1
else
    echo "⚠️  Listener status unclear"
fi

# Test connection
echo ""
echo "🔌 Testing connection..."
echo "   Connecting to: /ip4/${CONTAINER_IP}/udp/9999/quic-v1"
echo "   This will timeout after ${TIMEOUT} seconds"
echo ""

# Run dialer in background and capture output
LOG_FILE=$(mktemp)
timeout ${TIMEOUT} cargo run --bin cryprq -- --peer "/ip4/${CONTAINER_IP}/udp/9999/quic-v1" --vpn > "${LOG_FILE}" 2>&1 &
DIALER_PID=$!

# Wait a bit for connection
sleep 5

# Check for connection success
if grep -q "Connection established\|Connected to\|Inbound connection" "${LOG_FILE}" 2>/dev/null; then
    echo "✅ Connection established!"
else
    echo "⚠️  Connection status unclear (checking logs...)"
fi

# Check for encryption events
echo ""
echo "🔐 Checking encryption events..."
ENCRYPT_COUNT=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -c "ENCRYPT" || echo "0")
DECRYPT_COUNT=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -c "DECRYPT" || echo "0")
FORWARDED_COUNT=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -c "Forwarded.*packet" || echo "0")

echo "   Encryption events: ${ENCRYPT_COUNT}"
echo "   Decryption events: ${DECRYPT_COUNT}"
echo "   Forwarded packets: ${FORWARDED_COUNT}"

# Cleanup
if kill -0 ${DIALER_PID} 2>/dev/null; then
    kill ${DIALER_PID} 2>/dev/null || true
fi

rm -f "${LOG_FILE}"

echo ""
if [ "${ENCRYPT_COUNT}" -gt 0 ] || [ "${DECRYPT_COUNT}" -gt 0 ] || [ "${FORWARDED_COUNT}" -gt 0 ]; then
    echo "✅ Packet forwarding is working!"
else
    echo "⚠️  No packet forwarding events detected yet"
    echo "   This may be normal if no traffic was generated"
    echo "   Try generating test traffic (ping, curl, etc.)"
fi

echo ""
echo "📋 Next steps:"
echo "   1. Generate test traffic to trigger packet forwarding"
echo "   2. Monitor logs: docker logs -f ${CONTAINER_NAME}"
echo "   3. Check web UI: http://localhost:8787 (if web server is running)"

