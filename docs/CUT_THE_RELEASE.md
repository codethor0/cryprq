# Cut the Release — v1.0.1-web-preview

**Purpose:** Step-by-step runbook to cut the CrypRQ v1.0.1 web-only preview release.

**Prerequisites:** All structural work is complete (spec, engine, docs, prompts, checklists).

---

## Phase 1: Pre-Release Validation

### Step 1.1: Run PRE_TAG_CHECKLIST.md

Open `docs/PRE_TAG_CHECKLIST.md` and work through each section:

1. **CLI Sanity** (should already be ✅)
   - Confirm `VALIDATION_RUN.md` shows minimal sanity test: PASS
   - Verify CLI file transfer still works: `cryprq send-file` / `receive-file`

2. **Web Path Sanity**
   ```bash
   # Start web stack
   docker compose -f docker-compose.web.yml up --build
   
   # Run WEB-1 (minimal web file transfer)
   # - Send small file via Web UI
   # - Verify hash matches
   # - Check logs show FILE_META / FILE_CHUNK
   
   # Run WEB-2 (medium file) if time permits
   ```
   - Update `WEB_VALIDATION_RUN.md` with PASS/WARN/BLOCK status

3. **Docs Coherence**
   - [ ] Test `WEB_STACK_QUICK_START.md` from fresh clone → web UI working
   - [ ] Verify `DOCKER_WEB_GUIDE.md` ports match `docker-compose.web.yml`
   - [ ] Verify `WEB_UI_GUIDE.md` endpoints match actual backend

4. **Security Disclaimers**
   - [ ] Check `README.md` has "testing/lab use only" warning
   - [ ] Check `WEB_STACK_QUICK_START.md` has disclaimer
   - [ ] Check `SECURITY_NOTES.md` clearly states test-mode limitations
   - [ ] Check `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` has explicit warnings

5. **Release Metadata**
   - [ ] `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` is up to date
   - [ ] Prompt names resolved (no duplicates)

**Output:** Updated `PRE_TAG_CHECKLIST.md` with all items checked.

---

### Step 1.2: Optional — Web Stack Alignment Scan

Run `MASTER_WEB_ALIGNMENT_PROMPT.md` once via your AI/dev assistant:

- Scan for web/backend drift vs docs
- Fix any small nits found
- Document any larger issues (but don't block release for minor nits)

**Goal:** Catch doc drift before release, not after.

---

### Step 1.3: Run MASTER_WEB_RELEASE_PROMPT.md

Use `docs/MASTER_WEB_RELEASE_PROMPT.md` as the "release engineer brain":

1. **Section 1:** Verify protocol alignment for web stack
2. **Section 2:** Walk through `WEB_VALIDATION_RUN.md` execution
3. **Section 3:** Check Docker + web UI sanity
4. **Section 4:** Confirm security posture messaging
5. **Section 6:** Get final verdict (GO / NO-GO)

**Output:** Verdict from `MASTER_WEB_RELEASE_PROMPT.md`:
- `APPROVE_WEB_PREVIEW_RELEASE` → proceed to Phase 2
- `BLOCK_WEB_PREVIEW_RELEASE` → fix blocking issues, then retry

---

## Phase 2: Tag the Release

### Step 2.1: Final Pre-Tag Check

```bash
# Ensure you're on the correct branch (main/master)
git status

# Ensure all changes are committed
git diff --staged
git diff

# Verify build still works
cargo build --release -p cryprq
```

### Step 2.2: Create Tag

```bash
# Create annotated tag
git tag -a v1.0.1-web-preview -m "CrypRQ v1.0.1 web-only preview (test mode)"

# Verify tag
git show v1.0.1-web-preview

# Push tag to remote
git push origin v1.0.1-web-preview
```

**Tag Message Format:**
```
CrypRQ v1.0.1 web-only preview (test mode)

- Record layer + file transfer validated
- Web stack documented and wired
- Test mode only — NOT for production
- See docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md for details
```

---

## Phase 3: Create GitHub Release

### Step 3.1: Draft Release

1. Go to GitHub → **Releases** → **"Draft a new release"**

2. **Tag:** `v1.0.1-web-preview` (select from existing tags)

3. **Title:** `CrypRQ v1.0.1 — Web-Only Preview (Test Mode)`

4. **Body:** Copy entire contents from:
   ```
   docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md
   ```

5. **Release Type:** 
   - ☑ Pre-release (recommended for preview/test-mode releases)

6. **Attach Assets** (optional):
   - If you have pre-built binaries, attach them here
   - Otherwise, users build from source

### Step 3.2: Review and Publish

- Review the release body for accuracy
- Ensure security warnings are prominent
- Click **"Publish release"**

---

## Phase 4: Post-Release

### Step 4.1: Verify Release

- [ ] Tag exists: `git tag -l | grep v1.0.1-web-preview`
- [ ] GitHub release page is live
- [ ] Release body displays correctly
- [ ] Links to docs work

### Step 4.2: Announcement (Optional)

If announcing:
- Link to GitHub release page
- Emphasize: **test mode, not production**
- Point to `docs/WEB_STACK_QUICK_START.md` for getting started

### Step 4.3: Branch Strategy

**Preview Line (v1.0.1-web-preview):**
- Keep `main` branch stable
- Only allow:
  - Doc fixes
  - Small bugfixes
  - Tiny DX improvements
- No big changes

**Next Phase (Handshake/Identity):**
```bash
# Create feature branch
git checkout -b feature/handshake-and-identity

# Use MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md as driver
# Work on:
# - Real CRYPRQ CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH
# - Ed25519 / libp2p peer ID / PSK identity
# - Replace static test keys
# - Remove test-mode key direction hack
```

---

## Quick Reference: Key Files

- **Pre-tag checklist:** `docs/PRE_TAG_CHECKLIST.md`
- **Release validation:** `docs/MASTER_WEB_RELEASE_PROMPT.md`
- **Web alignment:** `docs/MASTER_WEB_ALIGNMENT_PROMPT.md`
- **Release body:** `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
- **Release notes:** `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- **Web validation:** `docs/WEB_VALIDATION_RUN.md`

---

## Troubleshooting

**Issue:** PRE_TAG_CHECKLIST shows blocking items
- **Fix:** Address blocking issues before proceeding
- **Don't:** Skip validation steps

**Issue:** MASTER_WEB_RELEASE_PROMPT returns BLOCK
- **Fix:** Review blocking reasons, fix issues, re-run
- **Don't:** Override the verdict without justification

**Issue:** Tag already exists
- **Fix:** Delete old tag: `git tag -d v1.0.1-web-preview && git push origin :refs/tags/v1.0.1-web-preview`
- **Then:** Re-run tag creation

**Issue:** GitHub release fails
- **Fix:** Check tag exists remotely, verify permissions
- **Fallback:** Create release manually via GitHub UI

---

## Success Criteria

✅ All PRE_TAG_CHECKLIST items pass  
✅ MASTER_WEB_RELEASE_PROMPT returns APPROVE  
✅ Tag `v1.0.1-web-preview` created and pushed  
✅ GitHub release published with correct body  
✅ Release is clearly marked as test-mode / not production  

**At this point:** You have a coherent, documented, reproducible web-only preview release that someone else could pick up and run.

---

**Last Updated:** (Update when cutting release)

