# Master Prompt: Fix and Test CrypRQ Web Tester

## Objective

Debug, fix, and comprehensively test the CrypRQ web tester to ensure:
1. Server connects successfully to CrypRQ binary
2. Frontend displays encryption status correctly in real-time
3. ML-KEM (Kyber768) + X25519 hybrid encryption is verified and working
4. Automated tests pass
5. Live testing works end-to-end

## Phase 1: Debugging and Fixing Connection Issues

### Step 1: Identify Root Cause
```bash
# Check server logs for errors
tail -100 /tmp/server_live_test.log

# Check if server is running
ps aux | grep "server.mjs"

# Test server endpoints
curl -v http://localhost:8787/events
curl -X POST http://localhost:8787/connect -H "Content-Type: application/json" -d '{"mode":"listener","port":10000}'

# Verify CrypRQ binary exists and is executable
ls -la target/release/cryprq
file target/release/cryprq
```

### Step 2: Fix Server Issues
**Tasks:**
1. **Fix Binary Path Detection**
   - Ensure `CRYPRQ_BIN` environment variable is set correctly
   - Add fallback to check multiple paths:
     - `target/release/cryprq` (cargo build)
     - `dist/macos/CrypRQ.app/Contents/MacOS/CrypRQ` (macOS app)
     - `cryprq` (system PATH)
   - Verify binary exists before spawning

2. **Fix Process Spawning**
   - Add error handling for spawn failures
   - Verify binary permissions (executable)
   - Add timeout handling for stuck processes
   - Ensure stdout/stderr are properly captured

3. **Fix Event Streaming**
   - Verify EventSource connection works
   - Ensure events are broadcast to all clients
   - Add reconnection logic for dropped connections
   - Verify CORS headers are correct

4. **Fix Port Conflicts**
   - Check for port conflicts before starting listener
   - Clean up stale processes properly
   - Use unique ports for testing

### Step 3: Fix Frontend Issues
**Tasks:**
1. **Fix EventSource Connection**
   - Verify EventSource connects to correct URL
   - Handle connection errors gracefully
   - Add reconnection logic
   - Display connection status

2. **Fix Status Updates**
   - Ensure encryption status updates immediately on Connect
   - Parse CrypRQ output correctly
   - Update UI state based on events
   - Handle edge cases (disconnections, errors)

3. **Fix Encryption Status Display**
   - Show encryption method correctly
   - Update connection status based on actual state
   - Display encryption proof events
   - Show peer ID when available

## Phase 2: Automated Testing

