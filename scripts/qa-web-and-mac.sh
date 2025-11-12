#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Web QA + macOS Packaging Orchestrator
# Runs on every change: validates web client, packages macOS app, smoke-tests both

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Configuration
WEB_DIR="${ROOT}/web"
WEB_SERVER_DIR="${WEB_DIR}/server"
ART_WEB="${ROOT}/artifacts/web-qa"
ART_MAC="${ROOT}/artifacts/macos-app"
PORT_WEB="${PORT_WEB:-5173}"
PORT_BRIDGE="${PORT_BRIDGE:-8787}"
CRYPRQ_PORT="${CRYPRQ_PORT:-9999}"
ROTATE_SECS="${ROTATE_SECS:-10}"

mkdir -p "$ART_WEB" "$ART_MAC"

have(){ command -v "$1" >/dev/null 2>&1; }
note(){ echo "[qa-web-mac] $*"; }
fail(){ echo "[qa-web-mac] ERROR: $*" >&2; exit 1; }

# Quality gate flags
WEB_FAILED=0
MAC_FAILED=0

# Cleanup function
cleanup() {
  note "Cleaning up..."
  pkill -f "node.*server.mjs" >/dev/null 2>&1 || true
  pkill -f "http-server.*web/dist" >/dev/null 2>&1 || true
  pkill -f "vite.*5173" >/dev/null 2>&1 || true
  pkill -f "cryprq.*listen" >/dev/null 2>&1 || true
}
trap cleanup EXIT

# ========== Web QA ==========
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
note "Web Cross-Browser QA"
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Build macOS binary (needed for bridge)
note "Building macOS binary for bridge server..."
rustup target add aarch64-apple-darwin >/dev/null 2>&1 || true
MACBIN="${ROOT}/target/aarch64-apple-darwin/release/cryprq"
if [[ ! -x "$MACBIN" ]]; then
  cargo build --release -p cryprq --target aarch64-apple-darwin 2>&1 | tee "${ART_WEB}/mac_build.txt" || fail "macOS binary build failed"
fi
note "✅ macOS binary ready: $MACBIN"

# Build web client
if [[ ! -d "${WEB_DIR}" ]]; then
  fail "Web client not found at ${WEB_DIR}. Run scripts/build-web-and-mac.sh first."
fi

note "Building web client..."
( cd "${WEB_DIR}" && npm ci >/dev/null 2>&1 || npm install )
( cd "${WEB_DIR}" && npm run build ) 2>&1 | tee "${ART_WEB}/web_build.txt" || fail "Web client build failed"
note "✅ Web client built"

# Start bridge server
note "Starting bridge server on port ${PORT_BRIDGE}..."
( cd "${WEB_SERVER_DIR}" && npm ci >/dev/null 2>&1 || npm install )
( cd "${WEB_SERVER_DIR}" && BRIDGE_PORT="${PORT_BRIDGE}" CRYPRQ_BIN="${MACBIN}" node server.mjs ) > "${ART_WEB}/bridge.log" 2>&1 &
BR_PID=$!
sleep 2

# Verify bridge is running
if ! curl -s "http://localhost:${PORT_BRIDGE}/events" >/dev/null 2>&1; then
  fail "Bridge server failed to start"
fi
note "✅ Bridge server running (PID: $BR_PID)"

# Serve built site
note "Serving built site on port ${PORT_WEB}..."
npx -y http-server "${WEB_DIR}/dist" -p "${PORT_WEB}" -s > "${ART_WEB}/http.log" 2>&1 &
HTTP_PID=$!
sleep 2

# Verify site is accessible
if ! curl -s "http://localhost:${PORT_WEB}" >/dev/null 2>&1; then
  fail "HTTP server failed to start"
fi
note "✅ HTTP server running (PID: $HTTP_PID)"

# Run Playwright tests
note "Running Playwright tests (Chromium/Firefox/WebKit)..."
set +e
npm run test:web 2>&1 | tee "${ART_WEB}/playwright.txt"
PLAYWRIGHT_RC=$?
set -e

if [[ $PLAYWRIGHT_RC -ne 0 ]]; then
  WEB_FAILED=1
  fail "Playwright tests failed"
fi

# Verify bridge logs show handshake/rotation events
note "Verifying bridge logs show handshake/rotation events..."
sleep 2
if ! grep -Eiq "handshake|peer|rotation|connected" "${ART_WEB}/bridge.log" 2>/dev/null; then
  WEB_FAILED=1
  fail "Bridge logs missing handshake/peer/rotation events"
fi

note "✅ Web QA passed (all browsers green, events verified)"

# ========== macOS Packaging ==========
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
note "macOS App Packaging"
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Package macOS app
note "Packaging macOS .app bundle..."
bash scripts/package-mac-app.sh 2>&1 | tee "${ART_MAC}/package.log" || {
  MAC_FAILED=1
  fail "macOS app packaging failed"
}

APP_PATH="${ART_MAC}/CrypRQ.app"
if [[ ! -d "$APP_PATH" ]]; then
  MAC_FAILED=1
  fail "macOS app not found at $APP_PATH"
fi
note "✅ macOS app packaged: $APP_PATH"

# Test macOS app
note "Running macOS app smoke test..."
bash scripts/test-mac-app.sh 2>&1 | tee "${ART_MAC}/smoke_test.log" || {
  MAC_FAILED=1
  fail "macOS app smoke test failed"
}

# Verify smoke test logs show successful handshake
if ! grep -Eiq "handshake|connected|peer" "${ART_MAC}/smoke_test.log" 2>/dev/null; then
  MAC_FAILED=1
  fail "macOS app smoke test logs missing handshake/connection confirmation"
fi

note "✅ macOS app smoke test passed"

# ========== Quality Gates ==========
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
note "Quality Gates Summary"
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ $WEB_FAILED -eq 1 ]]; then
  note "❌ Web QA FAILED - Blocking commit"
  exit 1
fi

if [[ $MAC_FAILED -eq 1 ]]; then
  note "❌ macOS Packaging FAILED - Blocking commit"
  exit 1
fi

note "✅ All quality gates passed"

# ========== Artifacts Summary ==========
{
  echo "# Web QA + macOS Packaging Summary"
  echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo ""
  echo "## Web QA"
  echo "- Playwright: ✅ Passed (Chromium/Firefox/WebKit)"
  echo "- Bridge logs: ✅ Handshake/peer/rotation events verified"
  echo "- Artifacts: ${ART_WEB}/"
  echo ""
  echo "## macOS App"
  echo "- Packaging: ✅ Success"
  echo "- Smoke test: ✅ Passed"
  echo "- App path: ${APP_PATH}"
  echo ""
  echo "## Artifacts"
  echo "- Web QA: ${ART_WEB}/"
  echo "- macOS App: ${ART_MAC}/"
} > "${ART_WEB}/summary.md"

note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
note "✅ Web QA + macOS Packaging Complete"
note "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
note ""
note "Web QA: ✅ All browsers passed"
note "macOS App: ✅ Packaged and verified"
note ""
note "Artifacts:"
note "  Web: ${ART_WEB}/"
note "  macOS: ${ART_MAC}/"

