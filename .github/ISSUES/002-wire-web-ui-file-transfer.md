# Wire Web UI for File Transfer and Log Streaming

**Summary**

Connect the React/TypeScript web UI to the CrypRQ v1.0.1 backend (via HTTP/WebSocket/SSE) so that file transfer and real-time log/status streaming work end-to-end using the record layer.

**Context**

* CLI `send-file` / `receive-file` are validated and use:
  * Record layer (20-byte header, epoch/seq, AES/ChaCha + AEAD).
  * `FileTransferManager` with `FILE_META`, `FILE_CHUNK`, `FILE_ACK`.
* Web UI already has endpoints like `/api/send-file` and `/events` but needs to be guaranteed aligned with the new stack.

**Tasks**

* [ ] Inspect frontend code (React/TS) to:
  * [ ] Identify components for file selection and "Send" actions.
  * [ ] Identify where logs/status are displayed (SSE/WebSocket/long-poll).
  * [ ] Map the API calls made (URLs, request bodies, expected responses).
* [ ] Define or confirm a simple JSON API contract, e.g.:
  * [ ] `POST /api/send-file`:
    - Request: `{ "file_name": string, "peer": string, "mode": "test" | "prod-like" }` (extend as needed).
    - Behavior: uses `Tunnel` + `FileTransferManager` (same as CLI).
  * [ ] `GET /events` or WebSocket endpoint:
    - Streams events: `file_transfer_started`, `file_chunk_sent`, `file_transfer_complete`, `error`, etc.
* [ ] Implement or update backend handlers to:
  * [ ] Use the already-validated `Tunnel` APIs for file transfer (no custom crypto).
  * [ ] Emit structured logs/events consumable by the UI (no secret material).
* [ ] Update the web UI to:
  * [ ] Call `POST /api/send-file` with appropriate payload.
  * [ ] Display transfer status (file name, bytes sent/total, progress).
  * [ ] Surface final result and show hash-match success/failure if available.
* [ ] Create or update `docs/WEB_UI_GUIDE.md`:
  * [ ] Describe UI workflows (file send, status view).
  * [ ] Document API endpoints and minimum JSON shapes.
  * [ ] Provide example flow for sending a test file to a local peer.

**Acceptance Criteria**

* [ ] From the UI, a user can:
  * [ ] Select a file.
  * [ ] Provide/choose a peer endpoint.
  * [ ] Start a transfer and see status/logs in real time.
* [ ] Transfer uses the same record layer + file-transfer stack as the CLI.
* [ ] `WEB_UI_GUIDE.md` accurately describes the request/response format and UI behavior.

**Related**

* See `docs/MASTER_WEB_RELEASE_PROMPT.md` Section 3 for detailed guidance.
* CLI reference: `cli/src/main.rs` (`handle_send_file`, `handle_receive_file`).

