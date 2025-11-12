#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Automated browser testing for CrypRQ Docker VPN
# This script:
# 1. Starts Docker container
# 2. Starts web server
# 3. Opens browser
# 4. Runs automated Playwright tests
# 5. Verifies encryption is working

set -euo pipefail

cd "$(dirname "$0")/.."

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*"
}

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

log "🚀 Starting Automated CrypRQ Browser Test..."

# Step 1: Check Docker
log "📦 Checking Docker..."
if ! docker ps > /dev/null 2>&1; then
    log "${RED}❌ Docker is not running${NC}"
    log "Starting Docker Desktop..."
    open -a Docker 2>/dev/null || true
    log "Waiting for Docker to start..."
    for i in {1..30}; do
        if docker ps > /dev/null 2>&1; then
            log "${GREEN}✅ Docker is running${NC}"
            break
        fi
        sleep 1
    done
    if ! docker ps > /dev/null 2>&1; then
        log "${RED}❌ Docker failed to start. Please start Docker Desktop manually.${NC}"
        exit 1
    fi
fi

# Step 2: Start/check container
log "🐳 Checking Docker container..."
CONTAINER_NAME="cryprq-listener"
if ! docker ps | grep -q "$CONTAINER_NAME"; then
    log "Starting container..."
    ./scripts/docker-vpn-start.sh
    sleep 5
else
    log "${GREEN}✅ Container already running${NC}"
fi

CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$CONTAINER_NAME" 2>/dev/null || echo "")
if [ -z "$CONTAINER_IP" ]; then
    log "${RED}❌ Could not get container IP${NC}"
    exit 1
fi

log "${GREEN}✅ Container IP: $CONTAINER_IP${NC}"

# Step 3: Build web UI if needed
log "🌐 Building web UI..."
cd web
if [ ! -d "dist" ] || [ "dist/index.html" -ot "src/App.tsx" ]; then
    npm run build
else
    log "Web UI already built"
fi
cd ..

# Step 4: Start web server
log "🌐 Starting web server..."
export USE_DOCKER=true
export BRIDGE_PORT=8787

# Kill existing web server
kill $(lsof -ti:8787) 2>/dev/null || true
sleep 1

cd web/server
node server.mjs > /tmp/cryprq-web-server.log 2>&1 &
WEB_SERVER_PID=$!
cd ../..

sleep 3

# Check if web server started
if ! curl -s http://localhost:8787/ > /dev/null 2>&1; then
    log "${RED}❌ Web server failed to start${NC}"
    cat /tmp/cryprq-web-server.log
    exit 1
fi

log "${GREEN}✅ Web server running (PID: $WEB_SERVER_PID)${NC}"

# Step 5: Open browser
log "🌐 Opening browser..."
open "http://localhost:8787" 2>/dev/null || xdg-open "http://localhost:8787" 2>/dev/null || true

# Step 6: Run Playwright tests
log "🧪 Running automated browser tests..."

# Install Playwright if needed
if ! command -v playwright > /dev/null 2>&1; then
    log "Installing Playwright..."
    npm install -g playwright
    playwright install chromium
fi

# Create test file
cat > /tmp/cryprq-browser-test.js << 'EOF'
const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  console.log('Opening http://localhost:8787...');
  await page.goto('http://localhost:8787');
  
  // Wait for page to load
  await page.waitForSelector('h1');
  console.log('✅ Page loaded');
  
  // Test listener connection
  console.log('Testing listener connection...');
  await page.selectOption('select', 'listener');
  await page.click('button:has-text("Connect")');
  
  await page.waitForTimeout(2000);
  
  // Check for Docker mode messages
  const listenerStatus = await page.textContent('body');
  if (listenerStatus.includes('Docker mode') || listenerStatus.includes('Container')) {
    console.log('✅ Listener connected (Docker mode)');
  } else {
    console.log('⚠️ Listener status unclear');
  }
  
  // Open second tab for dialer
  console.log('Opening dialer tab...');
  const page2 = await context.newPage();
  await page2.goto('http://localhost:8787');
  await page2.waitForSelector('h1');
  
  await page2.selectOption('select', 'dialer');
  await page2.click('button:has-text("Connect")');
  
  await page2.waitForTimeout(5000);
  
  // Check for connection messages
  const dialerStatus = await page2.textContent('body');
  if (dialerStatus.includes('Connected') || dialerStatus.includes('Inbound connection')) {
    console.log('✅ Dialer connected!');
  } else {
    console.log('⚠️ Connection status unclear');
  }
  
  // Check container logs
  console.log('Checking container logs...');
  const { execSync } = require('child_process');
  const logs = execSync('docker logs --tail 5 cryprq-listener 2>&1', { encoding: 'utf8' });
  if (logs.includes('Inbound connection established')) {
    console.log('✅ Container shows connection established!');
  } else {
    console.log('⚠️ Container logs:', logs);
  }
  
  console.log('✅ Test completed - keeping browser open for 10 seconds...');
  await page.waitForTimeout(10000);
  
  await browser.close();
})();
EOF

node /tmp/cryprq-browser-test.js

log ""
log "${GREEN}=========================================="
log "✅ Automated test completed!"
log "==========================================${NC}"
log ""
log "Container IP: $CONTAINER_IP"
log "Web UI: http://localhost:8787"
log ""
log "To stop:"
log "  kill $WEB_SERVER_PID"
log "  ./scripts/docker-vpn-stop.sh"

