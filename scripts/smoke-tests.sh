#!/bin/bash
set -e

# Post-release smoke tests for CrypRQ Desktop
# Run these manually on each platform after release

echo "🧪 Running CrypRQ Desktop smoke tests..."

PLATFORM=$(uname -s)
echo "Platform: ${PLATFORM}"

# Test 1: Start → Connect → Rotate → Disconnect
echo ""
echo "Test 1: Start → Connect → Rotate → Disconnect"
echo "  - Launch app"
echo "  - Click Connect"
echo "  - Verify tray icon shows 'connected'"
echo "  - Wait for rotation (or simulate)"
echo "  - Verify tray icon shows 'rotating' then 'connected'"
echo "  - Click Disconnect"
echo "  - Verify tray icon shows 'disconnected'"
read -p "  ✓ Pass? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "❌ Test 1 failed"
  exit 1
fi

# Test 2: Fault injection
echo ""
echo "Test 2: Fault injection (exitCode=1)"
echo "  - Start app and connect"
echo "  - Use dev hook to simulate exitCode=1"
echo "  - Verify error modal appears"
echo "  - Verify structured logs contain error entry"
echo "  - Verify diagnostics timeline updated"
read -p "  ✓ Pass? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "❌ Test 2 failed"
  exit 1
fi

# Test 3: Diagnostics export
echo ""
echo "Test 3: Diagnostics export"
echo "  - Export diagnostics zip"
echo "  - Verify zip < 10MB"
echo "  - Verify secrets redacted (grep for 'bearer' or 'privKey' returns nothing)"
echo "  - Verify session-summary.json present"
echo "  - Verify metrics-snapshot.json present"
read -p "  ✓ Pass? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "❌ Test 3 failed"
  exit 1
fi

# Test 4: Structured logs
echo ""
echo "Test 4: Structured logs validation"
echo "  - Check log files contain JSONL format"
echo "  - Verify all entries have required fields (v, ts, lvl, src, event, msg)"
echo "  - Verify no secrets in logs"
read -p "  ✓ Pass? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "❌ Test 4 failed"
  exit 1
fi

echo ""
echo "✅ All smoke tests passed!"

