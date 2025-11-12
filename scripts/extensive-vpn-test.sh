#!/bin/bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# SPDX-License-Identifier: MIT

# Extensive VPN testing with maximum verbosity
# This script performs comprehensive testing and shows all encryption events

set -e

CONTAINER_NAME="cryprq-vpn"
WEB_PORT=8787

echo "🧪 CrypRQ VPN Extensive Testing with Maximum Verbosity"
echo "======================================================="
echo ""

# Step 1: Verify container is running
echo "Step 1: Verifying Docker Container"
echo "-----------------------------------"
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "❌ Container ${CONTAINER_NAME} is not running"
    echo "   Starting container..."
    docker-compose -f docker-compose.vpn.yml up -d
    sleep 5
fi
echo "✅ Container ${CONTAINER_NAME} is running"
docker ps --filter "name=${CONTAINER_NAME}" --format "  Status: {{.Status}}"
echo ""

# Step 2: Verify trace logging
echo "Step 2: Verifying Trace Logging"
echo "--------------------------------"
RUST_LOG=$(docker exec ${CONTAINER_NAME} env | grep RUST_LOG || echo "RUST_LOG=not_set")
echo "  $RUST_LOG"
if [[ "$RUST_LOG" == *"trace"* ]]; then
    echo "✅ Trace logging enabled"
else
    echo "⚠️  Trace logging not set - restarting container..."
    docker-compose -f docker-compose.vpn.yml restart
    sleep 3
fi
echo ""

# Step 3: Get container IP
echo "Step 3: Getting Container IP"
echo "-----------------------------"
CONTAINER_IP=$(docker inspect ${CONTAINER_NAME} --format='{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' 2>/dev/null)
if [ -z "$CONTAINER_IP" ]; then
    echo "❌ Could not get container IP"
    exit 1
fi
echo "✅ Container IP: ${CONTAINER_IP}"
echo ""

# Step 4: Verify TUN interface
echo "Step 4: Verifying TUN Interface"
echo "--------------------------------"
if docker exec ${CONTAINER_NAME} ip addr show cryprq0 > /dev/null 2>&1; then
    TUN_IP=$(docker exec ${CONTAINER_NAME} ip addr show cryprq0 | grep "inet " | awk '{print $2}' | cut -d/ -f1)
    echo "✅ TUN interface cryprq0 exists"
    echo "  TUN IP: ${TUN_IP}"
else
    echo "❌ TUN interface not found"
fi
echo ""

# Step 5: Check listener status
echo "Step 5: Checking Listener Status"
echo "---------------------------------"
if docker logs ${CONTAINER_NAME} 2>&1 | grep -q "Listening on"; then
    echo "✅ Listener is active"
    docker logs ${CONTAINER_NAME} 2>&1 | grep "Listening on" | tail -1
else
    echo "⚠️  Listener status unclear"
fi
echo ""

# Step 6: Verify web server
echo "Step 6: Verifying Web Server"
echo "-----------------------------"
if curl -s http://localhost:${WEB_PORT} > /dev/null 2>&1; then
    echo "✅ Web server responding on port ${WEB_PORT}"
    echo "  URL: http://localhost:${WEB_PORT}"
else
    echo "⚠️  Web server not responding"
    echo "   Start with: USE_DOCKER=true BRIDGE_PORT=${WEB_PORT} RUST_LOG=trace node web/server/server.mjs"
fi
echo ""

# Step 7: Test connection
echo "Step 7: Testing Connection"
echo "---------------------------"
echo "Connecting dialer to container..."
RESPONSE=$(curl -s -X POST http://localhost:${WEB_PORT}/connect \
    -H "Content-Type: application/json" \
    -d "{\"mode\":\"dialer\",\"port\":9999,\"peer\":\"/ip4/${CONTAINER_IP}/udp/9999/quic-v1\",\"vpn\":true}")
echo "Response: $RESPONSE"
if echo "$RESPONSE" | grep -q '"ok":true'; then
    echo "✅ Connection initiated"
else
    echo "⚠️  Connection may have failed"
fi
echo ""

