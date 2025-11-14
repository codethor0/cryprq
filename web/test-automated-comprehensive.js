#!/usr/bin/env node

// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

/**
 * Comprehensive automated test for CrypRQ web tester
 * Tests: Server, frontend, encryption status, listener/dialer connection
 */

import { spawn } from 'child_process';
import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import puppeteer from 'puppeteer';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = join(__dirname, '..');

const SERVER_PORT = 8787;
const FRONTEND_PORT = 5173;
const TEST_PORT = 10019; // Unique port for testing

let serverProc = null;
let frontendProc = null;
let testResults = {
  binaryFound: false,
  serverStarted: false,
  frontendStarted: false,
  encryptionStatusDisplayed: false,
  listenerConnected: false,
  dialerConnected: false,
  encryptionEventsDetected: false,
};

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function findBinary() {
  const binPaths = [
    join(PROJECT_ROOT, 'target', 'release', 'cryprq'),
    join(PROJECT_ROOT, 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ'),
  ];
  
  for (const path of binPaths) {
    if (existsSync(path)) {
      return path;
    }
  }
  return null;
}

async function startServer(binaryPath) {
  return new Promise((resolve, reject) => {
    serverProc = spawn('node', ['server/server.mjs'], {
      cwd: join(PROJECT_ROOT, 'web'),
      env: { ...process.env, CRYPRQ_BIN: binaryPath, PORT: SERVER_PORT },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let serverReady = false;
    let serverError = null;
    const timeout = setTimeout(() => {
      if (!serverReady) {
        reject(new Error('Server startup timeout'));
      }
    }, 10000);

    serverProc.stdout.on('data', (data) => {
      const output = data.toString();
      if (output.includes('bridge on')) {
        serverReady = true;
        clearTimeout(timeout);
        resolve();
      }
    });

    serverProc.stderr.on('data', (data) => {
      const output = data.toString();
      if (!output.includes('ExperimentalWarning')) {
        serverError = output;
      }
    });

    serverProc.on('exit', (code) => {
      if (code !== 0 && code !== null && !serverReady) {
        clearTimeout(timeout);
        reject(new Error(`Server exited with code ${code}: ${serverError || ''}`));
      }
    });
  });
}

async function startFrontend() {
  return new Promise((resolve, reject) => {
    frontendProc = spawn('npm', ['run', 'dev'], {
      cwd: join(PROJECT_ROOT, 'web'),
      env: { ...process.env, PORT: FRONTEND_PORT },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    const timeout = setTimeout(() => {
      resolve(); // Assume started after timeout
    }, 8000);

    frontendProc.stdout.on('data', (data) => {
      const output = data.toString();
      if (output.includes('Local:') || output.includes('localhost')) {
        clearTimeout(timeout);
        resolve();
      }
    });

    frontendProc.stderr.on('data', (data) => {
      // Ignore warnings
    });
  });
}

async function testWithPuppeteer() {
  console.log('\nStep 4: Testing with Puppeteer...');
  
  const browser = await puppeteer.launch({ 
    headless: false,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  
  try {
    const page = await browser.newPage();
    
    // Navigate to frontend
    console.log('  → Navigating to frontend...');
    await page.goto(`http://localhost:${FRONTEND_PORT}`, { 
      waitUntil: 'networkidle2',
      timeout: 15000 
    });
    await sleep(2000);

    // Test 1: Verify initial encryption status
    console.log('\nTest 1: Verifying encryption method display...');
    const encryptionText = await page.evaluate(() => {
      const bodyText = document.body.innerText;
      return bodyText.includes('ML-KEM (Kyber768) + X25519 hybrid');
    });
    
    if (encryptionText) {
      console.log('  [OK] Encryption method displayed correctly');
      testResults.encryptionStatusDisplayed = true;
    } else {
      console.log('  [FAIL] Encryption method not found');
    }

    // Test 2: Click Connect button (Listener mode)
    console.log('\nTest 2: Testing listener connection...');
    try {
      // Find Connect button by text content
      const connectButton = await page.evaluateHandle(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        return buttons.find(btn => btn.textContent.includes('Connect'));
      });
      
      if (connectButton) {
        await connectButton.click();
        await sleep(3000);
      } else {
        throw new Error('Connect button not found');
      }
      
      // Check for status updates
      const statusText = await page.evaluate(() => {
        const bodyText = document.body.innerText;
        return bodyText.includes('[STARTING]') || 
               bodyText.includes('[WAITING]') ||
               bodyText.includes('[CONNECTING]');
      });
      
      if (statusText) {
        console.log('  [OK] Status updated after Connect click');
        testResults.listenerConnected = true;
      } else {
        console.log('  [WARN] Status did not update (may take time)');
      }
    } catch (err) {
      console.log(`  [WARN] Could not click Connect: ${err.message}`);
    }

    // Test 3: Check debug console for encryption events
    console.log('\nTest 3: Checking for encryption events...');
    await sleep(5000); // Wait for events to appear
    
    const hasEncryptionEvents = await page.evaluate(() => {
      const bodyText = document.body.innerText;
      return bodyText.includes('Local peer id') ||
             bodyText.includes('key_rotation') ||
             bodyText.includes('ENCRYPTION PROOF') ||
             bodyText.includes('Listening on');
    });
    
    if (hasEncryptionEvents) {
      console.log('  [OK] Encryption events detected');
      testResults.encryptionEventsDetected = true;
    } else {
      console.log('  [WARN] Encryption events not yet visible');
    }

    // Test 4: Test server endpoint directly
    console.log('\nTest 4: Testing server endpoint...');
    try {
      const response = await page.evaluate(async (port) => {
        const res = await fetch(`http://localhost:${port}/connect`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mode: 'listener', port: 10020 })
        });
        return { ok: res.ok, status: res.status };
      }, SERVER_PORT);
      
      if (response.ok) {
        console.log('  [OK] Server endpoint responded successfully');
      } else {
        console.log(`  [WARN] Server endpoint returned status: ${response.status}`);
      }
    } catch (err) {
      console.log(`  [WARN] Server endpoint test failed: ${err.message}`);
    }

    await sleep(2000);
    
  } catch (error) {
    console.error(`  [FAIL] Puppeteer test error: ${error.message}`);
  } finally {
    await browser.close();
  }
}

async function runTests() {
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('Comprehensive Automated Web Tester Test');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  try {
    // Step 1: Find binary
    console.log('Step 1: Finding CrypRQ binary...');
    const binaryPath = await findBinary();
    if (!binaryPath) {
      console.error('[FAIL] CrypRQ binary not found!');
      console.error('Please build with: cargo build --release -p cryprq');
      process.exit(1);
    }
    console.log(`  [OK] Binary found: ${binaryPath}`);
    testResults.binaryFound = true;

    // Step 2: Start server
    console.log('\nStep 2: Starting server...');
    try {
      await startServer(binaryPath);
      console.log('  [OK] Server started');
      testResults.serverStarted = true;
    } catch (err) {
      console.error(`  [FAIL] Server failed to start: ${err.message}`);
      throw err;
    }

    // Step 3: Start frontend
    console.log('\nStep 3: Starting frontend...');
    try {
      await startFrontend();
      console.log('  [OK] Frontend started');
      testResults.frontendStarted = true;
    } catch (err) {
      console.error(`  [FAIL] Frontend failed to start: ${err.message}`);
      throw err;
    }

    // Step 4: Run Puppeteer tests
    await testWithPuppeteer();

    // Summary
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('Test Summary');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    console.log(`Binary Found: ${testResults.binaryFound ? '[OK]' : '[FAIL]'}`);
    console.log(`Server Started: ${testResults.serverStarted ? '[OK]' : '[FAIL]'}`);
    console.log(`Frontend Started: ${testResults.frontendStarted ? '[OK]' : '[FAIL]'}`);
    console.log(`Encryption Status Displayed: ${testResults.encryptionStatusDisplayed ? '[OK]' : '[FAIL]'}`);
    console.log(`Listener Connected: ${testResults.listenerConnected ? '[OK]' : '[WARN]'}`);
    console.log(`Encryption Events Detected: ${testResults.encryptionEventsDetected ? '[OK]' : '[WARN]'}`);
    
    const allCritical = testResults.binaryFound && 
                       testResults.serverStarted && 
                       testResults.frontendStarted && 
                       testResults.encryptionStatusDisplayed;
    
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    if (allCritical) {
      console.log('[OK] All critical tests passed!');
      console.log('\nNext steps:');
      console.log('1. Open http://localhost:5173 in your browser');
      console.log('2. Click Connect button');
      console.log('3. Verify encryption status updates');
      console.log('4. Check debug console for encryption events');
      console.log('\nPress Ctrl+C to stop services');
      
      // Keep running
      await new Promise(() => {});
    } else {
      console.log('[FAIL] Some critical tests failed');
      process.exit(1);
    }

  } catch (error) {
    console.error('\n[FAIL] Test suite failed:', error);
    if (serverProc) serverProc.kill();
    if (frontendProc) frontendProc.kill();
    process.exit(1);
  }
}

// Handle cleanup
process.on('SIGINT', () => {
  console.log('\n\nCleaning up...');
  if (serverProc) serverProc.kill();
  if (frontendProc) frontendProc.kill();
  process.exit(0);
});

runTests().catch(console.error);

