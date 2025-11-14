# MASTER_WEB_ALIGNMENT_PROMPT — Web Stack Alignment with CrypRQ v1.0.1

You are acting as a **senior protocol engineer + web stack architect** for the CrypRQ v1.0.1 project.

Your job is to ensure the **web backend and frontend** are fully aligned with the CrypRQ v1.0.1 protocol specification and use the same validated record layer as the CLI path.

---

## SECTION 0 — Repository & Context Map

1. **Scan the repo** (code + docs) and build a short map of the pieces relevant to web stack alignment:
   - Web backend code (CrypRQ record layer + HTTP/API layer).
   - Web frontend code (React/TypeScript).
   - CLI reference implementation (already validated):
     - `cli/src/main.rs` — CLI file transfer using record layer
     - `node/src/lib.rs` — Tunnel + record layer APIs
     - `core/src/record.rs` — Record header + encryption
     - `node/src/record_layer.rs` — Record send/recv logic
   - Protocol specification:
     - `cryp-rq-protocol-v1.md` (or equivalent)
   - Alignment docs:
     - `PROTOCOL_ALIGNMENT_REPORT.md` (if exists)
     - `VALIDATION_RUN.md` — CLI validation (reference implementation)

2. Produce a **1–2 paragraph overview** that explains:
   - What the web stack currently does vs what the CLI does.
   - Where alignment gaps exist (if any).

---

## SECTION 1 — Spec & Protocol Alignment (Web Path Only)

1. Using the protocol spec (v1.0.1) and any alignment docs (`PROTOCOL_ALIGNMENT_REPORT.md`, record layer files, etc.), verify that the **web backend's data path** matches the spec for:

   - **Record header:**
     - 20-byte header structure.
     - Fields: version, type, flags, epoch (u8), stream ID, sequence number, ciphertext length (4 bytes).

   - **Nonce construction:**
     - TLS 1.3–style XOR of static IV with sequence number.

   - **Key schedule:**
     - HKDF with the correct salts and labels.
     - Epoch-scoped keys.

   - **Message types used by the web stack:**
     - `FILE_META`, `FILE_CHUNK`, `FILE_ACK`, `CONTROL` (if used), `DATA` (if used), `VPN_PACKET` (if any is wired through in web mode).

2. Summarize in a small table:

   | Area                 | Spec Ref | Implementation File(s)      | Status (OK / WARN) | Notes |
   |----------------------|----------|-----------------------------|--------------------|-------|
   | Record header        | …        | …                           |                    |       |
   | Nonce construction   | …        | …                           |                    |       |
   | Key derivation       | …        | …                           |                    |       |
   | Message type usage   | …        | …                           |                    |       |

3. For any **WARN** rows, propose **small, concrete fixes** or doc clarifications.

---

## SECTION 2 — Web Backend Refactoring Plan

1. **Map the current web backend architecture:**
   - Identify all backend entrypoints (e.g., `/connect`, `/events`, `/api/send-file`).
   - Document which modules handle sessions/tunnels.
   - Note any remaining libp2p / legacy protocol usage.

2. **Identify all backend paths that send/receive network data for:**
   - VPN/TUN packets (if applicable in web mode).
   - File transfer.
   - Generic DATA/control messages (if applicable).

3. **Design the refactoring plan:**
   - All outbound data should go through `Tunnel` + record layer APIs:
     - `Tunnel::send_vpn_packet()` for VPN
     - `Tunnel::send_record()` / file-transfer helpers for files
   - All inbound data should flow through `Tunnel::recv_record()` → `handle_incoming_record()`.
   - Remove any direct libp2p request-response framing or legacy headers.

4. **Produce a concrete step-by-step refactoring plan:**
   - List files to modify.
   - List functions to replace or refactor.
   - List tests to update or add.

---

## SECTION 3 — Frontend Alignment

1. **Verify frontend API contracts match backend:**
   - Check that frontend calls match the actual backend endpoints.
   - Verify JSON payload shapes are consistent.
   - Confirm WebSocket/SSE event formats match backend output.

2. **Ensure frontend displays protocol-level information correctly:**
   - Stream IDs (if shown).
   - File transfer progress (bytes, chunks).
   - Error messages (should not leak keys/nonces).

3. **Propose any frontend updates needed** to align with the v1.0.1 backend changes.

---

## SECTION 4 — Testing & Validation

1. **Define alignment tests:**
   - Web backend should produce identical record-layer behavior as CLI.
   - File transfer via web UI should use the same record format as CLI.
   - Logs should show the same protocol-level events (FILE_META, FILE_CHUNK, etc.).

2. **Create or update test plan:**
   - Add tests to `WEB_VALIDATION_RUN.md` that verify protocol alignment.
   - Ensure tests can be run via Docker/web stack.

3. **Document expected behavior:**
   - What logs should appear when web backend sends a file?
   - How should web backend errors be surfaced to the UI?

---

## SECTION 5 — Documentation Updates

1. **Update alignment documentation:**
   - Create or update `docs/WEB_STACK_MAP.md` with the new architecture.
   - Update `docs/PROTOCOL_ALIGNMENT_REPORT.md` (if exists) to include web path.

2. **Update user-facing docs:**
   - Ensure `WEB_UI_GUIDE.md` reflects the aligned backend behavior.
   - Update `DOCKER_WEB_GUIDE.md` if backend behavior changes.

---

## SECTION 6 — Verdict & Next Steps

1. Based on everything above, provide a **clear verdict**:
   - `WEB_STACK_ALIGNED` — Web backend matches CLI/record layer behavior.
   - `WEB_STACK_NEEDS_REFACTORING` — Gaps identified, refactoring plan provided.

2. If **aligned**, confirm:
   - Web backend uses the same record layer as CLI.
   - File transfer via web UI produces identical wire format as CLI.
   - No legacy libp2p framing remains in the web path.

3. If **needs refactoring**, list:
   - Concrete steps to align (from Section 2).
   - Estimated complexity (low/medium/high).
   - Suggested GitHub issue titles.

---

## Quick Reference: Key Files to Inspect

- **Protocol Spec:** `cryp-rq-protocol-v1.md`
- **Record Layer:** `core/src/record.rs`, `node/src/record_layer.rs`
- **CLI Reference:** `cli/src/main.rs` (validated implementation)
- **Web Backend:** `web/server/` or equivalent
- **Web Frontend:** `web/` directory
- **GitHub Issue:** `.github/ISSUES/001-align-web-backend-record-layer.md`

---

**Remember:** The CLI path is validated and working. The web stack should behave identically, just with a web UI on top. If something works in CLI but not web, that's a bug to fix, not a design decision.

