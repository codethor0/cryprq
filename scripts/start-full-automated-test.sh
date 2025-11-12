#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Complete automated test: Docker + Web Server + Browser + Tests
# This is the main script to run everything automatically

set -euo pipefail

cd "$(dirname "$0")/.."

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

log "🚀 Starting Full Automated CrypRQ Test Suite..."

# Run the automated browser test script
./scripts/automated-browser-test.sh

log ""
log "✅ All tests completed!"
log ""
log "The browser should be open showing the connection."
log "Check the debug console for connection events and encryption logs."