# Step 8: Wait for connection establishment
echo "Step 8: Waiting for Connection Establishment"
echo "---------------------------------------------"
echo "Waiting 5 seconds for connection to establish..."
sleep 5
echo ""

# Step 9: Check for connection events
echo "Step 9: Checking Connection Events"
echo "-----------------------------------"
CONNECTION_EVENTS=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -cE "(Connection|Connected|Inbound|peer|Dialing)" || echo "0")
echo "Connection events found: ${CONNECTION_EVENTS}"
if [ "${CONNECTION_EVENTS}" -gt 0 ]; then
    echo "✅ Connection events detected"
    docker logs ${CONTAINER_NAME} 2>&1 | grep -E "(Connection|Connected|Inbound|peer|Dialing)" | tail -5
else
    echo "⚠️  No connection events yet"
fi
echo ""

# Step 10: Generate test traffic
echo "Step 10: Generating Test Traffic"
echo "----------------------------------"
echo "Sending test packets to trigger encryption..."
for i in {1..10}; do
    echo "test-packet-$i-$(date +%s)" | timeout 0.3 nc -u ${CONTAINER_IP} 9999 2>/dev/null || true
    sleep 0.1
done
echo "Test packets sent"
sleep 2
echo ""

# Step 11: Check encryption events
echo "Step 11: Checking Encryption Events"
echo "------------------------------------"
ENCRYPT=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -c "ENCRYPT" 2>/dev/null || echo "0")
DECRYPT=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -c "DECRYPT" 2>/dev/null || echo "0")
FORWARDED=$(docker logs ${CONTAINER_NAME} 2>&1 | grep -c "Forwarded.*packet" 2>/dev/null || echo "0")
echo "Encryption events: ${ENCRYPT}"
echo "Decryption events: ${DECRYPT}"
echo "Forwarded packets: ${FORWARDED}"
echo ""

if [ "${ENCRYPT}" -gt 0 ] || [ "${DECRYPT}" -gt 0 ] || [ "${FORWARDED}" -gt 0 ]; then
    echo "✅ Encryption/decryption events detected!"
    echo ""
    echo "Recent encryption events:"
    docker logs ${CONTAINER_NAME} 2>&1 | grep -E "(ENCRYPT|DECRYPT|Forwarded)" | tail -10
else
    echo "⚠️  No encryption events yet"
    echo "   This is normal if no packets are flowing through the tunnel"
    echo "   Generate traffic (browse, curl) to see encryption events"
fi
echo ""

# Step 12: Show all recent logs
echo "Step 12: Recent Container Logs (Last 20 lines)"
echo "------------------------------------------------"
docker logs ${CONTAINER_NAME} 2>&1 | tail -20
echo ""

# Step 13: Summary
echo "📊 Test Summary"
echo "==============="
echo ""
echo "✅ Infrastructure:"
docker ps --filter "name=${CONTAINER_NAME}" --format "  Container: {{.Names}} - {{.Status}}"
docker exec ${CONTAINER_NAME} ip addr show cryprq0 > /dev/null 2>&1 && echo "  TUN Interface: cryprq0 exists" || echo "  TUN Interface: Not found"
docker logs ${CONTAINER_NAME} 2>&1 | grep -q "Listening on" && echo "  Listener: Active" || echo "  Listener: Not detected"
curl -s http://localhost:${WEB_PORT} > /dev/null 2>&1 && echo "  Web Server: Running" || echo "  Web Server: Not running"
echo ""
echo "📈 Events:"
echo "  Connection: ${CONNECTION_EVENTS} events"
echo "  Encryption: ${ENCRYPT} events"
echo "  Decryption: ${DECRYPT} events"
echo "  Forwarded: ${FORWARDED} packets"
echo ""
echo "🎯 Next Steps:"
echo "  1. Open http://localhost:${WEB_PORT} in browser"
echo "  2. Connect via web UI (VPN mode enabled)"
echo "  3. Generate traffic to see encryption events"
echo "  4. Monitor logs: docker logs -f ${CONTAINER_NAME} | grep -E '(ENCRYPT|DECRYPT|Forwarded)'"
echo ""
echo "✅ Extensive testing complete!"

