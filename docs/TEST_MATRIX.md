# CrypRQ v1.0.1 Test Matrix

## Overview

This document provides a systematic test plan for validating the CrypRQ v1.0.1 protocol implementation, focusing on the new record-layer-based data path.

## Prerequisites

- Rust workspace builds: `cargo build --release -p cryprq`
- Test files prepared in `/tmp/`
- Two terminals available for sender/receiver testing

## 1. Minimal Sanity Test

### Purpose
Confirm the happy path with the new CrypRQ record layer stack.

### Steps

**Terminal 1 – Receiver:**
```bash
cryprq receive-file \
  --listen /ip4/0.0.0.0/udp/20440/quic-v1 \
  --output-dir /tmp/receive
```

**Terminal 2 – Sender:**
```bash
cryprq send-file \
  --peer /ip4/127.0.0.1/udp/20440/quic-v1 \
  --file /tmp/testfile.bin
```

**Verification:**
```bash
sha256sum /tmp/testfile.bin /tmp/receive/testfile.bin
```

### Expected Results
- Hashes match
- Logs show FILE_META → FILE_CHUNK → completion
- File appears in `/tmp/receive/` with correct name

---

## 2. File Transfer Correctness & Corner Cases

### 2.1. Tiny File (Single Chunk)

**Test File:**
```bash
echo 'hello' > /tmp/small.txt
```

**Run:**
```bash
# Terminal 1: Receiver (already running)
# Terminal 2:
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/small.txt
```

**Verify:**
- Only 1-2 FILE_CHUNK records in logs
- SHA-256 verification passes
- No extra partial writes on receiver side
- File size matches exactly

**Commands:**
```bash
sha256sum /tmp/small.txt /tmp/receive/small.txt
ls -lh /tmp/receive/small.txt
```

---

### 2.2. Multi-Chunk Medium File (~5-20 MB)

**Test File:**
```bash
dd if=/dev/urandom of=/tmp/medium.bin bs=1M count=10
```

**Run:**
```bash
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/medium.bin
```

**Verify:**
- Sequence numbers increase monotonically per stream
- No gaps or reordering in logs
- Final hash matches
- Logs show multiple FILE_CHUNK records with sequential seq numbers

**Commands:**
```bash
sha256sum /tmp/medium.bin /tmp/receive/medium.bin
# Check logs for sequence numbers: seq=0, seq=1, seq=2, ...
```

---

### 2.3. Large File (~500MB-1GB)

**Purpose:** Stress epoch rotation & sequence counters.

**Test File:**
```bash
dd if=/dev/urandom of=/tmp/large.bin bs=1M count=500
```

**Run:**
```bash
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/large.bin
```

**Verify:**
- Epoch increments (from 0 → 1 → maybe 2) in logs during transfer
- No decrypt/AAD errors at epoch boundaries
- No stalls when keys rotate mid-transfer
- Sequence counters reset on epoch change (seq starts at 0 for new epoch)
- Final hash matches

**Commands:**
```bash
sha256sum /tmp/large.bin /tmp/receive/large.bin
# Monitor logs for epoch changes and sequence resets
```

---

### 2.4. Concurrent Transfers

**Purpose:** Verify independent stream handling.

**Test Files:**
```bash
echo "file1 content" > /tmp/file1.bin
echo "file2 content" > /tmp/file2.bin
```

**Run (Terminal 2 & 3 simultaneously):**
```bash
# Terminal 2:
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/file1.bin

# Terminal 3:
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/file2.bin
```

**Verify:**
- Different Stream IDs (e.g., stream_id=2, stream_id=3) for each transfer
- Records for each stream are independent and in-order
- Both outputs in `/tmp/receive` match originals
- Logs show interleaved FILE_META/FILE_CHUNK for different stream_ids

**Commands:**
```bash
sha256sum /tmp/file1.bin /tmp/receive/file1.bin
sha256sum /tmp/file2.bin /tmp/receive/file2.bin
```

---

### 2.5. Interrupted / Partial Transfer

**Purpose:** Verify graceful handling of incomplete transfers.

**Test File:**
```bash
dd if=/dev/urandom of=/tmp/interrupt.bin bs=1M count=100
```

**Run:**
```bash
# Terminal 2: Start transfer
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/interrupt.bin

# After ~5 seconds, kill sender (Ctrl+C)
```