### Step 1: Create Automated Test Script
**File: `web/test-web-tester-automated.js`**
```javascript
#!/usr/bin/env node
// Automated test for CrypRQ web tester
// Tests: Server startup, binary detection, connection, encryption status

import puppeteer from 'puppeteer';
import { spawn } from 'child_process';
import { existsSync } from 'fs';
import { join } from 'path';

const SERVER_PORT = 8787;
const FRONTEND_PORT = 5173;
const TEST_TIMEOUT = 30000;

async function testWebTester() {
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('🧪 Automated Web Tester Test');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  // Step 1: Verify binary exists
  console.log('Step 1: Verifying CrypRQ binary...');
  const binPaths = [
    join(process.cwd(), 'target', 'release', 'cryprq'),
    join(process.cwd(), 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ'),
  ];
  
  let binaryPath = null;
  for (const path of binPaths) {
    if (existsSync(path)) {
      binaryPath = path;
      console.log(`✅ Binary found: ${path}`);
      break;
    }
  }
  
  if (!binaryPath) {
    console.error('❌ CrypRQ binary not found!');
    process.exit(1);
  }

  // Step 2: Start server
  console.log('\nStep 2: Starting server...');
  const serverProc = spawn('node', ['server/server.mjs'], {
    cwd: join(process.cwd(), 'web'),
        env: { ...process.env, CRYPRQ_BIN: binaryPath, PORT: SERVER_PORT },
    stdio: ['ignore', 'pipe', 'pipe']
  });

  let serverReady = false;
  serverProc.stdout.on('data', (data) => {
    const output = data.toString();
    if (output.includes('bridge on')) {
      serverReady = true;
      console.log('✅ Server started');
    }
  });

  serverProc.stderr.on('data', (data) => {
    console.error(`[SERVER ERROR] ${data.toString()}`);
  });

  // Wait for server to start
  await new Promise(resolve => setTimeout(resolve, 3000));
  if (!serverReady) {
    console.error('❌ Server failed to start');
    serverProc.kill();
    process.exit(1);
  }

  // Step 3: Start frontend
  console.log('\nStep 3: Starting frontend...');
  const frontendProc = spawn('npm', ['run', 'dev'], {
    cwd: join(process.cwd(), 'web'),
    env: { ...process.env, PORT: FRONTEND_PORT },
    stdio: ['ignore', 'pipe', 'pipe']
  });

  await new Promise(resolve => setTimeout(resolve, 5000));

  // Step 4: Test with Puppeteer
  console.log('\nStep 4: Testing with Puppeteer...');
  const browser = await puppeteer.launch({ headless: false });
  const page = await browser.newPage();

  try {
    // Navigate to frontend
    await page.goto(`http://localhost:${FRONTEND_PORT}`, { waitUntil: 'networkidle2' });

    // Test 1: Verify initial state
    console.log('\nTest 1: Verifying initial state...');
    const encryptionMethod = await page.evaluate(() => {
      const elem = Array.from(document.querySelectorAll('*')).find(e => 
        e.textContent && e.textContent.includes('ML-KEM (Kyber768) + X25519 hybrid')
      );
      return elem ? elem.textContent : null;
    });
    
    if (encryptionMethod) {
      console.log('✅ Encryption method displayed correctly');
    } else {
      console.error('❌ Encryption method not found');
    }

    // Test 2: Click Connect button
    console.log('\nTest 2: Clicking Connect button...');
    await page.click('button:has-text("Connect")');
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Test 3: Verify status updates
    console.log('\nTest 3: Verifying status updates...');
    const statusText = await page.evaluate(() => {
      const statusElem = Array.from(document.querySelectorAll('*')).find(e =>
        e.textContent && (e.textContent.includes('[STARTING]') || 
                         e.textContent.includes('[WAITING]') ||
                         e.textContent.includes('[CONNECTING]'))
      );
      return statusElem ? statusElem.textContent : null;
    });

    if (statusText) {
      console.log(`✅ Status updated: ${statusText}`);
    } else {
      console.error('❌ Status did not update');
    }

    // Test 4: Check debug console for encryption events
    console.log('\nTest 4: Checking debug console...');
    const debugEvents = await page.evaluate(() => {
      const consoleElem = Array.from(document.querySelectorAll('*')).find(e =>
        e.textContent && (e.textContent.includes('Local peer id') ||
                         e.textContent.includes('key_rotation') ||
                         e.textContent.includes('ENCRYPTION PROOF'))
      );
      return consoleElem ? consoleElem.textContent : null;
    });

    if (debugEvents) {
      console.log('✅ Encryption events detected in debug console');
    } else {
      console.log('⚠️  Encryption events not yet visible (may take time)');
    }

    console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('✅ Automated tests completed');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  } catch (error) {
    console.error('❌ Test failed:', error);
  } finally {
    await browser.close();
    serverProc.kill();
    frontendProc.kill();
  }
}

testWebTester().catch(console.error);
```

### Step 2: Run Automated Tests
```bash
cd web
node test-web-tester-automated.js
```

## Phase 3: Live Testing

### Step 1: Start Services
```bash
# Kill any existing processes
pkill -f "server.mjs|vite" || true

# Set binary path
export CRYPRQ_BIN=$(pwd)/target/release/cryprq

# Start server
cd web
node server/server.mjs > /tmp/server_test.log 2>&1 &
SERVER_PID=$!

# Start frontend
npm run dev > /tmp/frontend_test.log 2>&1 &
FRONTEND_PID=$!

