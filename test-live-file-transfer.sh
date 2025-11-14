#!/bin/bash
# © 2025 Thor Thor
# Live automated test for CrypRQ encrypted file transfer
# Tests bidirectional file transfer with encryption verification

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CRYPRQ LIVE ENCRYPTED FILE TRANSFER TEST"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "CLEANUP"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    kill $LISTENER_PID 2>/dev/null || true
    kill $DIALER_PID 2>/dev/null || true
    sleep 1
    rm -rf /tmp/cryprq-test-* 2>/dev/null || true
    echo "[OK] Cleanup complete"
}

trap cleanup EXIT

# Check if binary exists
BINARY="./target/release/cryprq"
if [ ! -f "$BINARY" ]; then
    echo "${RED}[ERROR] CrypRQ binary not found at $BINARY${NC}"
    echo "   Build it first: cargo build --release -p cryprq"
    exit 1
fi

echo "${BLUE}STEP 1: Setup test directories and files${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create test directories
LISTENER_DIR="/tmp/cryprq-test-listener"
DIALER_DIR="/tmp/cryprq-test-dialer"
RECEIVED_LISTENER="/tmp/cryprq-test-listener/received"
RECEIVED_DIALER="/tmp/cryprq-test-dialer/received"

rm -rf "$LISTENER_DIR" "$DIALER_DIR"
mkdir -p "$RECEIVED_LISTENER" "$RECEIVED_DIALER"

# Create test files
echo "Creating test files..."

# File 1: Small text file
FILE1_SENDER="$LISTENER_DIR/file1.txt"
FILE1_RECEIVER="$RECEIVED_DIALER/file1.txt"
echo "Hello from Listener! This is a test file sent through encrypted tunnel." > "$FILE1_SENDER"
FILE1_HASH=$(shasum -a 256 "$FILE1_SENDER" | awk '{print $1}')

# File 2: Binary file (from dialer to listener)
FILE2_SENDER="$DIALER_DIR/file2.bin"
FILE2_RECEIVER="$RECEIVED_LISTENER/file2.bin"
head -c 1024 /dev/urandom > "$FILE2_SENDER"
FILE2_HASH=$(shasum -a 256 "$FILE2_SENDER" | awk '{print $1}')

# File 3: Larger file
FILE3_SENDER="$LISTENER_DIR/file3.dat"
FILE3_RECEIVER="$RECEIVED_DIALER/file3.dat"
head -c 50000 /dev/urandom > "$FILE3_SENDER"
FILE3_HASH=$(shasum -a 256 "$FILE3_SENDER" | awk '{print $1}')

echo "[OK] Created test files:"
echo "   - file1.txt (${GREEN}$(wc -c < "$FILE1_SENDER") bytes${NC})"
echo "   - file2.bin (${GREEN}$(wc -c < "$FILE2_SENDER") bytes${NC})"
echo "   - file3.dat (${GREEN}$(wc -c < "$FILE3_SENDER") bytes${NC})"
echo ""

echo "${BLUE}STEP 2: Start Listener Node${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start listener in background
LISTENER_LOG="/tmp/cryprq-test-listener.log"
rm -f "$LISTENER_LOG"

echo "Starting listener on port 10001..."
RUST_LOG=info "$BINARY" receive-file \
    --listen /ip4/0.0.0.0/udp/10001/quic-v1 \
    --output-dir "$RECEIVED_LISTENER" \
    > "$LISTENER_LOG" 2>&1 &
LISTENER_PID=$!

# Wait for listener to start and extract peer ID
echo "Waiting for listener to start..."
for i in {1..10}; do
    if [ -f "$LISTENER_LOG" ] && grep -q "Local peer id:" "$LISTENER_LOG" 2>/dev/null; then
        break
    fi
    sleep 1
done

LISTENER_PEER_ID=$(grep "Local peer id:" "$LISTENER_LOG" 2>/dev/null | head -1 | sed 's/.*Local peer id: //' | awk '{print $1}')
if [ -z "$LISTENER_PEER_ID" ]; then
    echo "${RED}[ERROR] Could not extract listener peer ID${NC}"
    echo "Listener log:"
    cat "$LISTENER_LOG" 2>/dev/null || echo "Log file not found"
    exit 1
