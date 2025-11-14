# CrypRQ v1.0.1 — Web UI Guide

**File:** docs/WEB_UI_GUIDE.md  
**Scope:** Using the web UI to exercise the CrypRQ v1.0.1 stack (test mode)  
**Audience:** Developers, testers, and demo users.

> ⚠️ **Important:** The web UI currently drives a **test-mode** backend:
> - Static test keys
> - No handshake or peer authentication
> - NOT suitable for production use

See `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` and `SECURITY_NOTES.md` for the security posture.

---

## 1. Web UI Overview

The CrypRQ Web UI is a thin control and visibility layer over the v1.0.1 record-based backend. It provides:

- A **File Transfer** panel for:
  - Selecting a file.
  - Entering/choosing a peer or endpoint.
  - Starting a CrypRQ-based file transfer.
  - Observing progress and completion status.
- A **Status / Logs** section for:
  - Real-time updates from the backend (via `/events` or WebSocket).
  - Displaying high-level events: transfers starting/completing, errors, warnings.

The UI is intentionally minimal and focused on protocol validation and demos.

---

## 2. Architecture (High-Level)

```text
[Browser / Web UI]  <-- HTTP / WebSocket / SSE -->  [Web Backend]
         |                                                |
         |                              CrypRQ v1.0.1 record layer
         |                                                |
         '---------------------------------->  [UDP peer]  (loopback or remote)
```

Key backend endpoints (as integrated in previous phases):

- `GET /connect` — (Optional) Initialize or verify a backend connection / configuration.
- `GET /events` — Server-Sent Events (SSE) or equivalent for live status/log streaming.
- `POST /api/send-file` — Initiates a file transfer via the CrypRQ tunnel.

Each `send-file` operation results in:
- Backend creating/using a `Tunnel`.
- File metadata and chunks being sent as `FILE_META` + `FILE_CHUNK` records.
- Events being pushed to the UI via `/events`.

---

## 3. Starting the Web Stack

Follow `DOCKER_WEB_GUIDE.md` for full details. Short version:

```bash
docker compose -f docker-compose.web.yml up --build
```

Then open the UI in your browser:

```
http://localhost:<frontend_port>
```