**Verify:**
- Receiver does NOT claim completion
- Partial file exists OR transfer marked incomplete in logs
- No "transfer complete" log message
- FileTransferManager logs show incomplete transfer state

---

## 3. Epoch, Nonce, and Key-Rotation Behavior

### 3.1. Aggressive Epoch Rotation

**Purpose:** Force epoch changes during transfer to verify key rotation correctness.

**Setup:** Temporarily modify key rotation interval in `node/src/lib.rs`:
```rust
let mut interval = time::interval(Duration::from_secs(5)); // Changed from 300
```

**Test File:**
```bash
dd if=/dev/urandom of=/tmp/epoch_test.bin bs=1M count=50
```

**Run:**
```bash
cryprq send-file --peer /ip4/127.0.0.1/udp/20440/quic-v1 --file /tmp/epoch_test.bin
```

**Verify:**
- Epoch in logs increments frequently (0, 1, 2,...)
- Sequence counters reset on epoch change
- No AEAD decryption failures or "bad tag" errors
- Transfer completes successfully despite multiple epoch changes

**Revert:** Change rotation interval back to 300 seconds after test.

---

### 3.2. Epoch Wrap-Around

**Purpose:** Test epoch wrapping from 255 → 0.

**Setup:** Add debug mode to force epoch to 254 on startup (requires code modification).

**Verify:**
- Labels used in HKDF derived keys change with epoch, including wrapped value
- Receiver happily decrypts across wrap (no misaligned keys)
- Epoch transitions: 254 → 255 → 0 (mod 256)

**Note:** This test requires code modification to force initial epoch.

---

### 3.3. Nonce Reuse Check (Indirect)

**Purpose:** Verify nonce construction prevents reuse.

**Enable Debug Logs:** Add logging for (epoch, stream_id, seq_number) in record layer.

**Verify:**
- For a given (key, epoch, direction), sequence numbers never repeat
- No negative or decreasing sequence numbers
- Nonce construction uses TLS 1.3-style XOR correctly

**Log Pattern to Check:**
```
[RECORD] epoch=0 stream_id=2 seq=0 nonce=...
[RECORD] epoch=0 stream_id=2 seq=1 nonce=...
[RECORD] epoch=1 stream_id=2 seq=0 nonce=... (reset after epoch change)
```

---

## 4. VPN/TUN Path Over Records

### 4.1. Bring Up VPN Stack

**Command:**
```bash
docker compose -f docker-compose.vpn.yml up --build
```

**Verify:**
- TUN interface created
- Tunnel established
- Logs show VPN_PACKET message types

---

### 4.2. Ping Test Over CrypRQ Tunnel

**Setup:** Two VPN nodes connected.

**Run:**
```bash
# On one side:
ping <remote_vpn_ip>
```

**Verify:**
- Logs show MSG_TYPE_VPN_PACKET flow
- TUN read → send_vpn_packet() → records → peer → TUN write
- Ping packets successfully routed

---

### 4.3. Route Actual Traffic

**Run:**
```bash
# Over VPN tunnel:
curl http://example.com
```

**Verify:**
- No crash when under sustained traffic
- Record layer doesn't become bottleneck
- VPN_PACKET records flowing correctly

---

## 5. Web UI & Backend Integration

> **Note:** For comprehensive web-only validation, see `WEB_VALIDATION_RUN.md` which defines a full test matrix (WEB-1 through WEB-7) covering minimal transfers, medium files, concurrent transfers, log streaming, protocol alignment, and security posture checks.

### 5.1. Start Web Stack

**Command:**
```bash
docker compose -f docker-compose.web.yml up --build
```

---

### 5.2. File Upload via Web UI

**Steps:**
1. Open web UI in browser
2. Connect to node (same port as receive-file)
3. Use UI file upload to send a file

**Verify:**
- Backend logs show `send_file_meta` / `send_file_chunk` calls
- Record layer logs show FILE_META / FILE_CHUNK for that stream
- File lands in output directory
- Hashes match

**Reference:** See `WEB_VALIDATION_RUN.md` WEB-1 (Minimal Web Loopback File Transfer) for detailed steps and acceptance criteria.

---

### 5.3. Real-Time Logs

**Verify:**
- Log streaming in UI works with record-based tunnel
- Doesn't depend on old libp2p behavior
- Shows FILE_META, FILE_CHUNK, epoch changes, etc.

**Reference:** See `WEB_VALIDATION_RUN.md` WEB-5 (Web Log Streaming / Events) for detailed validation steps.

---