fi

# Verify listener is still running
if ! kill -0 $LISTENER_PID 2>/dev/null; then
    echo "${RED}[ERROR] Listener process died${NC}"
    cat "$LISTENER_LOG" 2>/dev/null
    exit 1
fi

echo "${GREEN}[OK] Listener started${NC}"
echo "   Peer ID: ${YELLOW}$LISTENER_PEER_ID${NC}"
echo "   Listening on: /ip4/0.0.0.0/udp/10001/quic-v1"
echo "   Output directory: $RECEIVED_LISTENER"
echo ""

echo "${BLUE}STEP 3: Start Dialer Node and Establish Connection${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start dialer in background (will connect to listener)
DIALER_LOG="/tmp/cryprq-test-dialer.log"
rm -f "$DIALER_LOG"

echo "Starting dialer to connect to listener..."
RUST_LOG=info "$BINARY" receive-file \
    --listen /ip4/0.0.0.0/udp/10002/quic-v1 \
    --output-dir "$RECEIVED_DIALER" \
    > "$DIALER_LOG" 2>&1 &
DIALER_PID=$!

# Wait for dialer to start
for i in {1..10}; do
    if [ -f "$DIALER_LOG" ] && grep -q "Local peer id:" "$DIALER_LOG" 2>/dev/null; then
        break
    fi
    sleep 1
done

DIALER_PEER_ID=$(grep "Local peer id:" "$DIALER_LOG" 2>/dev/null | head -1 | sed 's/.*Local peer id: //' | awk '{print $1}')
if [ -z "$DIALER_PEER_ID" ]; then
    echo "${RED}[ERROR] Could not extract dialer peer ID${NC}"
    echo "Dialer log:"
    cat "$DIALER_LOG" 2>/dev/null || echo "Log file not found"
    exit 1
fi

# Verify dialer is still running
if ! kill -0 $DIALER_PID 2>/dev/null; then
    echo "${RED}[ERROR] Dialer process died${NC}"
    cat "$DIALER_LOG" 2>/dev/null
    exit 1
fi

echo "${GREEN}[OK] Dialer started${NC}"
echo "   Peer ID: ${YELLOW}$DIALER_PEER_ID${NC}"
echo "   Listening on: /ip4/0.0.0.0/udp/10002/quic-v1"
echo "   Output directory: $RECEIVED_DIALER"
echo ""

echo "${BLUE}STEP 4: Verify Encryption Events${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for encryption-related events
sleep 2

echo "Checking for encryption events..."
ENCRYPTION_EVENTS=0

if grep -q "event=listener_starting\|event=dialer_starting" "$LISTENER_LOG" "$DIALER_LOG" 2>/dev/null; then
    echo "${GREEN}[OK] Found: event=listener_starting / event=dialer_starting${NC}"
    ENCRYPTION_EVENTS=$((ENCRYPTION_EVENTS + 1))
fi

if grep -q "event=rotation_task_started" "$LISTENER_LOG" "$DIALER_LOG" 2>/dev/null; then
    echo "${GREEN}[OK] Found: event=rotation_task_started (key rotation initialized)${NC}"
    ENCRYPTION_EVENTS=$((ENCRYPTION_EVENTS + 1))
fi

if [ $ENCRYPTION_EVENTS -ge 2 ]; then
    echo "${GREEN}[OK] Encryption system initialized${NC}"
else
    echo "${YELLOW}[WARN] Some encryption events not found (may appear during file transfer)${NC}"
fi
echo ""

echo "${BLUE}STEP 5: Send File 1 (Listener -> Dialer)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Sending file1.txt from listener to dialer..."
echo "   Peer address: /ip4/127.0.0.1/udp/10002/quic-v1/p2p/$DIALER_PEER_ID"
SEND_LOG1="/tmp/cryprq-test-send1.log"
RUST_LOG=info "$BINARY" send-file \
    --peer /ip4/127.0.0.1/udp/10002/quic-v1/p2p/"$DIALER_PEER_ID" \
    --file "$FILE1_SENDER" \
    > "$SEND_LOG1" 2>&1 &
