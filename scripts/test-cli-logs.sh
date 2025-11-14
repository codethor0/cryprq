#!/bin/bash
# © 2025 Thor Thor
# Test script to verify CLI logging is observable
# This script starts a listener and dialer, then checks for expected log events

set -e

BINARY="./target/release/cryprq"
LISTENER_PORT=9999
LOG_DIR="/tmp/cryprq-test-logs"
LISTENER_LOG="$LOG_DIR/listener.log"
DIALER_LOG="$LOG_DIR/dialer.log"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CrypRQ CLI Logging Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}ERROR: Binary not found at $BINARY${NC}"
    echo "Please build first: cargo build --release -p cryprq"
    exit 1
fi

# Create log directory
mkdir -p "$LOG_DIR"

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    pkill -f "cryprq.*--listen" 2>/dev/null || true
    pkill -f "cryprq.*--peer" 2>/dev/null || true
    sleep 1
}

trap cleanup EXIT

echo "Starting listener..."
RUST_LOG=info "$BINARY" --listen "/ip4/0.0.0.0/udp/$LISTENER_PORT/quic-v1" > "$LISTENER_LOG" 2>&1 &
LISTENER_PID=$!

# Wait for listener to start
sleep 2

# Check listener started
if ! kill -0 $LISTENER_PID 2>/dev/null; then
    echo -e "${RED}ERROR: Listener failed to start${NC}"
    cat "$LISTENER_LOG"
    exit 1
fi

echo -e "${GREEN}✓ Listener started (PID: $LISTENER_PID)${NC}"

# Check for listener startup logs
echo ""
echo "Checking listener logs..."
if grep -q "event=listener_starting" "$LISTENER_LOG"; then
    echo -e "${GREEN}✓ Found: event=listener_starting${NC}"
else
    echo -e "${RED}✗ Missing: event=listener_starting${NC}"
fi

if grep -q "event=listener_ready" "$LISTENER_LOG"; then
    echo -e "${GREEN}✓ Found: event=listener_ready${NC}"
else
    echo -e "${RED}✗ Missing: event=listener_ready${NC}"
fi

if grep -q "event=rotation_task_started" "$LISTENER_LOG"; then
    echo -e "${GREEN}✓ Found: event=rotation_task_started${NC}"
else
    echo -e "${RED}✗ Missing: event=rotation_task_started${NC}"
fi

echo ""
echo "Starting dialer..."
RUST_LOG=info "$BINARY" --peer "/ip4/127.0.0.1/udp/$LISTENER_PORT/quic-v1" > "$DIALER_LOG" 2>&1 &
DIALER_PID=$!

# Wait for connection
sleep 3

# Check dialer started
if ! kill -0 $DIALER_PID 2>/dev/null; then
    echo -e "${RED}ERROR: Dialer failed to start${NC}"
    cat "$DIALER_LOG"
    exit 1
fi

echo -e "${GREEN}✓ Dialer started (PID: $DIALER_PID)${NC}"

# Check for dialer startup logs
echo ""
echo "Checking dialer logs..."
if grep -q "event=dialer_starting" "$DIALER_LOG"; then
    echo -e "${GREEN}✓ Found: event=dialer_starting${NC}"
else
    echo -e "${RED}✗ Missing: event=dialer_starting${NC}"
fi

# Wait a bit more for handshake
sleep 2

# Check for handshake logs
echo ""
echo "Checking handshake logs..."
if grep -q "event=handshake_complete" "$LISTENER_LOG" || grep -q "event=handshake_complete" "$DIALER_LOG"; then
    echo -e "${GREEN}✓ Found: event=handshake_complete${NC}"
else
    echo -e "${YELLOW}⚠ Missing: event=handshake_complete (may need more time)${NC}"
fi

if grep -q "event=connection_established" "$LISTENER_LOG" || grep -q "event=connection_established" "$DIALER_LOG"; then
    echo -e "${GREEN}✓ Found: event=connection_established${NC}"
else
    echo -e "${YELLOW}⚠ Missing: event=connection_established (may need more time)${NC}"
fi

# Show recent logs
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Recent Listener Logs:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
tail -20 "$LISTENER_LOG" | grep -E "event=|Local peer|Listening" || tail -10 "$LISTENER_LOG"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Recent Dialer Logs:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
tail -20 "$DIALER_LOG" | grep -E "event=|Local peer|Connected" || tail -10 "$DIALER_LOG"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Log files:"
echo "  Listener: $LISTENER_LOG"
echo "  Dialer:   $DIALER_LOG"
echo ""
echo "To view full logs:"
echo "  tail -f $LISTENER_LOG"
echo "  tail -f $DIALER_LOG"
echo ""
echo "Test complete. Processes will be cleaned up on exit."