### 5.4. Web-Only Test Matrix

For complete web-only validation, refer to `WEB_VALIDATION_RUN.md` which includes:

- **WEB-1:** Minimal web loopback file transfer
- **WEB-2:** Medium file web transfer
- **WEB-3:** Concurrent web transfers
- **WEB-4:** CLI ↔ Web mixed transfer (optional)
- **WEB-5:** Web log streaming / events
- **WEB-6:** Protocol alignment (web path)
- **WEB-7:** Security posture checks (web)

See also: `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` for release summary and security posture.

---

## 6. Protocol Alignment Verification

### 6.1. Record Header Structure

**Check:** `core/src/record.rs`

**Verify:**
- 20-byte header exactly
- Field sizes: Version (u8), MsgType (u8), Flags (u8), Epoch (u8), StreamID (u32), Seq (u64), CiphertextLen (u32)
- Byte order: big-endian for multi-byte fields

---

### 6.2. HKDF Key Schedule

**Check:** `crypto/src/kdf.rs`

**Verify:**
- Uses salt `"cryp-rq v1.0 hs"` for handshake keys
- Epoch-scoped keys use labels like `"cryp-rq epoch N ir key"`
- Key derivation uses HKDF-SHA256

---

### 6.3. Nonce Construction

**Check:** `node/src/crypto_utils.rs` or `core/src/record.rs`

**Verify:**
- Nonce = static_IV XOR seq_be (TLS 1.3 style)
- Not simple counter
- 96-bit nonce (12 bytes)

---

### 6.4. Sequence Number Management

**Check:** `node/src/seq_counters.rs`

**Verify:**
- Per-message-type counters (VPN, data, file)
- Monotonic and per-direction
- Reset on epoch change

---

## 7. Security Warnings

### Current Limitations

1. **Hardcoded Test Keys:** CLI uses `[0x01; 32]` etc. for testing
 - **MUST FIX:** Replace with real ML-KEM + X25519 handshake
 - **MUST FIX:** Use Ed25519 peer identity verification

2. **No Real Handshake:** Currently uses placeholder handshake
 - **MUST FIX:** Implement CRYPRQ_CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH

3. **No Peer Authentication:** Peer identity not verified
 - **MUST FIX:** Wire Ed25519 signature verification

### Safe for Testing
- Record layer structure
- File transfer correctness
- Epoch rotation behavior
- Nonce construction

---

## 8. Release Checklist

### MUST Fix Before Production
- [ ] Replace hardcoded keys with real handshake
- [ ] Implement ML-KEM + X25519 hybrid key exchange
- [ ] Wire Ed25519 peer identity verification
- [ ] Remove test key placeholders

### SHOULD Fix Soon
- [ ] Add proper error handling for partial transfers
- [ ] Implement transfer resume/cleanup
- [ ] Add metrics for transfer success/failure rates
- [ ] Improve logging for debugging

### NICE TO HAVE
- [ ] Transfer progress indicators
- [ ] Bandwidth throttling
- [ ] Transfer cancellation API
- [ ] Comprehensive integration tests

---

## 9. Test Execution Log Template

```markdown
## Test Run: [DATE]

### Test: [TEST_NAME]
- **Status:** PASS / FAIL
- **Duration:** [TIME]
- **Logs:** [KEY LOG ENTRIES]
- **Issues:** [ANY ISSUES FOUND]

### Protocol Alignment Check:
- Record header: ✅ / ❌
- HKDF keys: ✅ / ❌
- Nonce construction: ✅ / ❌
- Sequence numbers: ✅ / ❌
```

---

## 10. Quick Reference Commands

### Prepare Test Files
```bash
# Tiny file
echo 'hello' > /tmp/small.txt

# Medium file (10MB)
dd if=/dev/urandom of=/tmp/medium.bin bs=1M count=10

# Large file (500MB)
dd if=/dev/urandom of=/tmp/large.bin bs=1M count=500
```

### Verify Transfers
```bash
# Compare hashes
sha256sum /tmp/original.bin /tmp/receive/original.bin

# Check file sizes
ls -lh /tmp/original.bin /tmp/receive/original.bin
```

### Monitor Logs
```bash
# Filter for file transfer events
grep -E "FILE_META|FILE_CHUNK|stream_id" <log_file>

# Filter for epoch changes
grep -E "epoch|Epoch" <log_file>

# Filter for sequence numbers
grep -E "seq=|sequence" <log_file>
```

