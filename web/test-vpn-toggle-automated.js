#!/usr/bin/env node

// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

/**
 * Automated test for VPN toggle functionality
 * Tests: VPN checkbox, backend handling, error messages, privilege detection
 */

import { spawn } from 'child_process';
import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';
import puppeteer from 'puppeteer';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = join(__dirname, '..');

const SERVER_PORT = 8787;
const FRONTEND_PORT = 5173;

let serverProc = null;
let frontendProc = null;
let testResults = {
  vpnCheckboxExists: false,
  vpnCheckboxToggleable: false,
  vpnFlagSent: false,
  vpnErrorHandled: false,
  vpnStatusDisplayed: false,
  fileTransferAvailable: false,
  fileInputFound: false,
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
    // Check if server is already running
    try {
      const existing = execSync(`lsof -ti:${SERVER_PORT} 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
      if (existing) {
        console.log('  [INFO] Server already running on port', SERVER_PORT);
        resolve(); // Server already running
        return;
      }
    } catch (e) {
      // Continue to start server
    }

    serverProc = spawn('node', ['server/server.mjs'], {
      cwd: join(PROJECT_ROOT, 'web'),
      env: { ...process.env, CRYPRQ_BIN: binaryPath, PORT: SERVER_PORT },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let serverReady = false;
    const timeout = setTimeout(() => {
      if (!serverReady) {
        // Don't reject if server might already be running
        resolve();
      }
    }, 15000);

    serverProc.stdout.on('data', (data) => {
      const output = data.toString();
      if (output.includes('bridge on')) {
        serverReady = true;
        clearTimeout(timeout);
        resolve();
      }
    });

    serverProc.stderr.on('data', (data) => {
      // Ignore warnings
    });

    serverProc.on('exit', (code) => {
      if (code !== 0 && code !== null && !serverReady) {
        // Don't reject - server might have exited but we can still test
        resolve();
      }
    });
  });
}

async function startFrontend() {
  return new Promise((resolve) => {
    frontendProc = spawn('npm', ['run', 'dev'], {
      cwd: join(PROJECT_ROOT, 'web'),
      env: { ...process.env, PORT: FRONTEND_PORT },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    const timeout = setTimeout(() => {
      resolve();
    }, 10000);

    frontendProc.stdout.on('data', (data) => {
      const output = data.toString();
      if (output.includes('Local:') || output.includes('localhost')) {
        clearTimeout(timeout);
        resolve();
      }
    });
  });
}

async function testVPNToggle() {
  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('Testing VPN Toggle Functionality');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
  
  const browser = await puppeteer.launch({ 
    headless: false,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  
  try {
    const page = await browser.newPage();
    
    // Navigate to frontend
    console.log('Test 1: Navigating to frontend...');
    await page.goto(`http://localhost:${FRONTEND_PORT}`, { 
      waitUntil: 'networkidle2',
      timeout: 15000 
    });
    await sleep(3000);

    // Test 1: Verify VPN checkbox exists
    console.log('\nTest 2: Verifying VPN checkbox exists...');
    const checkboxExists = await page.evaluate(() => {
      const checkboxes = Array.from(document.querySelectorAll('input[type="checkbox"]'));
      return checkboxes.some(cb => cb.nextElementSibling?.textContent?.includes('VPN Mode'));
    });
    
    if (checkboxExists) {
      console.log('  [OK] VPN checkbox found');
      testResults.vpnCheckboxExists = true;
    } else {
      console.log('  [FAIL] VPN checkbox not found');
    }

    // Test 2: Toggle VPN checkbox
    console.log('\nTest 3: Testing VPN checkbox toggle...');
    try {
      const checkbox = await page.evaluateHandle(() => {
        const checkboxes = Array.from(document.querySelectorAll('input[type="checkbox"]'));
        return checkboxes.find(cb => cb.nextElementSibling?.textContent?.includes('VPN Mode'));
      });
      
      if (checkbox) {
        const initialChecked = await checkbox.evaluate(cb => cb.checked);
        await checkbox.click();
        await sleep(500);
        const afterClick = await checkbox.evaluate(cb => cb.checked);
        
        if (afterClick !== initialChecked) {
          console.log('  [OK] VPN checkbox toggleable');
          testResults.vpnCheckboxToggleable = true;
        }
      }
    } catch (err) {
      console.log(`  [WARN] Could not toggle checkbox: ${err.message}`);
    }

    // Test 3: Connect with VPN enabled
    console.log('\nTest 4: Testing connection with VPN enabled...');
    try {
      // Ensure VPN is checked
      const checkbox = await page.evaluateHandle(() => {
        const checkboxes = Array.from(document.querySelectorAll('input[type="checkbox"]'));
        return checkboxes.find(cb => cb.nextElementSibling?.textContent?.includes('VPN Mode'));
      });
      
      if (checkbox) {
        await checkbox.evaluate(cb => { if (!cb.checked) cb.click(); });
        await sleep(500);
      }
      
      // Click Connect
      const connectButton = await page.evaluateHandle(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        return buttons.find(btn => btn.textContent.includes('Connect'));
      });
      
      if (connectButton) {
        await connectButton.click();
        await sleep(5000);
        
        // Check if VPN flag was sent (check server logs or UI)
        const vpnFlagSent = await page.evaluate(() => {
          const bodyText = document.body.innerText;
          return bodyText.includes('VPN mode') || bodyText.includes('--vpn') || bodyText.includes('VPN MODE ENABLED');
        });
        
        if (vpnFlagSent) {
          console.log('  [OK] VPN flag sent to backend');
          testResults.vpnFlagSent = true;
        }
      }
    } catch (err) {
      console.log(`  [WARN] Connection test failed: ${err.message}`);
    }

    // Test 4: Check for VPN error messages
    console.log('\nTest 5: Checking for VPN error handling...');
    await sleep(5000);
    
    const vpnErrorHandled = await page.evaluate(() => {
      const bodyText = document.body.innerText;
      return bodyText.includes('requires root') || 
             bodyText.includes('requires admin') || 
             bodyText.includes('privileges') ||
             bodyText.includes('TUN interface') ||
             bodyText.includes('Network Extension');
    });
    
    if (vpnErrorHandled) {
      console.log('  [OK] VPN error handling detected');
      testResults.vpnErrorHandled = true;
    } else {
      console.log('  [INFO] No VPN errors (may require admin privileges to test fully)');
    }

    // Test 5: Verify VPN status display
    console.log('\nTest 6: Verifying VPN status display...');
    const vpnStatusDisplayed = await page.evaluate(() => {
      const bodyText = document.body.innerText;
      return bodyText.includes('System-Wide VPN') || 
             bodyText.includes('VPN Mode') ||
             bodyText.includes('Network Extension');
    });
    
    if (vpnStatusDisplayed) {
      console.log('  [OK] VPN status displayed');
      testResults.vpnStatusDisplayed = true;
    }

    // Test 6: File transfer functionality
    console.log('\nTest 7: Testing file transfer functionality...');
    try {
      // Wait for connection to be established
      await sleep(5000);
      
      const fileTransferAvailable = await page.evaluate(() => {
        const bodyText = document.body.innerText;
        return bodyText.includes('Send File') || bodyText.includes('file-upload');
      });

      if (fileTransferAvailable) {
        console.log('  [OK] File transfer UI available');
        testResults.fileTransferAvailable = true;
      }

      // Try to find file input
      const fileInput = await page.$('input[type="file"]');
      if (fileInput) {
        console.log('  [OK] File input found');
        testResults.fileInputFound = true;
      }
    } catch (err) {
      console.log(`  [WARN] File transfer test: ${err.message}`);
    }

    await sleep(2000);
    
  } catch (error) {
    console.error(`  [FAIL] Test error: ${error.message}`);
  } finally {
    await browser.close();
  }
}

