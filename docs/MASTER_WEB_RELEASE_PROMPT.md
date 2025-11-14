# MASTER_WEB_RELEASE_PROMPT — CrypRQ v1.0.1 Web-Only Release

You are acting as a **release engineer + senior protocol reviewer** for the CrypRQ v1.0.1 project.

Your job is to take the repository from "web stack implemented and documented" to a **clean, web-only v1.0.1 release candidate**, using ONLY the existing docs, code, and test instructions.

---

## SECTION 0 — Repository & Context Map

1. **Scan the repo** (code + docs) and build a short map of the pieces relevant to the web-only release:
 - Web backend code (CrypRQ record layer + HTTP/API layer).
 - Web frontend code (React/TypeScript).
 - Docker / Compose files (especially `docker-compose.web.yml`).
 - Validation docs:
 - `VALIDATION_RUN.md`
 - `WEB_VALIDATION_RUN.md`
 - `TEST_MATRIX.md`
 - Security & release docs:
 - `SECURITY_NOTES.md`
 - `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
 - Web guides:
 - `DOCKER_WEB_GUIDE.md`
 - `WEB_UI_GUIDE.md`
 - `WEB_STACK_QUICK_START.md`

2. Produce a **1–2 paragraph overview** that explains:
 - What the web-only stack currently does.
 - What is intentionally out-of-scope (handshake, identity, production hardening).

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

 | Area | Spec Ref | Implementation File(s) | Status (OK / WARN) | Notes |
 |----------------------|----------|-----------------------------|--------------------|-------|
 | Record header | … | … | | |
 | Nonce construction | … | … | | |
 | Key derivation | … | … | | |
 | Message type usage | … | … | | |

3. For any **WARN** rows, propose **small, concrete fixes** or doc clarifications.

---

## SECTION 2 — Web Validation Execution (WEB_VALIDATION_RUN.md)

1. Open `docs/WEB_VALIDATION_RUN.md` and:
 - List all test IDs (WEB-1, WEB-2, …).
 - Note which ones are:
 - Required for a web-only release.
 - Nice-to-have / extended coverage.
 - Can be postponed (but should be documented as such).

2. For each **required** test (e.g., WEB-1 minimal file transfer, WEB-2 medium file, WEB-3 error path, etc.):
 - Confirm the steps are executable with the current Docker/web setup.
 - If any step is ambiguous, propose a **small edit** to `WEB_VALIDATION_RUN.md` to make it crystal clear (but DO NOT change semantics).

3. Produce a **release-focused mini-matrix**:

 | Test ID | Name | Required? | Status (PASS/WARN/BLOCK) | Notes |
 |---------|-------------------------------|----------|--------------------------|-------|
 | WEB-1 | Minimal sanity (web) | | | |
 | WEB-2 | Medium file web transfer | | | |
 | WEB-3 | Error / failure behavior | | | |
 | WEB-4+ | … | / | | |

4. If any test is a **BLOCK** (e.g., reproducible bug), describe:
 - Exact symptom.
 - Likely root cause (based on logs/code).
 - Suggested minimal fix or tracking issue title.

---

## SECTION 3 — Docker & Web UI Sanity

Using:
- `docs/DOCKER_WEB_GUIDE.md`
- `docs/WEB_UI_GUIDE.md`
- `docs/WEB_STACK_QUICK_START.md`
- `docker-compose.web.yml`
- Frontend/Backend sources

1. Check that the **docs match reality**:
 - Ports in compose file vs ports mentioned in the docs.
 - Service names in compose vs logs/commands in the docs.
 - Environment variables expected vs documented.

2. Identify any **drifts** (even small ones) and propose doc edits such as:
 - "Change port in DOCKER_WEB_GUIDE.md from 8080 → 8081."
 - "Update WEB_UI_GUIDE.md to use `/api/send-file` instead of `/api/upload`," etc.

3. Validate that the **Web UI workflow** in `WEB_UI_GUIDE.md` matches:
 - The actual UI components (file picker, peer input, send button).
 - The backend endpoints actually implemented.

4. Produce a **short checklist**:
 - [ ] Docker guide matches compose file.
 - [ ] Web UI guide matches visible UI fields.
 - [ ] Quick start path (`WEB_STACK_QUICK_START.md`) is executable end-to-end.
 - [ ] No critical missing step between "git clone" and "see UI working".

---

## SECTION 4 — Security Posture & Disclaimers

Using:
- `SECURITY_NOTES.md`
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- Any inline comments/docs related to "test mode" and hardcoded keys.

1. Confirm the following are **explicitly and loudly documented**:
 - Web stack is **test mode**:
 - Static keys.
 - No handshake.
 - No peer auth.
 - Key-direction hack on the receiver in test mode.
 - Web stack is **NOT** production-ready.

2. Check that every user-facing doc that looks like a starting point (e.g., `README.md`, `WEB_STACK_QUICK_START.md`) has:
 - At least one clear line: 
 "This is a **testing / lab** configuration and MUST NOT be used in production."

3. Summarize the security posture in a table:

 | Area | Status (OK / RISK) | Notes / Required for Production |
 |-------------------|--------------------|---------------------------------|
 | Handshake | RISK | … |
 | Identity / auth | RISK | … |
 | Keys (test mode) | RISK | … |
 | Logging hygiene | OK / RISK | … |
 | Web/API hardening | OK / RISK | … |

4. Identify **MUST-FIX** items to move from "test-mode web demo" → "production candidate" (e.g. implement handshake, remove static keys, add identity, etc.), referencing `MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` when relevant.

---

## SECTION 5 — GitHub Release Packaging

Using `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` as the base:

1. Draft a **GitHub Release body** for a tag such as `v1.0.1-web-preview` that includes:
 - Short elevator pitch.
 - "What's included" bullet list.
 - "What's not included / limitations" bullet list.
 - Clear statement that this is a **web-only, test-mode, non-production preview**.
 - Pointers to:
 - `WEB_STACK_QUICK_START.md`
 - `DOCKER_WEB_GUIDE.md`
 - `WEB_UI_GUIDE.md`
 - `WEB_VALIDATION_RUN.md`
 - "Next steps / roadmap" bullets (e.g., handshake/identity implementation, production hardening).

2. Ensure that the release notes:
 - Do NOT oversell security.
 - Are honest and explicit about test mode and limitations.
 - Highlight the protocol work (ML-KEM + X25519, record layer) in a way that's understandable to a technical audience.

3. Provide the final release text as a **copy-paste-ready markdown block**.

---

## SECTION 6 — Verdict & Next Actions

1. Based on everything above, give a **clear verdict**:
 - `APPROVE_WEB_PREVIEW_RELEASE` 
 or 
 - `BLOCK_WEB_PREVIEW_RELEASE` (with reasons).

2. If **approved**, list the **concrete steps** to cut the release, for example:
 1. Ensure `VALIDATION_RUN.md` and `WEB_VALIDATION_RUN.md` are up-to-date.
 2. Tag repository: `git tag v1.0.1-web-preview` and `git push --tags`.
 3. Create GitHub release using the prepared body.
 4. Open issues for each MUST-FIX item before production.

3. If **blocked**, list:
 - The **minimum** set of fixes required.
 - Suggested GitHub issue titles and one-line descriptions for each.

4. Finish with a **short summary paragraph** that a future maintainer could read to understand:
 - What this release is.
 - How safe it is to use.
 - What work remains to reach production-grade.

---

## Quick Reference: Key Files to Inspect

- **Protocol Spec:** `cryp-rq-protocol-v1.md`
- **Record Layer:** `core/src/record.rs`, `node/src/record_layer.rs`
- **CLI Reference:** `cli/src/main.rs` (validated implementation)
- **Web Backend:** `web/server/` or equivalent
- **Web Frontend:** `web/` directory
- **Docker:** `docker-compose.web.yml`
- **Validation:** `docs/VALIDATION_RUN.md`, `docs/WEB_VALIDATION_RUN.md`
- **Security:** `docs/SECURITY_NOTES.md`
- **Release Notes:** `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`

---

**Remember:** The CLI path is validated and working. The web stack should behave identically, just with a web UI on top. If something works in CLI but not web, that's a bug to fix, not a design decision.
