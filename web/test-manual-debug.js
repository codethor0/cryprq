#!/usr/bin/env node
// Manual debug test to verify encryption status detection

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = join(__dirname, '..');
const CRYPRQ_BIN = join(PROJECT_ROOT, 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ');

console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
console.log('🔍 Manual Debug Test - CrypRQ Output Capture');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

console.log('Binary:', CRYPRQ_BIN);
console.log('Starting CrypRQ listener...\n');

const proc = spawn(CRYPRQ_BIN, ['--listen', '/ip4/127.0.0.1/udp/10008/quic-v1'], {
  stdio: ['ignore', 'pipe', 'pipe'],
  env: { ...process.env, RUST_LOG: 'trace' }
});

let stdoutLines = [];
let stderrLines = [];

proc.stdout.on('data', (d) => {
  const s = d.toString();
  const lines = s.split(/\r?\n/).filter(Boolean);
  stdoutLines.push(...lines);
  lines.forEach(line => {
    console.log(`[STDOUT] ${line}`);
    if (/Local peer id:/i.test(line)) {
      console.log('  ✅ DETECTED: Peer ID (encryption indicator)');
    }
    if (/Starting listener/i.test(line)) {
      console.log('  ✅ DETECTED: Starting listener');
    }
    if (/Listening on/i.test(line)) {
      console.log('  ✅ DETECTED: Listening (encryption ready)');
    }
  });
});

proc.stderr.on('data', (d) => {
  const s = d.toString();
  const lines = s.split(/\r?\n/).filter(Boolean);
  stderrLines.push(...lines);
  lines.forEach(line => {
    console.log(`[STDERR] ${line}`);
    if (/key_rotation/i.test(line)) {
      console.log('  ✅ DETECTED: Key rotation (encryption active)');
    }
    if (/INFO.*p2p/i.test(line)) {
      console.log('  ✅ DETECTED: P2P log message');
    }
  });
});

proc.on('exit', (code) => {
  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('📊 Summary');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
  console.log(`Process exited with code: ${code}`);
  console.log(`STDOUT lines: ${stdoutLines.length}`);
  console.log(`STDERR lines: ${stderrLines.length}`);
  
  const hasPeerId = stdoutLines.some(l => /Local peer id:/i.test(l));
  const hasKeyRotation = stderrLines.some(l => /key_rotation/i.test(l));
  const hasListening = stdoutLines.some(l => /Listening on/i.test(l));
  
  console.log(`\nEncryption Indicators:`);
  console.log(`  • Peer ID: ${hasPeerId ? '✅' : '❌'}`);
  console.log(`  • Key Rotation: ${hasKeyRotation ? '✅' : '❌'}`);
  console.log(`  • Listening: ${hasListening ? '✅' : '❌'}`);
  
  if (hasPeerId && hasKeyRotation) {
    console.log('\n✅ Encryption is ACTIVE - all indicators detected!');
  } else {
    console.log('\n❌ Encryption indicators missing');
  }
  
  process.exit(0);
});

setTimeout(() => {
  console.log('\n⏱️  Timeout reached, killing process...\n');
  proc.kill();
}, 5000);

