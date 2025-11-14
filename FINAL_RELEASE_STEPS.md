# Final Release Steps — v1.0.1-web-preview

**Status:** Code committed, ready for final release steps.

**Last Updated:** $(date)

---

## ✅ Step 1: Complete — Code Changes Committed

All code changes have been committed:
- **Commit:** `a2439be` - "chore: prep for v1.0.1-web-preview (code + infra)"
- **Git Status:** Clean (only untracked files remain, won't block tagging)

---

## 📋 Step 2: Web Smoke Test (Manual - Required)

**Before tagging, complete this manual test:**

### 2.1 Start Web Stack

```bash
docker compose -f docker-compose.web.yml up --build
```

### 2.2 Test File Transfer

1. **Open browser:** `http://localhost:8787`
2. **Send test file:** Use Web UI to send `/tmp/testfile.bin`
3. **Verify hash:**
   ```bash
   sha256sum /tmp/testfile.bin /tmp/receive/testfile.bin
   ```
   **Expected:** `6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec`

### 2.3 Update Validation Doc

```bash
./scripts/update-web-validation.sh \
  WEB-1 PASS "$(date +%Y-%m-%d)" \
  "testfile.bin" \
  "6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec" \
  "matches CLI minimal sanity"
```

**Optional:** If you want WEB-2 explicitly tracked, repeat with WEB-2.

---

## 📋 Step 3: Run Preflight Script & Create Tag

### 3.1 Dry Run (Verify)

```bash
./scripts/preflight-and-tag.sh --dry-run
```

**Expected output:**
- ✅ CLI validation: PASS
- ✅ Web validation: WEB-1 marked PASS (after Step 2)
- ✅ Security disclaimers: All 4 files found
- ✅ Git status: Clean

### 3.2 Execute (Creates Tag)

```bash
./scripts/preflight-and-tag.sh
```

**What it does:**
- Re-checks all validations
- Creates annotated tag: `v1.0.1-web-preview`
- Pushes tag to remote
- Prints GitHub release instructions

**Result:** Tag `v1.0.1-web-preview` is live on remote.

---

## 📋 Step 4: Create GitHub Release (Manual UI)

### 4.1 Navigate to GitHub

1. Go to your GitHub repo
2. Click **"Releases"** → **"Draft a new release"**

### 4.2 Configure Release

- **Tag:** Select `v1.0.1-web-preview` from dropdown
- **Title:** `CrypRQ v1.0.1 — Web-Only Preview (Test Mode)`
- **Description:** 
  1. Open `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
  2. Copy **entire contents**
  3. Paste into GitHub release description field

### 4.3 Verify Security Warnings

Double-check the top of the release body clearly states:
- ✅ static keys
- ✅ no handshake
- ✅ no peer auth
- ✅ **NOT FOR PRODUCTION**

### 4.4 Publish

Click **"Publish release"**

**Result:** Web-only preview release is officially published. 🎉

---

## ✅ Step 5: Switch to Next Phase

### 5.1 Switch to Feature Branch

```bash
git checkout feature/handshake-and-identity
git merge main    # or rebase if you prefer clean history
```

### 5.2 Start Handshake/Identity Work

Open `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` and use it to guide:

1. **Real CrypRQ Handshake:**
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
./scripts/update-web-validation.sh WEB-1 PASS "$(date +%Y-%m-%d)" "testfile.bin" "hash..." "note"

# Preflight (dry run)
./scripts/preflight-and-tag.sh --dry-run

# Preflight (execute)
./scripts/preflight-and-tag.sh

# Switch to next phase
git checkout feature/handshake-and-identity
git merge main
```

---

## Success Checklist

- [x] Step 1: Code changes committed
- [ ] Step 2: Web smoke test completed (WEB-1 marked PASS)
- [ ] Step 3: Preflight script executed (tag created)
- [ ] Step 4: GitHub release published
- [ ] Step 5: Feature branch ready for next phase

---

**Current Status:** Ready for Step 2 (web smoke test).

