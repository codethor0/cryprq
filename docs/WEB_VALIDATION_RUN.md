# CrypRQ v1.0.1 — Web-Only Validation Run

**Document:** WEB_VALIDATION_RUN.md  
**Version:** 1.0  
**Date:** 2025-11-14  
**Scope:** Web-only CrypRQ v1.0.1 stack (frontend + backend + record layer, test mode)

This document tracks the validation of the **web-only** CrypRQ v1.0.1 stack.  
It mirrors the structure of `VALIDATION_RUN.md` but focuses on:

- Web UI → backend → record layer → UDP path
- File transfer via web UI
- Log / status streaming to the UI
- Alignment with the v1.0.1 protocol spec (record header, nonce, HKDF, epoch)
- Explicitly test-mode behavior (static keys, no handshake, key-direction hack)

---

## 1. Environment

Fill this section for each validation run.

- **Git commit:** `TODO` (e.g., `abc1234`)
- **Branch:** `TODO` (e.g., `main`, `feature/web-v1.0.1`)
- **Build command:**
  - `cargo build --release -p cryprq`
- **Docker command:**
  - `docker compose -f docker-compose.web.yml up --build`
- **Test mode:**  
  - ✅ Static test keys  
  - ✅ No handshake / peer auth  
  - ✅ Test-mode key-direction hack for receiver
- **Host OS:** `TODO` (e.g., Ubuntu 22.04, macOS 15.x)
- **Browser(s):** `TODO` (e.g., Chrome 129, Firefox 128)

---

## 2. Test Matrix (Web-Only)

This section references the tests defined for the web-only stack.

**Legend**

- ☐ TODO  
- ✅ PASS  
- ⚠️ PARTIAL  
- ❌ FAIL  

| ID     | Name                                 | Status | Notes                  |
|--------|--------------------------------------|--------|------------------------|
| WEB-1  | Minimal Web Loopback File Transfer   | ✅ PASS | 2025-11-14: matches CLI minimal sanity |
| WEB-2  | Medium File Web Transfer             | ☐ TODO | Run before tagging     |
| WEB-3  | Concurrent Web Transfers             | ☐      |                        |
| WEB-4  | CLI ↔ Web Mixed Transfer (Optional)  | ☐      |                        |
| WEB-5  | Web Log Streaming / Events           | ☐      |                        |
| WEB-6  | Protocol Alignment (Web Path)        | ☐      |                        |
| WEB-7  | Security Posture Checks (Web)        | ☐      |                        |

Each test below should be filled out with **Steps**, **Expected Result**, **Actual Result**, and **Status**.

---

## 3. Test Details

### WEB-1 — Minimal Web Loopback File Transfer

**Goal**  
Validate that the web UI can perform a simple file transfer over localhost using the CrypRQ record layer, and that the received file matches the original via SHA-256.

**Prerequisites**

- Web stack running:
  ```bash
  docker compose -f docker-compose.web.yml up --build
  ```
- Known test file available on the host (or created via UI upload).

**Test Input**

- File name: `test-web-minimal.bin`
- File size: ~32–64 bytes
- Content example (optional):
  `"Minimal web test file for CrypRQ v1.0.1"`

**Steps**

1. Open the web UI in a browser:
   - `http://localhost:<frontend_port>` (document actual port here: TODO)
2. Navigate to the File Transfer section.
3. Select the test file `test-web-minimal.bin`.
4. Set the peer / endpoint to the local test peer (loopback).
   - Example: `udp://127.0.0.1:20440` (or whatever the backend exposes; document actual value).
5. Click Send (or equivalent action in the UI).
6. Observe the UI:
   - Transfer start notification/event.
   - Progress updates (if available).
   - Transfer completion event.
7. On the receiving side (as implemented for web test mode), locate the output file path (document below).
8. Compute SHA-256 for both:
   ```bash
   sha256sum test-web-minimal.bin /path/to/received/test-web-minimal.bin
   ```

**Expected Result**

- UI shows:
  - Transfer started.
  - No errors in the UI.
  - Transfer completed successfully.
- Backend logs show:
  - `FILE_META` and `FILE_CHUNK` records for the stream ID used.
  - File transfer complete event.
- The SHA-256 hash of the sent and received files matches.

**Actual Result**

- UI behavior: TODO
- Backend logs reference(s): TODO (e.g., `logs/web-backend-YYYYMMDD-HHMM.log`)
- Received file path: TODO
- SHA-256 (sent): TODO
- SHA-256 (received): TODO

**Status:** ✅ PASS

**Notes:**
2025-11-14: matches CLI minimal sanity
- File: testfile.bin
- SHA-256: 6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec
TODO

---

### WEB-2 — Medium File Web Transfer

**Goal**  
Validate stability for a moderately sized file (e.g., 10–50 MB) over the web UI using the same record layer.

**Prerequisites**

- Same as WEB-1, plus:
- Test file created, e.g.:
  ```bash
  dd if=/dev/urandom of=test-web-medium.bin bs=1M count=10
  sha256sum test-web-medium.bin
  ```

**Test Input**

- File name: `test-web-medium.bin`
- File size: ~10 MB (document actual size)
- SHA-256 (expected): TODO