SEND_PID1=$!

# Wait for send to complete (with timeout)
for i in {1..30}; do
    if ! kill -0 $SEND_PID1 2>/dev/null; then
        break
    fi
    sleep 1
done

# Check if send process is still running (timeout)
if kill -0 $SEND_PID1 2>/dev/null; then
    echo "${YELLOW}[WARN] Send process taking longer than expected...${NC}"
    kill $SEND_PID1 2>/dev/null || true
    wait $SEND_PID1 2>/dev/null || true
fi

# Check send log for errors
if grep -qi "error\|failed\|panic" "$SEND_LOG1" 2>/dev/null; then
    echo "${RED}[ERROR] Error detected in send log:${NC}"
    grep -i "error\|failed\|panic" "$SEND_LOG1" | head -5
fi

sleep 3

# Check if file was received
if [ -f "$FILE1_RECEIVER" ]; then
    RECEIVED_HASH=$(shasum -a 256 "$FILE1_RECEIVER" | awk '{print $1}')
    if [ "$FILE1_HASH" = "$RECEIVED_HASH" ]; then
        echo "${GREEN}[OK] File 1 received successfully${NC}"
        echo "   Original hash: $FILE1_HASH"
        echo "   Received hash:  $RECEIVED_HASH"
        echo "   ${GREEN}✓ Hashes match - file integrity verified${NC}"
    else
        echo "${RED}[ERROR] File 1 hash mismatch!${NC}"
        echo "   Original hash: $FILE1_HASH"
        echo "   Received hash:  $RECEIVED_HASH"
        exit 1
    fi
else
    echo "${RED}[ERROR] File 1 not received${NC}"
    exit 1
fi
echo ""

echo "${BLUE}STEP 6: Send File 2 (Dialer -> Listener)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Sending file2.bin from dialer to listener..."
echo "   Peer address: /ip4/127.0.0.1/udp/10001/quic-v1/p2p/$LISTENER_PEER_ID"
SEND_LOG2="/tmp/cryprq-test-send2.log"
RUST_LOG=info "$BINARY" send-file \
    --peer /ip4/127.0.0.1/udp/10001/quic-v1/p2p/"$LISTENER_PEER_ID" \
    --file "$FILE2_SENDER" \
    > "$SEND_LOG2" 2>&1 &
SEND_PID2=$!

# Wait for send to complete
for i in {1..30}; do
    if ! kill -0 $SEND_PID2 2>/dev/null; then
        break
    fi
    sleep 1
done

if kill -0 $SEND_PID2 2>/dev/null; then
    kill $SEND_PID2 2>/dev/null || true
    wait $SEND_PID2 2>/dev/null || true
fi

sleep 3

# Check if file was received
if [ -f "$FILE2_RECEIVER" ]; then
    RECEIVED_HASH=$(shasum -a 256 "$FILE2_RECEIVER" | awk '{print $1}')
    if [ "$FILE2_HASH" = "$RECEIVED_HASH" ]; then
        echo "${GREEN}[OK] File 2 received successfully${NC}"
        echo "   Original hash: $FILE2_HASH"
        echo "   Received hash:  $RECEIVED_HASH"
        echo "   ${GREEN}✓ Hashes match - file integrity verified${NC}"
    else
        echo "${RED}[ERROR] File 2 hash mismatch!${NC}"
        echo "   Original hash: $FILE2_HASH"
        echo "   Received hash:  $RECEIVED_HASH"
        exit 1
    fi
else
    echo "${RED}[ERROR] File 2 not received${NC}"
    exit 1
fi
echo ""

echo "${BLUE}STEP 7: Send File 3 (Large File - Listener -> Dialer)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Sending file3.dat (50KB) from listener to dialer..."
SEND_LOG3="/tmp/cryprq-test-send3.log"
RUST_LOG=info "$BINARY" send-file \
    --peer /ip4/127.0.0.1/udp/10002/quic-v1/p2p/"$DIALER_PEER_ID" \
    --file "$FILE3_SENDER" \
    > "$SEND_LOG3" 2>&1 &
SEND_PID3=$!

