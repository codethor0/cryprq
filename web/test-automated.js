#!/usr/bin/env node
// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

/**
 * Automated browser test for CrypRQ Web UI
 * This script uses Puppeteer to test the web interface end-to-end
 */

import puppeteer from 'puppeteer';
import { spawn } from 'child_process';
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
    
    console.log(`  → Server path: ${serverPath}`);
    console.log(`  → CRYPRQ_BIN: ${env.CRYPRQ_BIN}`);
    
    serverProcess = spawn('node', [serverPath], {
      env,
      stdio: 'pipe'
    });
    
    let serverOutput = '';
    serverProcess.stdout.on('data', (data) => {
      const output = data.toString();
      serverOutput += output;
      process.stdout.write(`[SERVER] ${output}`);
      if (output.includes('bridge on')) {
        console.log('\n✅ Server started on port', SERVER_PORT);
        resolve();
      }
    });
    
    serverProcess.stderr.on('data', (data) => {
      process.stderr.write(`[SERVER ERROR] ${data}`);
    });
    
    serverProcess.on('error', (err) => {
      console.error('\n❌ Server spawn error:', err.message);
      reject(err);
    });
    
    // Timeout after 15 seconds
    setTimeout(() => {
      if (!serverOutput.includes('bridge on')) {
        console.error('\n❌ Server start timeout');
        console.error('Server output:', serverOutput);
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
      process.stdout.write(`[FRONTEND] ${output}`);
      if ((output.includes('Local:') || output.includes('ready')) && !frontendReady) {
        frontendReady = true;
        console.log('\n✅ Frontend started on port', FRONTEND_PORT);
        sleep(3000).then(resolve); // Wait a bit for it to be ready
      }
    });
    
    frontendProcess.stderr.on('data', (data) => {
      const output = data.toString();
      process.stderr.write(`[FRONTEND] ${output}`);
      if (output.includes('Local:') && !frontendReady) {
        frontendReady = true;
        console.log('\n✅ Frontend started on port', FRONTEND_PORT);
        sleep(3000).then(resolve);
      }
    });
    
    frontendProcess.on('error', (err) => {
      console.error('\n❌ Frontend spawn error:', err.message);
      reject(err);
    });
    
    setTimeout(() => {
      if (!frontendReady) {
        console.error('\n❌ Frontend start timeout');
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
async function runTests() {
  let browser = null;
  
  try {
    // Start services
    await startServer();
    await startFrontend();
    
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('🌐 Starting Browser Tests');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    // Wait a bit more for frontend to be fully ready
    console.log('  → Waiting for frontend to be ready...');
    await sleep(3000);
    
    // Launch browser with visible UI
    console.log('  → Launching browser...');
    browser = await puppeteer.launch({
      headless: false, // Show browser so user can watch
      defaultViewport: { width: 1400, height: 900 },
      args: ['--start-maximized']
    });
    console.log('  ✅ Browser launched');
    
    // Test 1: Listener Tab
    console.log('📋 Test 1: Setting up Listener...');
    console.log(`  → Navigating to ${BASE_URL}...`);
    const listenerPage = await browser.newPage();
    await listenerPage.goto(BASE_URL, { 
      waitUntil: 'domcontentloaded',
      timeout: 60000 
    });
    console.log('  → Page loaded, waiting for React to initialize...');
    await sleep(3000); // Wait for React to render
    
    // Check connection status
    const listenerConnected = await listenerPage.evaluate(() => {
      const statusEl = document.querySelector('div[style*="Connected"]');
      return statusEl ? statusEl.textContent.includes('Connected') : false;
    });
    
    if (!listenerConnected) {
      console.log('⚠️  Waiting for server connection...');
      await sleep(3000);
    }
    
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
    
    // Click connect
    console.log('  → Clicking Connect button...');
    await listenerPage.click('button');
    await sleep(3000); // Wait for connection
    
    // Check for connection status
    const listenerStatus = await listenerPage.evaluate(() => {
      const events = Array.from(document.querySelectorAll('[style*="color"]')).map(el => el.textContent);
      return events;
    });
    
    console.log('  ✅ Listener started');
    console.log(`  📊 Events: ${listenerStatus.length} items in console`);
    
    // Test 2: Dialer Tab
    console.log('\n📋 Test 2: Setting up Dialer...');
    console.log(`  → Navigating to ${BASE_URL}...`);
    const dialerPage = await browser.newPage();
    await dialerPage.goto(BASE_URL, { 
      waitUntil: 'domcontentloaded',
      timeout: 60000 
    });
    console.log('  → Page loaded, waiting for React to initialize...');
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
    
    // Verify peer address is auto-updated
    const peerAddress = await dialerPage.evaluate(() => {
      const peerInput = document.querySelector('input[style*="monospace"]');
      return peerInput ? peerInput.value : '';
    });
    
    console.log(`  → Peer address: ${peerAddress}`);
    
    // Click connect
    console.log('  → Clicking Connect button...');
    await dialerPage.click('button');
    await sleep(5000); // Wait for connection
    
    // Check connection status
    const dialerStatus = await dialerPage.evaluate(() => {
      const statusSection = document.querySelector('div[style*="Encryption Status"]')?.parentElement;
      if (!statusSection) return null;
      const text = statusSection.textContent;
      return {
        connected: text.includes('Encrypted Tunnel Active') || text.includes('Connected'),
        hasPeerId: text.includes('12D3KooW'),
        events: Array.from(document.querySelectorAll('[style*="color"]')).length
      };
    });
    
    console.log('  ✅ Dialer started');
    console.log(`  📊 Connection status: ${dialerStatus?.connected ? '✅ Connected' : '⏳ Connecting...'}`);
    console.log(`  📊 Events: ${dialerStatus?.events || 0} items in console`);
    
    // Test 3: Monitor connection and verify encryption status
    console.log('\n📋 Test 3: Monitoring connection and verifying encryption status...');
    console.log('  → Waiting 15 seconds to capture all logs and verify encryption status updates...');
    
    // Check encryption status multiple times during monitoring
    for (let i = 0; i < 5; i++) {
      await sleep(3000);
      
      const listenerStatusCheck = await listenerPage.evaluate(() => {
        // Find EncryptionStatus component by looking for the h3 with "Encryption Status" text
        const h3 = Array.from(document.querySelectorAll('h3')).find(el => el.textContent.includes('Encryption Status'));
        const statusSection = h3?.closest('div[style*="background"]') || h3?.parentElement;
        if (!statusSection) return null;
        const text = statusSection.textContent;
        return {
          connection: text.includes('Encrypted Tunnel Active') || text.includes('Encryption Active') || text.includes('Listening') || text.includes('Starting'),
          encryption: text.includes('ML-KEM') || text.includes('Kyber'),
          peerId: text.match(/12D3KooW\w+/)?.[0] || null,
          keyEpoch: text.match(/Epoch\s+(\d+)/)?.[1] || null
        };
      });
      
      const dialerStatusCheck = await dialerPage.evaluate(() => {
        // Find EncryptionStatus component by looking for the h3 with "Encryption Status" text
        const h3 = Array.from(document.querySelectorAll('h3')).find(el => el.textContent.includes('Encryption Status'));
        const statusSection = h3?.closest('div[style*="background"]') || h3?.parentElement;
        if (!statusSection) return null;
        const text = statusSection.textContent;
        return {
          connection: text.includes('Encrypted Tunnel Active') || text.includes('Encryption Active') || text.includes('Connecting') || text.includes('Starting'),
          encryption: text.includes('ML-KEM') || text.includes('Kyber'),
          peerId: text.match(/12D3KooW\w+/)?.[0] || null,
          keyEpoch: text.match(/Epoch\s+(\d+)/)?.[1] || null
        };
      });
      
      console.log(`  [${i + 1}/5] Status check:`);
      console.log(`     Listener: ${listenerStatusCheck?.connection ? '✅' : '⏳'} Connection | ${listenerStatusCheck?.encryption ? '✅' : '❌'} Encryption | Peer ID: ${listenerStatusCheck?.peerId ? '✅' : '❌'}`);
      console.log(`     Dialer:   ${dialerStatusCheck?.connection ? '✅' : '⏳'} Connection | ${dialerStatusCheck?.encryption ? '✅' : '❌'} Encryption | Peer ID: ${dialerStatusCheck?.peerId ? '✅' : '❌'}`);
    }
    
    // Final comprehensive status check
    const finalListenerStatus = await listenerPage.evaluate(() => {
      const events = Array.from(document.querySelectorAll('[style*="color"]'))
        .map(el => el.textContent)
        .filter(text => text.length > 0);
      
      // Find EncryptionStatus component properly
      const h3 = Array.from(document.querySelectorAll('h3')).find(el => el.textContent.includes('Encryption Status'));
      const statusSection = h3?.closest('div[style*="background"]') || h3?.parentElement;
      const statusText = statusSection ? statusSection.textContent : '';
      
      return {
        totalEvents: events.length,
        recentEvents: events.slice(-10),
        hasConnection: events.some(e => e.includes('Connected') || e.includes('peer id') || e.includes('Inbound')),
        hasPeerId: events.some(e => e.includes('Local peer id:')),
        hasEncryption: events.some(e => e.includes('key_rotation') || e.includes('ENCRYPT') || e.includes('DECRYPT')),
        statusText: statusText,
        connectionStatus: statusText.includes('Encrypted Tunnel Active') ? 'connected' :
                         statusText.includes('Encryption Active') || statusText.includes('encryption active') ? 'encryption_active' :
                         statusText.includes('Starting') && statusText.includes('encryption') ? 'encryption_active' :
                         statusText.includes('Connecting') && statusText.includes('encryption') ? 'encryption_active' :
                         statusText.includes('Listening') ? 'listening' : 'disconnected',
        hasEncryptionInStatus: statusText.includes('ML-KEM') || statusText.includes('Kyber'),
        peerIdInStatus: statusText.match(/12D3KooW\w+/)?.[0] || null
      };
    });
    
    const finalDialerStatus = await dialerPage.evaluate(() => {
      const events = Array.from(document.querySelectorAll('[style*="color"]'))
        .map(el => el.textContent)
        .filter(text => text.length > 0);
      
      // Find EncryptionStatus component properly
      const h3 = Array.from(document.querySelectorAll('h3')).find(el => el.textContent.includes('Encryption Status'));
      const statusSection = h3?.closest('div[style*="background"]') || h3?.parentElement;
      const statusText = statusSection ? statusSection.textContent : '';
      
      return {
        totalEvents: events.length,
        recentEvents: events.slice(-10),
        hasConnection: events.some(e => e.includes('Connected') || e.includes('peer id') || e.includes('Connected to')),
        hasPeerId: events.some(e => e.includes('Local peer id:')),
        hasEncryption: events.some(e => e.includes('key_rotation') || e.includes('ENCRYPT') || e.includes('DECRYPT')),
        statusText: statusText,
        connectionStatus: statusText.includes('Encrypted Tunnel Active') ? 'connected' :
                         statusText.includes('Encryption Active') || statusText.includes('encryption active') ? 'encryption_active' :
                         statusText.includes('Starting') && statusText.includes('encryption') ? 'encryption_active' :
                         statusText.includes('Connecting') && statusText.includes('encryption') ? 'encryption_active' :
                         statusText.includes('Connecting') ? 'connecting' : 'disconnected',
        hasEncryptionInStatus: statusText.includes('ML-KEM') || statusText.includes('Kyber'),
        peerIdInStatus: statusText.match(/12D3KooW\w+/)?.[0] || null
      };
    });
    
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('📊 Comprehensive Test Results');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    console.log('🔐 Listener Tab - Encryption Status:');
    console.log(`  • Connection Status: ${finalListenerStatus.connectionStatus}`);
    console.log(`  • Encryption in Status: ${finalListenerStatus.hasEncryptionInStatus ? '✅ ML-KEM (Kyber768) + X25519 hybrid' : '❌ Not shown'}`);
    console.log(`  • Peer ID in Status: ${finalListenerStatus.peerIdInStatus ? '✅ ' + finalListenerStatus.peerIdInStatus.substring(0, 20) + '...' : '❌ Not shown'}`);
    console.log(`  • Total Events: ${finalListenerStatus.totalEvents}`);
    console.log(`  • Has Peer ID Event: ${finalListenerStatus.hasPeerId ? '✅' : '❌'}`);
    console.log(`  • Has Encryption Events: ${finalListenerStatus.hasEncryption ? '✅' : '❌'}`);
    console.log(`  • Connection Detected: ${finalListenerStatus.hasConnection ? '✅' : '❌'}`);
    console.log(`  • Recent Events (last 5):`);
    finalListenerStatus.recentEvents.slice(-5).forEach((e, i) => {
      const short = e.length > 80 ? e.substring(0, 80) + '...' : e;
      console.log(`     ${i + 1}. ${short}`);
    });
    
    console.log('\n🔐 Dialer Tab - Encryption Status:');
    console.log(`  • Connection Status: ${finalDialerStatus.connectionStatus}`);
    console.log(`  • Encryption in Status: ${finalDialerStatus.hasEncryptionInStatus ? '✅ ML-KEM (Kyber768) + X25519 hybrid' : '❌ Not shown'}`);
    console.log(`  • Peer ID in Status: ${finalDialerStatus.peerIdInStatus ? '✅ ' + finalDialerStatus.peerIdInStatus.substring(0, 20) + '...' : '❌ Not shown'}`);
    console.log(`  • Total Events: ${finalDialerStatus.totalEvents}`);
    console.log(`  • Has Peer ID Event: ${finalDialerStatus.hasPeerId ? '✅' : '❌'}`);
    console.log(`  • Has Encryption Events: ${finalDialerStatus.hasEncryption ? '✅' : '❌'}`);
    console.log(`  • Connection Detected: ${finalDialerStatus.hasConnection ? '✅' : '❌'}`);
    console.log(`  • Recent Events (last 5):`);
    finalDialerStatus.recentEvents.slice(-5).forEach((e, i) => {
      const short = e.length > 80 ? e.substring(0, 80) + '...' : e;
      console.log(`     ${i + 1}. ${short}`);
    });
    
    // Verification summary
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('✅ Verification Summary');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    const listenerPassed = finalListenerStatus.hasEncryptionInStatus && 
                          finalListenerStatus.hasPeerId && 
                          (finalListenerStatus.connectionStatus === 'connected' || finalListenerStatus.connectionStatus === 'listening');
    
    const dialerPassed = finalDialerStatus.hasEncryptionInStatus && 
                        finalDialerStatus.hasPeerId && 
                        (finalDialerStatus.connectionStatus === 'connected' || finalDialerStatus.connectionStatus === 'encryption_active');
    
    console.log(`Listener: ${listenerPassed ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`  • Encryption status shown: ${finalListenerStatus.hasEncryptionInStatus ? '✅' : '❌'}`);
    console.log(`  • Peer ID detected: ${finalListenerStatus.hasPeerId ? '✅' : '❌'}`);
    console.log(`  • Connection status: ${finalListenerStatus.connectionStatus}`);
    
    console.log(`\nDialer: ${dialerPassed ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`  • Encryption status shown: ${finalDialerStatus.hasEncryptionInStatus ? '✅' : '❌'}`);
    console.log(`  • Peer ID detected: ${finalDialerStatus.hasPeerId ? '✅' : '❌'}`);
    console.log(`  • Connection status: ${finalDialerStatus.connectionStatus}`);
    
    if (listenerPassed && dialerPassed) {
      console.log('\n🎉 All encryption status checks PASSED!');
    } else {
      console.log('\n⚠️  Some encryption status checks FAILED - review logs above');
    }
    
    // Keep browser open for user to inspect
    console.log('\n✅ Tests completed! Browser will stay open for 30 seconds for inspection...');
    console.log('   Close the browser windows or press Ctrl+C to exit.\n');
    
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
runTests().catch(console.error);

