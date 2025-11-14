# MASTER_CUT_THE_RELEASE_PROMPT — CrypRQ v1.0.1 Web-Only Preview Release

You are acting as a **release engineer** for the CrypRQ project.

Your job is to drive the v1.0.1 web-only preview release end-to-end, using only the documentation committed in the repository. Follow every step carefully, update the docs as you go, and produce a final human-readable summary at the end.

---

## SECTION 0 — Context & Goals

**Locate and open the following docs in the repo** (read-only first, no edits yet):

- `docs/CUT_THE_RELEASE.md`
- `docs/PRE_TAG_CHECKLIST.md`
- `docs/WEB_VALIDATION_RUN.md`
- `docs/VALIDATION_RUN.md`
- `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
- `docs/SECURITY_NOTES.md`
- `docs/WEB_STACK_QUICK_START.md`
- `README.md`

**Summarize in your own words:**

- What v1.0.1-web-preview is (CLI + web-only stack, test mode).
- The explicit non-production constraints (static keys, no handshake, no peer auth).
- The goal of this run: cut a web-only preview release tag and prepare the GitHub Release body, without changing any cryptographic behavior.

**Report that summary back to me before proceeding to the next sections.**

---

## SECTION 1 — Pre-Release Validation (PRE_TAG_CHECKLIST)

Using `docs/PRE_TAG_CHECKLIST.md` and `docs/CUT_THE_RELEASE.md` as the source of truth:

### 1.1 Mirror the Checklist

Extract all items from `PRE_TAG_CHECKLIST.md` into a structured list:

- CLI sanity
- Web path sanity (WEB-1 / WEB-2 tests)
- Docs coherence
- Security disclaimers
- Release metadata

For each item, write:

- **ID** (e.g., PRE-CLI-1)
- **Description**
- **Status:** TODO / PASS / FAIL / SKIP
- **Evidence needed** (logs, commands, files)

### 1.2 CLI Sanity (Already Validated but Re-Check Briefly)

Confirm `docs/VALIDATION_RUN.md` shows:

- Minimal sanity test for CLI file transfer is **PASS** (hash-verified).

If yes, set CLI sanity items to **PASS** and quote the relevant lines from `VALIDATION_RUN.md`.

If not, mark them **FAIL** and describe what's missing.

### 1.3 Web Path Sanity (WEB-1 / WEB-2)

Using `docs/WEB_VALIDATION_RUN.md` and `docs/TEST_MATRIX.md`:

**Identify:**

- **WEB-1:** Minimal web sanity (or equivalent).
- **WEB-2:** Basic web file transfer/logs (or equivalent).

**For each:**

- Summarize the expected steps and expected result.
- If the doc already has results:
  - Record the status (PASS/WARN/FAIL) and any notes.
- If not yet executed:
  - Propose the exact commands / steps a human would run (e.g., `docker compose …`, URL to open, what to check in the UI).
  - Update `WEB_VALIDATION_RUN.md` in-place (if you are allowed to modify files) with a simple table or bullet list:
    - Test ID
    - Date
    - Executor
    - Result
    - Notes

### 1.4 Docs Coherence

Cross-check these docs for consistency:

- `README.md`
- `WEB_STACK_QUICK_START.md`
- `DOCKER_WEB_GUIDE.md`
- `WEB_UI_GUIDE.md`
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`

**Verify that they all agree on:**

- How to start the web stack (commands, ports, main URL).
- Where the test-mode disclaimers appear.
- The naming of the release: `v1.0.1-web-preview`.

If you find minor inconsistencies, list them and propose exact wording changes, but **do not change any technical behavior** (no code changes, no new endpoints).

### 1.5 Security Disclaimers

Using `README.md`, `SECURITY_NOTES.md`, `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`, and `WEB_STACK_QUICK_START.md`:

**Confirm that all of them clearly state:**

- Static keys / no handshake / no peer auth.
- MUST NOT be used in production.

If any doc is weaker / unclear, propose stronger wording that:

- Is explicit about "test-mode" and "lab use only".
- Uses **MUST NOT** for production usage.

### 1.6 Release Metadata

Using `GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`:

**Verify:**

- Tag name: `v1.0.1-web-preview`
- Title clearly indicates "web-only" and "preview/test-mode"
- Sections: security notice, overview, what's included, limitations, quick start, validation status, roadmap.

If anything is missing/ambiguous, propose concrete text changes.

**At the end of this section, produce:**

- A table of `PRE_TAG_CHECKLIST` items with final statuses (PASS/WARN/FAIL).
- A short summary of any WARN/FAIL items and whether they block tagging.

---

## SECTION 2 — Tagging and Git Commands (Dry Plan)

Using `docs/CUT_THE_RELEASE.md` as the source of truth:

**Extract the exact git commands** the human operator should run to tag the release:

```bash
git tag -a v1.0.1-web-preview -m "CrypRQ v1.0.1 web-only preview (test mode)"
git push origin v1.0.1-web-preview
```

**Produce a copy-paste block with:**

- Preconditions (e.g., on the correct branch, clean working tree).
- The git commands.
- Expected outputs and how to verify the tag exists (`git tag`, GitHub UI).

**Do not claim you actually ran these commands; just produce a precise runbook for a human.**

---

## SECTION 3 — GitHub Release Body Verification

Open `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md` and:

**Confirm it:**

- Starts with a clear security notice.
- Describes the release accurately (web-only, preview, test-mode).
- Matches the validated docs (paths, commands, feature set).

**Sanity-check that:**

- No part implies this is "production ready".
- There's a clear "roadmap / next steps" pointing to handshake/identity/hardening.

**Generate a final GitHub-ready release body by:**

- Using the file as the base.
- Applying any minor wording improvements you identified earlier.
- Keeping the structure and technical content the same.

**Return that final body as a markdown block** I can paste directly into GitHub's "Release description" field.

---

## SECTION 4 — Final Verdict & Next Steps

Based on all previous sections:

**Provide a GO / NO-GO verdict for:**

- Tagging `v1.0.1-web-preview`
- Publishing a GitHub Release

**If GO:**

Restate the **3 human steps**, in order:

1. Run the git tag/push commands.
2. Create GitHub Release with the prepared body.
3. Share the link internally as a "web-only preview / test-mode build".

**If NO-GO:**

List:

- **MUST-FIX** items (blocking)
- **SHOULD-FIX** items (not blocking, but recommended)
- Suggest concrete follow-up tasks (with doc/code files named explicitly).

**Regardless of GO/NO-GO, outline the Next Phase after the preview release:**

- **Branch:** `feature/handshake-and-identity`
- **Driver doc:** `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`
- **Core goals:**
  - Replace static keys with real CrypRQ handshake.
  - Add peer identity (Ed25519 / peer IDs / PSK).
  - Remove test-mode hacks.
  - Prepare for a "non-test-mode" release line.

**Return everything as a well-structured markdown report** so I can read it and decide whether to cut the release right now.

---

## Quick Reference: Key Files

- **Release runbook:** `docs/CUT_THE_RELEASE.md`
- **Pre-tag checklist:** `docs/PRE_TAG_CHECKLIST.md`
- **Web validation:** `docs/WEB_VALIDATION_RUN.md`
- **CLI validation:** `docs/VALIDATION_RUN.md`
- **Release body:** `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
- **Release notes:** `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- **Security posture:** `docs/SECURITY_NOTES.md`

---

**Remember:** This is a **test-mode preview release**. Do not oversell security or production-readiness. Be explicit about limitations and the path forward to a production-ready release.