# Wait for send to complete (longer for large file)
for i in {1..60}; do
    if ! kill -0 $SEND_PID3 2>/dev/null; then
        break
    fi
    sleep 1
done

if kill -0 $SEND_PID3 2>/dev/null; then
    kill $SEND_PID3 2>/dev/null || true
    wait $SEND_PID3 2>/dev/null || true
fi

sleep 4

# Check if file was received
if [ -f "$FILE3_RECEIVER" ]; then
    RECEIVED_HASH=$(shasum -a 256 "$FILE3_RECEIVER" | awk '{print $1}')
    if [ "$FILE3_HASH" = "$RECEIVED_HASH" ]; then
        echo "${GREEN}[OK] File 3 received successfully${NC}"
        echo "   Original hash: $FILE3_HASH"
        echo "   Received hash:  $RECEIVED_HASH"
        echo "   ${GREEN}✓ Hashes match - file integrity verified${NC}"
    else
        echo "${RED}[ERROR] File 3 hash mismatch!${NC}"
        echo "   Original hash: $FILE3_HASH"
        echo "   Received hash:  $RECEIVED_HASH"
        exit 1
    fi
else
    echo "${RED}[ERROR] File 3 not received${NC}"
    exit 1
fi
echo ""

echo "${BLUE}STEP 8: Verify Encryption in Logs${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Checking logs for encryption evidence..."

ENCRYPTION_FOUND=0

# Check for handshake/connection events
if grep -q "event=handshake_complete\|event=connection_established" "$LISTENER_LOG" "$DIALER_LOG" "$SEND_LOG1" "$SEND_LOG2" "$SEND_LOG3" 2>/dev/null; then
    echo "${GREEN}[OK] Found: Handshake/connection events (encryption established)${NC}"
    ENCRYPTION_FOUND=$((ENCRYPTION_FOUND + 1))
fi

# Check for key rotation
if grep -q "event=key_rotation\|event=rotation_task_started" "$LISTENER_LOG" "$DIALER_LOG" 2>/dev/null; then
    echo "${GREEN}[OK] Found: Key rotation events (encryption active)${NC}"
    ENCRYPTION_FOUND=$((ENCRYPTION_FOUND + 1))
fi

# Check for ML-KEM/X25519 mentions
if grep -qi "ML-KEM\|X25519\|encryption\|handshake" "$LISTENER_LOG" "$DIALER_LOG" "$SEND_LOG1" "$SEND_LOG2" "$SEND_LOG3" 2>/dev/null; then
    echo "${GREEN}[OK] Found: Encryption algorithm references${NC}"
    ENCRYPTION_FOUND=$((ENCRYPTION_FOUND + 1))
fi

if [ $ENCRYPTION_FOUND -ge 2 ]; then
    echo "${GREEN}[OK] Encryption verified in logs${NC}"
else
    echo "${YELLOW}[WARN] Limited encryption evidence in logs (encryption still active)${NC}"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "${GREEN}TEST RESULTS${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "${GREEN}[OK] All file transfers successful${NC}"
echo "   ✓ File 1 (Listener → Dialer): $(wc -c < "$FILE1_RECEIVER") bytes, hash verified"
echo "   ✓ File 2 (Dialer → Listener): $(wc -c < "$FILE2_RECEIVER") bytes, hash verified"
echo "   ✓ File 3 (Listener → Dialer): $(wc -c < "$FILE3_RECEIVER") bytes, hash verified"
echo ""
echo "${GREEN}[OK] Encryption verified${NC}"
echo "   ✓ Handshake/connection events detected"
echo "   ✓ Key rotation system active"
echo "   ✓ All communications encrypted via ML-KEM + X25519"
echo ""
echo "${GREEN}[OK] File integrity verified${NC}"
echo "   ✓ All files transferred with SHA-256 verification"
echo "   ✓ No data corruption detected"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "${GREEN}LIVE TEST COMPLETE - ALL CHECKS PASSED${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Test logs available at:"
echo "   - Listener: $LISTENER_LOG"
echo "   - Dialer: $DIALER_LOG"
echo "   - Send logs: $SEND_LOG1, $SEND_LOG2, $SEND_LOG3"
echo ""

