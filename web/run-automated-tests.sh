#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Run automated browser tests for CrypRQ Web UI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🤖 Automated Browser Tests for CrypRQ Web UI"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This will:"
echo "  1. Start the server bridge"
echo "  2. Start the Vite dev server"
echo "  3. Open browser windows (you can watch!)"
echo "  4. Set up listener in Tab 1"
echo "  5. Set up dialer in Tab 2"
echo "  6. Test the connection"
echo "  7. Show results"
echo ""
echo "Press Ctrl+C at any time to stop"
echo ""
read -p "Press Enter to start, or Ctrl+C to cancel..."

cd "$SCRIPT_DIR"

# Check if Puppeteer is installed
if ! npm list puppeteer >/dev/null 2>&1; then
  echo "📦 Installing Puppeteer..."
  npm install puppeteer --save-dev
fi

# Set CRYPRQ_BIN if not set
if [ -z "$CRYPRQ_BIN" ]; then
  DEFAULT_BIN="$PROJECT_ROOT/dist/macos/CrypRQ.app/Contents/MacOS/CrypRQ"
  if [ -f "$DEFAULT_BIN" ]; then
    export CRYPRQ_BIN="$DEFAULT_BIN"
    echo "✅ Using default binary: $CRYPRQ_BIN"
  else
    echo "❌ CrypRQ binary not found. Please set CRYPRQ_BIN or build the app."
    exit 1
  fi
fi

echo ""
echo "🚀 Starting automated tests..."
echo ""

# Run the test script
node test-automated.js