(e.g., `http://localhost:3000` if that's how the compose file is configured.)

---

## 4. File Transfer via Web UI

### 4.1 Minimal Web File Transfer

**Goal:** Send a small file via the web UI and confirm it is received correctly.

**Prepare a test file** on the host (optional if UI lets you create/choose):

```bash
echo "Test file for CrypRQ web v1.0.1" > test-web-minimal.bin
sha256sum test-web-minimal.bin
```

**Open the Web UI**

1. Navigate to the File Transfer section.

**Select a file**

2. Click the file picker.
3. Choose `test-web-minimal.bin`.

**Configure the peer/endpoint**

4. Enter the backend's CrypRQ target (depending on how the backend is set up).
   - In a simple loopback test, the backend may be configured to treat itself as both sender and receiver over `udp://127.0.0.1:20440` or similar.
   - Use whatever peer format the UI expects (`/ip4/.../udp/.../quic-v1` or simple `udp://host:port`), as documented in the UI/placeholder help text.

**Start the transfer**

5. Click Send (or the equivalent button).

**Watch the status panel**

6. You should see events like:
   - "Transfer started"
   - "Sending file: test-web-minimal.bin"
   - "Transfer complete" (or similar wording)
   - If an error occurs, you should see an error message.

**Verify on disk**

7. Locate the output directory used by the backend (see `WEB_VALIDATION_RUN.md` or backend config).
8. Compare hashes:
   ```bash
   sha256sum test-web-minimal.bin /path/to/received/test-web-minimal.bin
   ```
   They should be identical.

---

## 5. Status / Logs Panel

The UI should expose a log or status area that reflects backend events arriving from `/events` or from a WebSocket.

**Typical events** (sanitized):

- `file_transfer_started`
  - Fields: `filename`, `size`, `stream_id`
- `file_chunk_sent` or `file_chunk_received`
  - Optional, for detailed progress.
- `file_transfer_complete`
  - Fields: `filename`, `size`, `duration`, `stream_id`
- `error`
  - Fields: `type`, `message` (non-sensitive), possibly `stream_id`

**Notes:**

- No cryptographic keys, nonces, or raw ciphertext should appear in UI logs.
- File paths may be shown, but avoid leaking sensitive directories in production.

**Use this panel to:**

- Confirm that the UI is successfully connected to `/events`.
- Debug failures by inspecting high-level error messages.

---

## 6. API Details (Developer-Facing)

This is a conceptual summary for developers. The exact JSON shapes may vary; adjust this section to match the actual implementation.

### 6.1 POST /api/send-file

**Purpose:** Start a file transfer via the CrypRQ backend.

**Example** (conceptual)

Request: `multipart/form-data` or `application/json` with a file + metadata.

Possible JSON shape (if not multipart):

```json
{
  "peer": "/ip4/127.0.0.1/udp/20440/quic-v1",
  "filename": "test-web-minimal.bin",
  "size": 39,
  "content_base64": "VGhpcyBpcyBhIHRlc3QgZmlsZSBjb250ZW50..."
}
```

Response: `200 OK` with a transfer ID:

```json
{
  "status": "accepted",
  "transfer_id": "abc123",
  "stream_id": 2
}
```

The backend then:
- Allocates a `stream_id`.
- Sends `FILE_META` and `FILE_CHUNK` records via the record layer.
- Emits events on `/events`.

### 6.2 GET /events

**Purpose:** Provide real-time updates to the UI.

If using SSE:
- Endpoint returns `text/event-stream`.
- Events look like:

```
event: file_transfer_started
data: {"transfer_id":"abc123","stream_id":2,"filename":"test-web-minimal.bin","size":39}

event: file_transfer_complete
data: {"transfer_id":"abc123","stream_id":2,"filename":"test-web-minimal.bin","bytes_sent":39,"elapsed_ms":42}
```

The UI subscribes and updates the status/log panel accordingly.

### 6.3 GET /connect (Optional)

**Purpose:** Initialize or verify backend state (for example, test mode and UDP config).

Response might include:

```json
{
  "mode": "test",
  "udp_bind": "0.0.0.0:20440",
  "record_layer": "cryp-rq v1.0.1"
}
```

The UI can show this information in a status bar or "connection info" section.

---

## 7. Limitations (Web UI in v1.0.1)

- **Test mode only**
  - No real handshake.
  - No peer identity validation.
  - Static keys and a key-direction hack on the receiver.
- **Single-node / lab use**
  - Designed for loopback or small lab environments.
  - Not tested or hardened for multi-tenant or Internet-wide use.
- **Focus on file transfer**
  - VPN/TUN flows are validated by CLI and backend, not exposed as a polished web UX yet.
  - Web UI currently treats everything as "file transfer use case."

See `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` for a full rundown of MUST-FIX items before production.

---

## 8. Suggested Workflows

### 8.1 Developer Regression Loop

1. Modify backend or record layer code.
2. Rebuild and restart Docker:
   ```bash
   docker compose -f docker-compose.web.yml up --build
   ```
3. Use the Web UI to send:
   - A small file.
   - A medium file (~10 MB).
4. Verify hashes and check logs.
5. Update `WEB_VALIDATION_RUN.md` as needed (WEB-1, WEB-2 tests).

### 8.2 Demo Workflow

1. Start the stack ahead of time.
2. Show:
   - A small file transfer (fast).
   - Status panel updates in real time.
3. Optionally show:
   - `tcpdump` on the UDP port to prove encrypted traffic.
   - Mapping from UI action → backend events.

---

## 9. References

- `DOCKER_WEB_GUIDE.md` — Docker deployment details.
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` — Web-only release description.
- `WEB_VALIDATION_RUN.md` — Web validation matrix and results.
- `VALIDATION_RUN.md` — CLI/tunnel validation.
- `TEST_MATRIX.md` — Combined test matrix (CLI + web).
- `SECURITY_NOTES.md` — Security posture and limitations.