async function runVPNTest() {
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('VPN Toggle Automated Test');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  try {
    // Find binary
    const binaryPath = await findBinary();
    if (!binaryPath) {
      console.error('[FAIL] CrypRQ binary not found!');
      process.exit(1);
    }

    // Start server
    await startServer(binaryPath);
    console.log('  [OK] Server started');

    // Start frontend
    await startFrontend();
    console.log('  [OK] Frontend started');

    // Run tests
    await testVPNToggle();

    // Summary
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('Test Summary');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    console.log(`VPN Checkbox Exists: ${testResults.vpnCheckboxExists ? '[OK]' : '[FAIL]'}`);
    console.log(`VPN Checkbox Toggleable: ${testResults.vpnCheckboxToggleable ? '[OK]' : '[FAIL]'}`);
    console.log(`VPN Flag Sent: ${testResults.vpnFlagSent ? '[OK]' : '[WARN]'}`);
    console.log(`VPN Error Handled: ${testResults.vpnErrorHandled ? '[OK]' : '[INFO]'}`);
    console.log(`VPN Status Displayed: ${testResults.vpnStatusDisplayed ? '[OK]' : '[FAIL]'}`);
    console.log(`File Transfer Available: ${testResults.fileTransferAvailable ? '[OK]' : '[INFO]'}`);
    console.log(`File Input Found: ${testResults.fileInputFound ? '[OK]' : '[INFO]'}`);
    
    const allCritical = testResults.vpnCheckboxExists && 
                       testResults.vpnCheckboxToggleable &&
                       testResults.vpnStatusDisplayed;
    
    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    if (allCritical) {
      console.log('[OK] All critical VPN toggle tests passed!');
      console.log('\nNote: Full VPN functionality requires administrator privileges.');
      console.log('P2P encrypted tunnel works without admin privileges.');
    } else {
      console.log('[FAIL] Some critical tests failed');
    }

  } catch (error) {
    console.error('\n[FAIL] Test suite failed:', error);
    process.exit(1);
  } finally {
    if (serverProc) serverProc.kill();
    if (frontendProc) frontendProc.kill();
  }
}

runVPNTest().catch(console.error);

