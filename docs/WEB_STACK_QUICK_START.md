# CrypRQ Web Stack — Quick Start for New Contributors

**File:** docs/WEB_STACK_QUICK_START.md  
**Audience:** New contributors who want to focus on the web-only stack  
**Goal:** Get you productive quickly without reconstructing the entire project history

> **TL;DR:** If you only care about the web stack, read these four docs in this order:

---

## Reading Order (Web Stack Focus)

### 1. **`WEB_ONLY_RELEASE_NOTES_v1.0.1.md`** (5 min read)
   - **Why:** Understand what the web stack is, what it includes, and what it's NOT (test mode only).
   - **Key takeaway:** This is a test-mode preview, not production-ready.

### 2. **`DOCKER_WEB_GUIDE.md`** (10 min read)
   - **Why:** Learn how to bring up the web stack with one command.
   - **Key takeaway:** `docker compose -f docker-compose.web.yml up --build` → open browser → test file transfer.

### 3. **`WEB_UI_GUIDE.md`** (10 min read)
   - **Why:** Understand how to use the web UI and what the API endpoints do.
   - **Key takeaway:** File transfer workflow, event streaming, API contract.

### 4. **`WEB_VALIDATION_RUN.md`** (15 min read)
   - **Why:** See the test matrix and validation status for the web stack.
   - **Key takeaway:** What tests exist, what's passing, what needs work.

---

## Optional Deep Dives

If you want to understand the underlying protocol:

- **`cryp-rq-protocol-v1.md`** — Full protocol specification (v1.0.1)
- **`VALIDATION_RUN.md`** — CLI validation (shows the record layer works end-to-end)
- **`SECURITY_NOTES.md`** — Security posture and limitations

If you want to contribute code:

- **`MASTER_WEB_ALIGNMENT_PROMPT.md`** — Master prompt for aligning web stack with v1.0.1
- **`MASTER_WEB_RELEASE_PROMPT.md`** — Master prompt for web-only release engineering
- **`.github/ISSUES/001-004.md`** — GitHub issue templates for web stack work

---

## Quick Test (After Reading Guides)

1. **Start the stack:**
   ```bash
   docker compose -f docker-compose.web.yml up --build
   ```

2. **Open UI:** `http://localhost:3000` (or whatever port is configured)

3. **Send a test file:**
   - Create: `echo "test" > test.bin`
   - Use UI to send it
   - Verify hash matches

4. **If it works:** ✅ You're ready to contribute!

5. **If it doesn't:** Check `DOCKER_WEB_GUIDE.md` troubleshooting section.

---

## What You're NOT Reading (For Now)

- CLI implementation details (`cli/src/main.rs`)
- VPN/TUN mode specifics
- Handshake implementation (not done yet)
- Production deployment guides (not applicable to test mode)

These are documented elsewhere if you need them later.

---

## Next Steps After Quick Start

- **Want to validate the web stack?** → Use `WEB_VALIDATION_RUN.md` test matrix
- **Want to align web backend with record layer?** → See `.github/ISSUES/001-align-web-backend-record-layer.md`
- **Want to wire up new UI features?** → See `.github/ISSUES/002-wire-web-ui-file-transfer.md`
- **Want to understand the protocol?** → Read `cryp-rq-protocol-v1.md`

---

**Remember:** The CLI path is validated and working. The web stack should behave identically, just with a web UI on top. If something works in CLI but not web, that's a bug to fix, not a design decision.

