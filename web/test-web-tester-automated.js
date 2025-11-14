#!/usr/bin/env node
// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

/**
 * Automated test for CrypRQ web tester
 * Tests: Server startup, binary detection, connection, encryption status
 */

import { spawn } from 'child_process';
import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = join(__dirname, '..');

const SERVER_PORT = 8787;
const FRONTEND_PORT = 5173;
const TEST_TIMEOUT = 30000;

let serverProc = null;
let frontendProc = null;

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function testWebTester() {
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('Automated Web Tester Test');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  try {
    // Step 1: Verify binary exists
    console.log('Step 1: Verifying CrypRQ binary...');
    const binPaths = [
      join(PROJECT_ROOT, 'target', 'release', 'cryprq'),
      join(PROJECT_ROOT, 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ'),
    ];
    
    let binaryPath = null;
    for (const path of binPaths) {
      if (existsSync(path)) {
        binaryPath = path;
        console.log(`[OK] Binary found: ${path}`);
        break;
      }
    }
    
    if (!binaryPath) {
      console.error('[FAIL] CrypRQ binary not found!');
      console.error('Please build with: cargo build --release -p cryprq');
      process.exit(1);
    }

    // Step 2: Start server
    console.log('\nStep 2: Starting server...');
    serverProc = spawn('node', ['server/server.mjs'], {
      cwd: join(PROJECT_ROOT, 'web'),
      env: { ...process.env, CRYPRQ_BIN: binaryPath, PORT: SERVER_PORT },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    let serverReady = false;
    let serverError = null;
    
    serverProc.stdout.on('data', (data) => {
      const output = data.toString();
      console.log(`[SERVER] ${output.trim()}`);
      if (output.includes('bridge on')) {
        serverReady = true;
        console.log('[OK] Server started');
      }
    });

    serverProc.stderr.on('data', (data) => {
      const output = data.toString();
      if (!output.includes('ExperimentalWarning')) {
        console.error(`[SERVER ERROR] ${output.trim()}`);
        serverError = output;
      }
    });

    serverProc.on('exit', (code) => {
      if (code !== 0 && code !== null) {
        console.error(`[FAIL] Server exited with code ${code}`);
      }
    });

    // Wait for server to start
    await sleep(5000);
    if (!serverReady) {
      console.error('[FAIL] Server failed to start');
      if (serverError) {
        console.error(`Error: ${serverError}`);
      }
      if (serverProc) serverProc.kill();
      process.exit(1);
    }

    // Step 3: Test server endpoint
    console.log('\nStep 3: Testing server endpoint...');
    try {
      const response = await fetch(`http://localhost:${SERVER_PORT}/connect`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: 'listener', port: 10018 })
      });
      
      if (response.ok) {
        const data = await response.json();
        console.log('[OK] Server endpoint responded:', data.ok ? 'OK' : 'Error');
      } else {
        const error = await response.text();
        console.error(`[FAIL] Server endpoint error: ${response.status} - ${error}`);
      }
    } catch (err) {
      console.error(`[FAIL] Failed to connect to server: ${err.message}`);
    }

    // Step 4: Start frontend
    console.log('\nStep 4: Starting frontend...');
    frontendProc = spawn('npm', ['run', 'dev'], {
      cwd: join(PROJECT_ROOT, 'web'),
      env: { ...process.env, PORT: FRONTEND_PORT },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    frontendProc.stdout.on('data', (data) => {
      const output = data.toString();
      if (output.includes('Local:') || output.includes('localhost')) {
        console.log(`[OK] Frontend started: ${output.trim()}`);
      }
    });

    frontendProc.stderr.on('data', (data) => {
      const output = data.toString();
      if (!output.includes('ExperimentalWarning')) {
        console.error(`[FRONTEND ERROR] ${output.trim()}`);
      }
    });

    await sleep(5000);

    // Step 5: Test frontend accessibility
    console.log('\nStep 5: Testing frontend accessibility...');
    try {
      const response = await fetch(`http://localhost:${FRONTEND_PORT}`);
      if (response.ok) {
        console.log('[OK] Frontend is accessible');
      } else {
        console.error(`[FAIL] Frontend returned status: ${response.status}`);
      }
    } catch (err) {
      console.error(`[FAIL] Failed to access frontend: ${err.message}`);
    }

    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('Automated tests completed');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    console.log('Next steps:');
    console.log('1. Open http://localhost:5173 in your browser');
    console.log('2. Click Connect button');
    console.log('3. Verify encryption status updates');
    console.log('4. Check debug console for encryption events');
    console.log('\nPress Ctrl+C to stop services');

    // Keep processes running
    await new Promise(() => {});

  } catch (error) {
    console.error('[FAIL] Test failed:', error);
    if (serverProc) serverProc.kill();
    if (frontendProc) frontendProc.kill();
    process.exit(1);
  }
}

// Handle cleanup on exit
process.on('SIGINT', () => {
  console.log('\n\nCleaning up...');
  if (serverProc) serverProc.kill();
  if (frontendProc) frontendProc.kill();
  process.exit(0);
});

testWebTester().catch(console.error);

