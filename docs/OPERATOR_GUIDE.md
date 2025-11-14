# Operator Guide — Cutting v1.0.1-web-preview Release

**Purpose:** Short, operator-level game plan for cutting the web-only preview release and starting the next phase.

**Status:** All release infrastructure is in place. This guide provides the concrete steps to execute.

---

## Path 1: Cut v1.0.1-web-preview Today

### Step 1: Run Pre-Tag Checklist (Quick Pass)

From `docs/PRE_TAG_CHECKLIST.md`, confirm:

#### 1.1 CLI Sanity 
- Check `docs/VALIDATION_RUN.md` → minimal sanity test is marked **PASS** (hash-verified file transfer).
- If that's still true, engine is good.

#### 1.2 Web Sanity (WEB-1 / WEB-2)
Follow tests in `docs/WEB_VALIDATION_RUN.md`:

```bash
# Bring up web stack
docker compose -f docker-compose.web.yml up --build

# Hit the web UI (see DOCKER_WEB_GUIDE.md for URL)
# Do a simple file transfer & confirm:
# - UI shows success
# - File appears on disk with correct SHA-256
```

**Update `WEB_VALIDATION_RUN.md`** for each test:
- Date
- Result (PASS/WARN)
- One-line note

#### 1.3 Docs + Disclaimers Sanity
Skim these files and confirm they all clearly say:
- Static keys
- No handshake
- No peer auth
- **MUST NOT be used in production**

Files to check:
- `README.md`
- `WEB_STACK_QUICK_START.md`
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- `SECURITY_NOTES.md`

**If all checks pass → PRE_TAG_CHECKLIST is **

---

### Step 2: Tag the Release

**Preconditions:**
```bash
# Make sure you're on the correct branch (main or release/v1.0.1-web-preview)
git status

# Ensure working tree is clean
git diff
```

**Create and push tag:**
```bash
# Create annotated tag
git tag -a v1.0.1-web-preview -m "CrypRQ v1.0.1 web-only preview (test mode)"

# Push tag to remote
git push origin v1.0.1-web-preview

# Sanity check locally
git tag | grep v1.0.1
```

**If that all passes: tag is live.**

---

### Step 3: Create GitHub Release

1. Go to **GitHub → Releases → Draft new release**

2. **Tag:** `v1.0.1-web-preview`

3. **Title:** `CrypRQ v1.0.1 — Web-Only Preview (Test Mode, Non-Production)`

4. **Description:** Copy entire contents of:
   ```
   docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md
   ```

5. **Double-check** that the very first lines clearly state:
 - Test mode
 - Static keys
 - Not for production

6. **Publish the release**

**Result:** Preview release is officially out.

---

## Path 2: Start Next Phase (Handshake + Identity)

**After the preview is tagged and released:**

### Step 4: Create Next-Phase Branch

```bash
git checkout -b feature/handshake-and-identity
```

**This branch's mission:** Kill test-mode hacks and implement the real protocol plumbing.

### Step 5: Drive Work with Master Prompt

Use `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` as the driver for:

#### 5.1 Real Handshake
- Implement `CRYPRQ_CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH` on the wire
- Plug in the v1.0.1 key schedule (HKDF, epochs, proper `keys_ir`/`keys_ri`)

#### 5.2 Identity / Auth
- Decide on initial identity scheme (e.g., Ed25519 keys or libp2p-style peer IDs / PSK)
- Make sure the handshake actually authenticates peers, not just encrypts

#### 5.3 Direction Correctness
- Replace the "test-mode: both sides are initiator using `keys_outbound`" fix with proper role semantics

#### 5.4 Removing Test-Mode
- Remove static keys
- Remove any "bypass handshake" logic
- Tighten `SECURITY_NOTES.md` to split:
 - web-preview line (test mode)
 - future non-test-mode line

**Note:** You already have the protocol spec and KDF/record layer aligned — this phase is mostly: "Make the running system behave exactly like the spec, instead of spec emulating the test harness."

---

## Current Status Summary

### Complete
- **Engine & record layer:** Implemented and validated (CLI file transfer with SHA-256 verification)
- **Web stack docs:** Ready (web guides, Docker guide, validation docs, release notes)
- **Release infrastructure:** All in place:
 - `CUT_THE_RELEASE.md`
 - `PRE_TAG_CHECKLIST.md`
 - `GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
 - `MASTER_CUT_THE_RELEASE_PROMPT.md`

### Security Posture
- Clearly labeled **test-mode only** (static keys, no handshake, no peer auth)

---

## Two Action Paths

### Operator Hat → Cut Release
Follow `docs/CUT_THE_RELEASE.md` and actually ship `v1.0.1-web-preview`.

**Quick path:** Follow Steps 1-3 above.

### Protocol Engineer Hat → Next Phase
Branch and start the handshake/identity phase.

**Quick path:** Follow Steps 4-5 above.

---

## Quick Reference

**Release Runbook:** `docs/CUT_THE_RELEASE.md` 
**Pre-Tag Checklist:** `docs/PRE_TAG_CHECKLIST.md` 
**Release Body:** `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md` 
**Master Release Prompt:** `docs/MASTER_CUT_THE_RELEASE_PROMPT.md` 
**Next Phase Prompt:** `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`

---

**Last Updated:** (Update when cutting release or starting next phase)