echo "Server PID: $SERVER_PID"
echo "Frontend PID: $FRONTEND_PID"
```

### Step 2: Manual Testing Checklist
1. **Initial State Verification**
   - [ ] Open http://localhost:5173
   - [ ] Encryption Method shows "ML-KEM (Kyber768) + X25519 hybrid"
   - [ ] Connection Status shows "[READY] Encryption Active (ready to connect)..."
   - [ ] Mode is set to "Listener"
   - [ ] Port is 10000

2. **Listener Mode Test**
   - [ ] Click "Connect" button
   - [ ] Status immediately changes to "[STARTING] Starting (encryption active)..."
   - [ ] Status updates to "[WAITING] Listening (encryption active, waiting for peer)"
   - [ ] Debug Console shows encryption events:
     - [ ] "[ENCRYPTION PROOF] Peer ID generated"
     - [ ] "Local peer id: 12D3KooW..."
     - [ ] "Listening on /ip4/127.0.0.1/udp/10000/quic-v1"

3. **Dialer Mode Test**
   - [ ] Open second browser tab
   - [ ] Set mode to "Dialer"
   - [ ] Set peer address to listener's address
   - [ ] Click "Connect"
   - [ ] Status updates to "[ACTIVE] Encrypted Tunnel Active"
   - [ ] Debug Console shows connection events:
     - [ ] "Connected to 12D3KooW..."
     - [ ] "key_rotation" events

4. **Error Handling**
   - [ ] Test with invalid port (should show error)
   - [ ] Test with missing binary (should show error)
   - [ ] Test disconnection (should handle gracefully)

## Phase 4: Fix Specific Issues

### Issue 1: Server Not Starting
**Symptoms:** Server crashes or doesn't respond
**Fix:**
- Check for `require()` statements in ES module (should use `import`)
- Verify all imports are correct
- Check for port conflicts
- Verify Node.js version compatibility

### Issue 2: Binary Not Found
**Symptoms:** "CrypRQ binary not found" error
**Fix:**
- Build binary: `cargo build --release -p cryprq`
- Set `CRYPRQ_BIN` environment variable
- Add fallback paths in server code
- Verify binary permissions

### Issue 3: Events Not Streaming
**Symptoms:** Debug console shows no events
**Fix:**
- Verify EventSource connection
- Check server logs for errors
- Ensure `push()` function broadcasts to all clients
- Verify CORS headers
- Check CrypRQ binary output format

### Issue 4: Status Not Updating
**Symptoms:** UI shows "Disconnected" when encryption is active
**Fix:**
- Update `connect()` to set encryption status immediately
- Fix event parsing logic
- Update `EncryptionStatus.tsx` display logic
- Add more event patterns to detect encryption

### Issue 5: Port Conflicts
**Symptoms:** "Address already in use" error
**Fix:**
- Check for existing processes: `lsof -ti:10000`
- Kill stale processes before starting
- Use unique ports for testing
- Add port conflict detection

## Phase 5: Comprehensive Verification

### Step 1: Run All Tests
```bash
# Unit tests
cargo test --all

# Automated web tester test
cd web && node test-web-tester-automated.js

# Manual verification
# Follow Phase 3 checklist
```

### Step 2: Generate Test Report
**File: `WEB_TESTER_TEST_REPORT.md`**
- Document all test results
- List any issues found and fixes applied
- Verify encryption method is working
- Confirm real-time status updates

### Step 3: Cleanup and Documentation
- Update README with testing instructions
- Document known issues and workarounds
- Add troubleshooting section
- Update API documentation

## Expected Outcomes

1. ✅ Server starts successfully and connects to CrypRQ binary
2. ✅ Frontend displays encryption status correctly
3. ✅ Real-time status updates work
4. ✅ Encryption method (ML-KEM + X25519) is verified
5. ✅ Automated tests pass
6. ✅ Live testing works end-to-end
7. ✅ Error handling works correctly
8. ✅ Documentation is updated

## Deliverables

1. Fixed server code (`web/server/server.mjs`)
2. Fixed frontend code (`web/src/App.tsx`, `web/src/EncryptionStatus.tsx`)
3. Automated test script (`web/test-web-tester-automated.js`)
4. Test report (`WEB_TESTER_TEST_REPORT.md`)
5. Updated documentation
6. Troubleshooting guide

## Debugging Commands

```bash
# Check server logs
tail -f /tmp/server_test.log

# Check frontend logs
tail -f /tmp/frontend_test.log

# Test server endpoint
curl -v http://localhost:8787/connect -X POST -H "Content-Type: application/json" -d '{"mode":"listener","port":10000}'

# Check for port conflicts
lsof -ti:8787
lsof -ti:5173
lsof -ti:10000

# Verify binary
file target/release/cryprq
target/release/cryprq --help

# Test binary directly
RUST_LOG=trace target/release/cryprq --listen /ip4/127.0.0.1/udp/10018/quic-v1
```

## Success Criteria

- [ ] Server starts without errors
- [ ] Frontend loads and displays correctly
- [ ] Connect button works
- [ ] Encryption status updates in real-time
- [ ] Debug console shows encryption events
- [ ] Two nodes can connect (listener + dialer)
- [ ] Automated tests pass
- [ ] No console errors in browser
- [ ] No server errors in logs

