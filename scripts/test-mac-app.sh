#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Smoke-launch helper for macOS .app
# Tests that the app launches and can connect

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_NAME="${APP_NAME:-CrypRQ}"
APPDIR="${ROOT}/artifacts/macos-app/${APP_NAME}.app"
BIN="${APPDIR}/Contents/MacOS/cryprq"
PORT="${CRYPRQ_PORT:-9999}"

if [[ ! -d "$APPDIR" ]]; then
  echo "Error: App not found at $APPDIR"
  echo "Run: make mac-app or bash scripts/package-mac-app.sh"
  exit 1
fi

if [[ ! -x "$BIN" ]]; then
  echo "Error: Binary not found at $BIN"
  exit 1
fi

echo "Testing macOS app: $APPDIR"
echo ""

# Test 1: Version check
echo "Test 1: Version check"
"$BIN" --version || exit 1
echo "✅ Version check passed"
echo ""

# Test 2: Listener + dialer connection
echo "Test 2: Listener + dialer connection"
set +e
"$BIN" --listen "/ip4/0.0.0.0/udp/${PORT}/quic-v1" > /tmp/cryprq-listener.log 2>&1 &
L_PID=$!
sleep 2

"$BIN" --peer "/ip4/127.0.0.1/udp/${PORT}/quic-v1" > /tmp/cryprq-dialer.log 2>&1
sleep 3

kill $L_PID >/dev/null 2>&1 || true
set -e

if grep -Ei "connected|handshake|ping" /tmp/cryprq-dialer.log >/dev/null; then
  echo "✅ Connection test passed"
else
  echo "❌ Connection test failed"
  echo "Listener log:"
  cat /tmp/cryprq-listener.log
  echo ""
  echo "Dialer log:"
  cat /tmp/cryprq-dialer.log
  exit 1
fi

echo ""
echo "✅ All macOS app tests passed"

