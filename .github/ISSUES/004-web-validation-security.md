# Web Validation Matrix & Security Posture (Test Mode)

**Summary**

Extend the existing validation and security documentation to explicitly cover the web-only CrypRQ v1.0.1 stack and clarify that it is **test/demo only**, not production.

**Context**

* Existing docs:
  * `TEST_MATRIX.md`
  * `VALIDATION_RUN.md`
  * `SECURITY_NOTES.md`
  * `MASTER_VALIDATION_PROMPT.md`
* Web stack now needs its own validation run + explicit security stance (test-mode static keys, no handshake, no peer auth, key-direction hack).

**Tasks**

* [ ] Create `docs/WEB_VALIDATION_RUN.md`:
  * [ ] Mirror the format of `VALIDATION_RUN.md`.
  * [ ] Define tests:
    - WEB-1: Minimal web loopback file transfer (test file).
    - WEB-2: Medium file test.
    - WEB-3: Concurrent transfers (if supported).
    - WEB-4: Optional CLI↔web mixed transfer test.
  * [ ] Provide space for:
    - Inputs, steps, expected results.
    - Actual results, PASS/FAIL.
    - Log references.
* [ ] Update `TEST_MATRIX.md`:
  * [ ] Add a section referencing the web-only tests and `WEB_VALIDATION_RUN.md`.
* [ ] Update `SECURITY_NOTES.md`:
  * [ ] Add a **"Web-Only Mode"** section that explicitly states:
    - Uses static test keys and no peer authentication.
    - Test-mode key-direction hack for receiver (both peers acting as initiator).
    - Safe for local testing/demo, NOT for production or hostile networks.
* [ ] Create `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`:
  * [ ] List what's included (web stack, record-layer-aligned file transfer).
  * [ ] List what is test-only and MUST NOT be considered production.
  * [ ] Enumerate MUST-FIX items for production:
    - Real handshake (`CRYPRQ_CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH`).
    - Proper initiator/responder role-based keys and directions.
    - Peer identity & authentication.
    - Removal of test-mode hacks and static keys.
* [ ] Append a short verdict section to `MASTER_VALIDATION_PROMPT.md` or `MASTER_WEB_RELEASE_PROMPT.md` (if present):
  * [ ] Summarize web-only validation status.
  * [ ] Link to `WEB_VALIDATION_RUN.md`.
  * [ ] State the security stance in 2–3 sentences.

**Acceptance Criteria**

* [ ] `WEB_VALIDATION_RUN.md` contains a defined set of web-only tests and is in a ready-to-run format.
* [ ] `TEST_MATRIX.md` clearly references web validation.
* [ ] `SECURITY_NOTES.md` explicitly warns against using web-only mode as a production VPN / secure file-transfer solution.
* [ ] `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` can be pasted directly into a GitHub release or changelog.
* [ ] Final verdict text (test-only vs production) is present and unambiguous.

**Related**

* See `docs/MASTER_WEB_RELEASE_PROMPT.md` Sections 5, 6, and 7 for detailed guidance.
* Reference: `docs/VALIDATION_RUN.md` (CLI validation format).

