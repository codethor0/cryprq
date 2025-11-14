# Release Execution — Execute Now

**Status:** Ready to execute. Follow these steps in order.

---

## ⚠️ Pre-Flight: Uncommitted Changes

There are uncommitted changes in the repo. You have two options:

**Option A: Commit changes first (recommended)**
```bash
git add -A
git commit -m "docs: add release infrastructure and validation docs"
```

**Option B: Proceed with uncommitted changes**
The preflight script will warn but allow you to continue.

---

## Step 1: Web Smoke Test (Manual - ~5 minutes)

### 1.1 Start Web Stack

```bash
# From repo root
docker compose -f docker-compose.web.yml up --build
```

**Expected:** Web stack starts, shows logs from both frontend and backend.

### 1.2 Test File Transfer

1. **Open browser:** `http://localhost:8787` (or `http://localhost:5173` if using Vite dev server)

2. **Send a test file:**
   - Use the Web UI to select and send a small test file
   - Example: Create `/tmp/web-test.bin` with content: `"Test file for CrypRQ web v1.0.1"`

3. **Verify receipt:**
   - Check the configured output directory (see `DOCKER_WEB_GUIDE.md`)
   - Run: `sha256sum /tmp/web-test.bin /path/to/received/file`
   - **Expected:** Identical hashes

### 1.3 Update Validation Doc

**Option A: Use helper script**
```bash
./scripts/update-web-validation.sh WEB-1 PASS "2025-11-14" "web-test.bin" "6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec" "matches CLI minimal sanity hash behavior"
```

**Option B: Manual edit**
Edit `docs/WEB_VALIDATION_RUN.md`:
- Mark WEB-1 as ✅ PASS
- Add date, file name, SHA-256, and note

---

## Step 2: Run Preflight Script

### 2.1 Dry Run (Preview)

```bash
./scripts/preflight-and-tag.sh --dry-run
```

**Expected output:**
- ✅ CLI validation: PASS
- ⚠️ Web validation: TODO (if not updated) or PASS
- ✅ Security disclaimers: All present
- ⚠️ Git status: Uncommitted changes (if any)

### 2.2 Execute (Creates Tag)

```bash
./scripts/preflight-and-tag.sh
```

**What happens:**
1. Confirms CLI validation ✅
2. Checks web validation (warns if TODO, allows override)
3. Verifies security disclaimers
4. Checks git status (warns if dirty)
5. Creates tag: `v1.0.1-web-preview`
6. Pushes tag to remote
7. Prints GitHub release instructions

**Result:** Tag `v1.0.1-web-preview` is live on remote.

---

## Step 3: Create GitHub Release (Manual - ~2 minutes)

### 3.1 Navigate to GitHub

1. Go to your GitHub repo
2. Click **"Releases"** → **"Draft a new release"**

### 3.2 Configure Release

- **Tag:** Select `v1.0.1-web-preview` from dropdown
- **Title:** `CrypRQ v1.0.1 — Web-Only Preview (Test Mode, Non-Production)`
- **Description:** 
  1. Open `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
  2. Copy **entire contents**
  3. Paste into GitHub release description field

### 3.3 Verify Security Warnings

Double-check the top of the release body clearly states:
- ✅ static keys
- ✅ no handshake
- ✅ no peer auth
- ✅ **NOT FOR PRODUCTION**

### 3.4 Publish

Click **"Publish release"**

**Result:** Web-only preview release is officially published. 🎉

---

## Step 4: Create Next-Phase Branch

### 4.1 Create Feature Branch

```bash
# From repo root
git checkout -b feature/handshake-and-identity
```

### 4.2 Start Next Phase

Open `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` and use it to guide:

1. **Real Handshake:**
   - Implement `CRYPRQ_CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH`
   - Plug in v1.0.1 key schedule (HKDF, epochs, proper `keys_ir`/`keys_ri`)

2. **Peer Identity & Authentication:**
   - Decide on identity scheme (Ed25519 / libp2p peer IDs / PSK)
   - Ensure handshake authenticates peers

3. **Fix Direction Correctness:**
   - Remove "both sides are initiator, decrypt with `keys_outbound`" hack
   - Implement proper initiator/responder role semantics

4. **Remove Test-Mode Hacks:**
   - Remove static keys
   - Remove "bypass handshake" logic
   - Tighten `SECURITY_NOTES.md` for production posture

---

## Quick Command Reference

```bash
# Web smoke test
docker compose -f docker-compose.web.yml up --build

# Update validation (after test)
./scripts/update-web-validation.sh WEB-1 PASS "2025-11-14" "testfile.bin" "hash..." "note"

# Preflight (dry run)
./scripts/preflight-and-tag.sh --dry-run

# Preflight (execute)
./scripts/preflight-and-tag.sh

# Create next-phase branch
git checkout -b feature/handshake-and-identity
```

---

## Success Checklist

- [ ] Web smoke test completed (WEB-1 marked PASS)
- [ ] `docs/WEB_VALIDATION_RUN.md` updated
- [ ] Preflight script passes all checks
- [ ] Tag `v1.0.1-web-preview` created and pushed
- [ ] GitHub release published with correct body
- [ ] Feature branch `feature/handshake-and-identity` created

**At this point:** Preview release is live, and next-phase work can begin.

---

**Created:** $(date)

