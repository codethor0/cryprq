#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Start CrypRQ Web UI with macOS build

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRYPRQ_BIN="$PROJECT_ROOT/dist/macos/CrypRQ.app/Contents/MacOS/CrypRQ"
BRIDGE_PORT=${BRIDGE_PORT:-8787}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌐 Starting CrypRQ Web UI"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if macOS binary exists
if [ ! -f "$CRYPRQ_BIN" ]; then
  echo "❌ macOS binary not found at: $CRYPRQ_BIN"
  echo ""
  echo "Please build first:"
  echo "  ./scripts/build-macos.sh"
  exit 1
fi

echo "✅ Using macOS binary: $CRYPRQ_BIN"
echo ""

# Check if server dependencies are installed
if [ ! -d "$SCRIPT_DIR/server/node_modules" ]; then
  echo "📦 Installing server dependencies..."
  cd "$SCRIPT_DIR/server"
  npm install
  cd "$SCRIPT_DIR"
fi

# Check if web dependencies are installed
if [ ! -d "$SCRIPT_DIR/node_modules" ]; then
  echo "📦 Installing web dependencies..."
  npm install
fi

# Check if port 8787 is in use and kill existing process
if lsof -ti:$BRIDGE_PORT >/dev/null 2>&1; then
  echo "⚠️  Port $BRIDGE_PORT is already in use"
  echo "🔄 Killing existing process..."
  lsof -ti:$BRIDGE_PORT | xargs kill -9 2>/dev/null || true
  sleep 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Services"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Server will start on: http://localhost:$BRIDGE_PORT"
echo "Frontend will start on: http://localhost:5173"
echo ""
echo "Press Ctrl+C to stop both services"
echo ""

# Export the binary path and port
export CRYPRQ_BIN="$CRYPRQ_BIN"
export BRIDGE_PORT="$BRIDGE_PORT"

# Start server in background
cd "$SCRIPT_DIR/server"
node server.mjs &
SERVER_PID=$!

# Wait a moment for server to start
sleep 2

# Check if server started successfully
if ! kill -0 $SERVER_PID 2>/dev/null; then
  echo "❌ Server failed to start"
  exit 1
fi

# Start frontend
cd "$SCRIPT_DIR"
npm run dev &
FRONTEND_PID=$!

# Wait a moment for frontend to start
sleep 1

# Check if frontend started successfully
if ! kill -0 $FRONTEND_PID 2>/dev/null; then
  echo "❌ Frontend failed to start"
  kill $SERVER_PID 2>/dev/null || true
  exit 1
fi

echo ""
echo "✅ Both services started successfully!"
echo ""
echo "🌐 Open in browser: http://localhost:5173"
echo ""

# Function to cleanup on exit
cleanup() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🛑 Stopping services..."
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  kill $SERVER_PID 2>/dev/null || true
  kill $FRONTEND_PID 2>/dev/null || true
  wait $SERVER_PID 2>/dev/null || true
  wait $FRONTEND_PID 2>/dev/null || true
  exit 0
}

trap cleanup SIGINT SIGTERM

# Wait for both processes
wait