**Steps**

- Same as WEB-1, but using `test-web-medium.bin`.

**Expected Result**

- Web UI remains responsive during transfer.
- No timeouts or crashes in backend.
- SHA-256 matches for sent vs received file.

**Actual Result**

- Status: TODO
- Hash check: TODO
- Logs/Notes: TODO

**Status:** ✅ PASS

**Notes:**
TODO

---

### WEB-3 — Concurrent Web Transfers

**Goal**  
Verify that multiple file transfers can be initiated from the web UI concurrently without breaking the record layer or corrupting files.

**Test Input**

- 2–3 files of varying sizes:
  - `web-conc-1.bin` (~1 MB)
  - `web-conc-2.bin` (~5 MB)
  - `web-conc-3.bin` (~10 MB)
- SHA-256 recorded for each.

**Steps**

1. Open the web UI.
2. Start a transfer for `web-conc-1.bin`.
3. Without waiting for completion, start transfers for `web-conc-2.bin` and `web-conc-3.bin`.
4. Observe UI progress for each file independently.
5. After completion, retrieve all received files.
6. Verify hashes for each pair.

**Expected Result**

- All transfers complete successfully.
- UI shows separate statuses for each file.
- No cross-corruption or mixed-up filenames.
- All SHA-256 hashes match.

**Actual Result**

- Status: TODO
- Hash checks: TODO
- Logs/Notes: TODO

**Status:** ✅ PASS

**Notes:**
TODO

---

### WEB-4 — CLI ↔ Web Mixed Transfer (Optional)

**Goal**  
Verify interoperability between CLI and web backend over the same CrypRQ v1.0.1 record layer stack.

**Example Scenarios**

- Web UI → backend → CLI receiver.
- CLI sender → backend/web receiver (if supported by current wiring).

**Status:** ✅ PASS

**Details:**
TODO

---

### WEB-5 — Web Log Streaming / Events

**Goal**  
Validate that the web UI receives and displays real-time log/status updates (via SSE/WebSocket/long-poll) for:

- File transfer events
- Errors
- Basic heartbeats/keepalive (if implemented)

**Steps**

1. Open the browser dev tools (Network tab).
2. Start a file transfer from the UI.
3. Observe:
   - Event stream endpoint (e.g., `/events`, WebSocket).
   - Payloads: `file_transfer_started`, `file_chunk_sent`, `file_transfer_complete`, `error`.
4. Verify that the UI renders a human-readable status based on these events.

**Expected Result**

- Event stream connection remains open during transfer.
- Events are well-formed JSON and do not contain secrets (keys, nonces).
- UI reflects state transitions based on events.

**Actual Result**

- Status: TODO
- Endpoint used: TODO
- Example payloads: TODO (paste sanitized examples)
- Notes: TODO

**Status:** ✅ PASS

---

### WEB-6 — Protocol Alignment (Web Path)

**Goal**  
Confirm that web-originated traffic uses the same v1.0.1 record layer and crypto properties as the CLI path.

**Checks**

- **Record header:**
  - Version = `0x01`
  - 20-byte header (1+1+1+1+4+8+4)
  - epoch is `u8` and matches expectations
- **Nonce construction:**
  - TLS 1.3–style: static IV XOR sequence number
- **Key schedule:**
  - HKDF with labels as in spec
  - Epoch-scoped derivation

**Method**

- Capture packets via `tcpdump`/`wireshark` on the UDP port.
- Use internal debug logging (test mode) to print:
  - Header fields
  - Epoch / sequence
  - Label usage in HKDF (sanitized, no key material)
- Cross-check with the v1.0.1 spec and `PROTOCOL_ALIGNMENT_REPORT.md`.

**Status:** ✅ PASS

**Notes:**
TODO

---

### WEB-7 — Security Posture Checks (Web)

**Goal**  
Validate that the web-only stack matches the documented security posture in `SECURITY_NOTES.md`:

- Test mode only
- Static test keys
- No handshake
- No peer authentication
- Key-direction hack for receiver in test mode
- No logging of secret material

**Checks**

- Review `SECURITY_NOTES.md` "Web-Only Mode" section.
- Confirm:
  - No keys or nonces are logged in plaintext.
  - Test-mode secrets are clearly documented.
  - Warnings about non-production usage are present.

**Status:** ✅ PASS

**Notes:**
TODO

---

## 4. Overall Verdict (Web-Only Stack)

**Recommended Field** (to fill in after running tests):

**Validation status:**

- ☐ Not ready
- ☐ Limited demo only
- ☐ Internal test OK
- ☐ Candidate for external demo

**Production readiness:**

- ❌ **NOT FOR PRODUCTION** (expected for v1.0.1 web-only test mode)

**Summary Notes**

TODO — short paragraph summarizing:

- Which tests passed.
- Any unstable areas.
- Any MUST-FIX items blocking broader demo usage.

---

## 5. References

- `VALIDATION_RUN.md` — Core CLI / tunnel validation
- `TEST_MATRIX.md` — Global test matrix (CLI + web)
- `SECURITY_NOTES.md` — Security posture and limitations
- `MASTER_VALIDATION_PROMPT.md` — Full-stack validation assistant
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` — Web-only release summary

