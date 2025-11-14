# Pre-Tag Checklist for v1.0.1-web-preview

**Purpose:** Run this checklist before tagging `v1.0.1-web-preview` to ensure the release is ready.

---

## 1. CLI Sanity (Already Validated)

- [x] `VALIDATION_RUN.md` shows minimal sanity test: **PASS** (CLI send/receive with hash match)
- [x] File transfer via CLI still passes with current main branch

**Status:** CLI path validated and stable

---

## 2. Web Path Sanity

- [ ] Run at least **WEB-1** (minimal web file transfer) from `WEB_VALIDATION_RUN.md`
 - [ ] Start web stack: `docker compose -f docker-compose.web.yml up --build`
 - [ ] Send a small file via Web UI
 - [ ] Verify file received correctly and SHA-256 hash matches
 - [ ] Check logs show FILE_META / FILE_CHUNK flow

- [ ] Run **WEB-2** (medium file web transfer) if time permits
 - [ ] Send a medium file (~10 MB) via Web UI
 - [ ] Verify transfer completes and hash matches

- [ ] Update `WEB_VALIDATION_RUN.md` with actual **PASS/WARN/BLOCK** status for:
 - [ ] WEB-1: Minimal sanity (web)
 - [ ] WEB-2: Medium file web transfer
 - [ ] Any other tests executed

**Status:** Web path validation (update after running tests)

---

## 3. Docs Coherence

- [ ] **WEB_STACK_QUICK_START.md** can be followed cleanly from fresh clone → web UI working
 - [ ] Test: Clone repo, follow quick start, verify web UI loads
 - [ ] No missing steps or broken commands

- [ ] **DOCKER_WEB_GUIDE.md** ports & service names match `docker-compose.web.yml`
 - [ ] Check port mappings (frontend, backend, UDP)
 - [ ] Check service names (web-backend, web-frontend, etc.)
 - [ ] Check environment variable names

- [ ] **WEB_UI_GUIDE.md** endpoints & UI flows match the current frontend/backend
 - [ ] Verify endpoint paths (`/api/send-file`, `/events`, etc.)
 - [ ] Verify UI component descriptions match actual UI
 - [ ] Verify JSON payload examples match backend expectations

**Status:** Docs coherence check (verify each item)

---

## 4. Security Posture Explicitly Loud

- [ ] **SECURITY_NOTES.md** clearly states:
 - [ ] Static keys / test mode
 - [ ] No handshake
 - [ ] No peer auth
 - [ ] Key-direction hack in test mode

- [ ] **README.md** has clear disclaimer:
 - [ ] "This configuration is for testing/lab use only and MUST NOT be used in production."

- [ ] **WEB_STACK_QUICK_START.md** has clear disclaimer:
 - [ ] "This configuration is for testing/lab use only and MUST NOT be used in production."

- [ ] **WEB_ONLY_RELEASE_NOTES_v1.0.1.md** explicitly states test-mode limitations

**Status:** Security disclaimers check (verify each doc)

---

## 5. Release Metadata

- [ ] **WEB_ONLY_RELEASE_NOTES_v1.0.1.md** is up to date
 - [ ] Includes all features included in this release
 - [ ] Lists all limitations clearly
 - [ ] Points to correct documentation files

- [ ] Duplicate prompt name issue resolved:
 - [x] `MASTER_WEB_ALIGNMENT_PROMPT.md` exists (web stack alignment)
 - [x] `MASTER_WEB_RELEASE_PROMPT.md` exists (release engineering)
 - [x] References updated in `WEB_STACK_QUICK_START.md`

**Status:** Release metadata ready

---

## 6. Tag & Release

- [ ] Tag planned: `v1.0.1-web-preview`
 - [ ] Tag message prepared (short summary)
 - [ ] Tag points to commit with all above items complete

- [ ] GitHub release body ready:
 - [ ] Use the release body from `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md` (or see below)
 - [ ] Review for accuracy and completeness

**Status:** Ready to tag (after all above items pass)

---

## Quick Commands

```bash
# 1. Build CLI
cargo build --release -p cryprq

# 2. Start web stack
docker compose -f docker-compose.web.yml up --build

# 3. Run web validation (follow WEB_VALIDATION_RUN.md)

# 4. Verify docs
grep -r "MUST NOT be used in production" README.md docs/WEB_STACK_QUICK_START.md docs/SECURITY_NOTES.md

# 5. Create tag
git tag -a v1.0.1-web-preview -m "Web-only preview release (test mode)"
git push --tags

# 6. Create GitHub release using the prepared body
```

---

## Final Verdict

- [ ] **APPROVE** — All items checked, ready to tag and release
- [ ] **BLOCK** — Issues found (list below)

**Blocking Issues:**
- (List any blocking issues here)

---

**Last Updated:** (Update this date when running the checklist)

