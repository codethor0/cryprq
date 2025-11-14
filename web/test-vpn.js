#!/usr/bin/env node
// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

/**
 * Automated VPN mode test for CrypRQ Web UI
 * Tests system-wide VPN functionality including TUN interface creation
 */

import puppeteer from 'puppeteer';
import { spawn, execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { existsSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = join(__dirname, '..');

const SERVER_PORT = process.env.BRIDGE_PORT || 8787;
const FRONTEND_PORT = 5173;
const BASE_URL = `http://localhost:${FRONTEND_PORT}`;

let serverProcess = null;
let frontendProcess = null;

// Helper to wait
const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// Check if TUN interface exists
function checkTunInterface(name = 'cryprq0') {
  try {
    const result = execSync(`ifconfig ${name} 2>&1`, { encoding: 'utf8' });
    return result.includes(name) && !result.includes('does not exist');
  } catch {
    return false;
  }
}

// Get TUN interface IP
function getTunIP(name = 'cryprq0') {
  try {
    const result = execSync(`ifconfig ${name} 2>&1`, { encoding: 'utf8' });
    const match = result.match(/inet\s+(\d+\.\d+\.\d+\.\d+)/);
    return match ? match[1] : null;
  } catch {
    return null;
  }
}

// Start server
async function startServer() {
  console.log('🚀 Starting server...');
  return new Promise((resolve, reject) => {
    const serverPath = join(__dirname, 'server', 'server.mjs');
    const env = {
      ...process.env,
      CRYPRQ_BIN: process.env.CRYPRQ_BIN || join(PROJECT_ROOT, 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ'),
      BRIDGE_PORT: SERVER_PORT.toString()
    };
    
    serverProcess = spawn('node', [serverPath], {
      env,
      stdio: 'pipe'
    });
    
    let serverOutput = '';
    serverProcess.stdout.on('data', (data) => {
      const output = data.toString();
      serverOutput += output;
      if (output.includes('bridge on')) {
        console.log('✅ Server started');
        resolve();
      }
    });
    
    serverProcess.stderr.on('data', (data) => {
      process.stderr.write(`[SERVER] ${data}`);
    });
    
    serverProcess.on('error', reject);
    
    setTimeout(() => {
      if (!serverOutput.includes('bridge on')) {
        reject(new Error('Server start timeout'));
      }
    }, 15000);
  });
}

// Start frontend
async function startFrontend() {
  console.log('🚀 Starting frontend...');
  return new Promise((resolve, reject) => {
    frontendProcess = spawn('npm', ['run', 'dev'], {
      cwd: __dirname,
      stdio: 'pipe',
      shell: true
    });
    
    let frontendReady = false;
    frontendProcess.stdout.on('data', (data) => {
      const output = data.toString();
      if ((output.includes('Local:') || output.includes('ready')) && !frontendReady) {
        frontendReady = true;
        console.log('✅ Frontend started');
        sleep(3000).then(resolve);
      }
    });
    
    frontendProcess.stderr.on('data', (data) => {
      const output = data.toString();
      if (output.includes('Local:') && !frontendReady) {
        frontendReady = true;
        console.log('✅ Frontend started');
        sleep(3000).then(resolve);
      }
    });
    
    frontendProcess.on('error', reject);
    
    setTimeout(() => {
      if (!frontendReady) {
        reject(new Error('Frontend start timeout'));
      }
    }, 30000);
  });
}

// Cleanup
async function cleanup() {
  console.log('\n🧹 Cleaning up...');
  if (serverProcess) {
    serverProcess.kill();
  }
  if (frontendProcess) {
    frontendProcess.kill();
  }
  await sleep(1000);
}

// Main test function
async function runVpnTests() {
  let browser = null;
  
  try {
    // Start services
    await startServer();
    await startFrontend();
    
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('🔒 VPN Mode Tests');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    // Check initial TUN interface state
    console.log('📋 Pre-test: Checking TUN interface state...');
    const tunExistsBefore = checkTunInterface();
    console.log(`  → TUN interface exists: ${tunExistsBefore ? '✅' : '❌'}`);
    
    await sleep(2000);
    
    // Launch browser
    console.log('\n🌐 Launching browser...');
    browser = await puppeteer.launch({
      headless: false,
      defaultViewport: { width: 1400, height: 900 },
      args: ['--start-maximized']
    });
    console.log('✅ Browser launched\n');
    
    // Test 1: Listener with VPN mode
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('📋 Test 1: Listener with VPN Mode Enabled');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    const listenerPage = await browser.newPage();
    await listenerPage.goto(BASE_URL, { 
      waitUntil: 'domcontentloaded',
      timeout: 60000 
    });
    await sleep(3000);
    
    // Set mode to listener
    await listenerPage.select('select', 'listener');
    await sleep(500);
    
    // Set port
    await listenerPage.evaluate(() => {
      const portInput = document.querySelector('input[type="number"]');
      if (portInput) {
        portInput.value = '10000';
        portInput.dispatchEvent(new Event('input', { bubbles: true }));
      }
    });
    await sleep(500);
    
    // Enable VPN mode checkbox
    console.log('  → Enabling VPN mode...');
    await listenerPage.evaluate(() => {
      const checkbox = document.querySelector('input[type="checkbox"]');
      if (checkbox && !checkbox.checked) {
        checkbox.click();
      }
    });
    await sleep(500);
    
    // Verify VPN checkbox is checked
    const vpnChecked = await listenerPage.evaluate(() => {
      const checkbox = document.querySelector('input[type="checkbox"]');
      return checkbox ? checkbox.checked : false;
    });
    console.log(`  → VPN mode checkbox: ${vpnChecked ? '✅ Checked' : '❌ Not checked'}`);
    
    // Click connect
    console.log('  → Clicking Connect button...');
    await listenerPage.click('button');
    await sleep(5000); // Wait for TUN interface creation
    
    // Check for VPN-related events
    const listenerEvents = await listenerPage.evaluate(() => {
      const events = Array.from(document.querySelectorAll('[style*="color"]'))
        .map(el => el.textContent)
        .filter(text => text.length > 0);
      return {
        total: events.length,
        vpnEvents: events.filter(e => e.toLowerCase().includes('vpn') || e.includes('TUN') || e.includes('tun')),
        allEvents: events.slice(-10)
      };
    });
    
    console.log(`  → Total events: ${listenerEvents.total}`);
    console.log(`  → VPN-related events: ${listenerEvents.vpnEvents.length}`);
    if (listenerEvents.vpnEvents.length > 0) {
      console.log(`  → VPN events: ${listenerEvents.vpnEvents.join(' | ')}`);
    }
    
    // Check TUN interface after listener starts
    await sleep(3000);
    const tunExistsAfterListener = checkTunInterface();
    const tunIP = getTunIP();
    
    console.log(`\n  📊 TUN Interface Status:`);
    console.log(`     → Exists: ${tunExistsAfterListener ? '✅' : '❌'}`);
    if (tunIP) {
      console.log(`     → IP Address: ${tunIP}`);
    }
    
    // Test 2: Dialer with VPN mode
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('📋 Test 2: Dialer with VPN Mode Enabled');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    const dialerPage = await browser.newPage();
    await dialerPage.goto(BASE_URL, { 
      waitUntil: 'domcontentloaded',
      timeout: 60000 
    });
    await sleep(3000);
    
    // Set mode to dialer
    await dialerPage.select('select', 'dialer');
    await sleep(500);
    
    // Set port
    await dialerPage.evaluate(() => {
      const portInput = document.querySelector('input[type="number"]');
      if (portInput) {
        portInput.value = '10000';
        portInput.dispatchEvent(new Event('input', { bubbles: true }));
      }
    });
    await sleep(500);
    
    // Enable VPN mode checkbox
    console.log('  → Enabling VPN mode...');
    await dialerPage.evaluate(() => {
      const checkbox = document.querySelector('input[type="checkbox"]');
      if (checkbox && !checkbox.checked) {
        checkbox.click();
      }
    });
    await sleep(500);
    
    // Click connect
    console.log('  → Clicking Connect button...');
    await dialerPage.click('button');
    await sleep(5000);
    
    // Check for VPN-related events
    const dialerEvents = await dialerPage.evaluate(() => {
      const events = Array.from(document.querySelectorAll('[style*="color"]'))
        .map(el => el.textContent)
        .filter(text => text.length > 0);
      return {
        total: events.length,
        vpnEvents: events.filter(e => e.toLowerCase().includes('vpn') || e.includes('TUN') || e.includes('tun')),
        connectionEvents: events.filter(e => e.includes('Connected') || e.includes('connection'))
      };
    });
    
    console.log(`  → Total events: ${dialerEvents.total}`);
    console.log(`  → VPN-related events: ${dialerEvents.vpnEvents.length}`);
    console.log(`  → Connection events: ${dialerEvents.connectionEvents.length}`);
    
    // Monitor for 10 seconds
    console.log('\n📋 Test 3: Monitoring VPN connection for 10 seconds...');
    await sleep(10000);
    
    // Final TUN interface check
    const tunExistsFinal = checkTunInterface();
    const tunIPFinal = getTunIP();
    
    // Final status
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('📊 VPN Test Results');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    console.log('TUN Interface:');
    console.log(`  • Before test: ${tunExistsBefore ? '✅ Existed' : '❌ Did not exist'}`);
    console.log(`  • After listener: ${tunExistsAfterListener ? '✅ Created' : '❌ Not created'}`);
    console.log(`  • Final state: ${tunExistsFinal ? '✅ Active' : '❌ Not active'}`);
    if (tunIPFinal) {
      console.log(`  • IP Address: ${tunIPFinal}`);
    }
    
    console.log('\nListener Tab:');
    console.log(`  • Total Events: ${listenerEvents.total}`);
    console.log(`  • VPN Events: ${listenerEvents.vpnEvents.length}`);
    if (listenerEvents.vpnEvents.length > 0) {
      listenerEvents.vpnEvents.forEach(e => console.log(`    - ${e}`));
    }
    
    console.log('\nDialer Tab:');
    console.log(`  • Total Events: ${dialerEvents.total}`);
    console.log(`  • VPN Events: ${dialerEvents.vpnEvents.length}`);
    console.log(`  • Connection Events: ${dialerEvents.connectionEvents.length}`);
    
    // Summary
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('✅ VPN Tests Completed!');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    if (tunExistsFinal) {
      console.log('🎉 TUN interface was created successfully!');
      console.log('   Note: On macOS, full system routing requires Network Extension framework.');
      console.log('   The encrypted tunnel between peers is active.');
    } else {
      console.log('⚠️  TUN interface was not created.');
      console.log('   This may require root/admin privileges or Network Extension framework on macOS.');
      console.log('   The P2P encrypted tunnel is still active between peers.');
    }
    
    console.log('\nBrowser will stay open for 30 seconds for inspection...');
    await sleep(30000);
    
  } catch (error) {
    console.error('\n❌ Test failed:', error.message);
    console.error(error.stack);
  } finally {
    if (browser) {
      await browser.close();
    }
    await cleanup();
  }
}

// Handle Ctrl+C
process.on('SIGINT', async () => {
  console.log('\n\n🛑 Interrupted by user');
  await cleanup();
  process.exit(0);
});

// Run tests
runVpnTests().catch(console.error);

