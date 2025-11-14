# Release Execution Status — v1.0.1-web-preview

**Last Updated:** $(date)

---

## ✅ Step 1: Web Stack Started

**Status:** ✅ Running

- **Docker Compose:** Started successfully
- **Container:** `cryprq-web` is running
- **Web UI:** http://localhost:8787
- **Ports:** 
  - 8787 (backend API)
  - 5173 (Vite dev server)

**Test File Ready:**
- **Path:** `/tmp/testfile.bin`
- **Hash:** `6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec` ✅

---

## 📋 Manual Action Required: Web Smoke Test

**Complete these steps:**

1. **Open browser:** http://localhost:8787

2. **Use Web UI to send:** `/tmp/testfile.bin`

3. **Verify received file:**
   ```bash
   sha256sum /tmp/testfile.bin /tmp/receive/testfile.bin
   ```
   **Expected:** `6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec`

4. **Update validation doc:**
   ```bash
   ./scripts/update-web-validation.sh WEB-1 PASS "$(date +%Y-%m-%d)" "testfile.bin" "6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec" "matches CLI minimal sanity"
   ```

---

## ⏭️ Step 2: Complete Release (After Web Test)

**After completing Step 1, run:**

```bash
./scripts/complete-release.sh
```

**This script will:**
- ✅ Run preflight checks (CLI validation, web validation, disclaimers)
- ✅ Create and push tag: `v1.0.1-web-preview`
- ✅ Guide you through GitHub Release creation
- ✅ Switch to `feature/handshake-and-identity` branch
- ✅ Merge main into feature branch

---

## 📋 Step 3: GitHub Release (Manual UI)

**When `complete-release.sh` prompts you:**

1. Go to **GitHub → Releases → Draft new release**
2. Select tag: `v1.0.1-web-preview`
3. Title: `CrypRQ v1.0.1 — Web-Only Preview (Test Mode)`
4. Copy body from: `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
5. Verify test-mode warnings are visible
6. Click **"Publish release"**

---

## ✅ Step 4: Next Phase (After Release)

**After release is published:**

You'll be on `feature/handshake-and-identity` branch.

**Open:** `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`

**Use that to guide:**
- Real CrypRQ handshake (CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH)
- Peer identity and authentication
- Remove static test keys
- Remove "both sides are initiator" test hack
- Update SECURITY_NOTES.md for production posture

---

## Quick Command Reference

```bash
# Check web stack status
docker compose -f docker-compose.web.yml ps

# View web stack logs
docker compose -f docker-compose.web.yml logs -f

# Stop web stack (after testing)
docker compose -f docker-compose.web.yml down

# After web test: Complete release
./scripts/complete-release.sh
```

---

## Current Status Summary

- ✅ **Web stack:** Running
- ✅ **Test file:** Ready
- 📋 **Web test:** Manual action required (browser)
- ⏭️ **Release script:** Ready to run after web test
- ⏭️ **GitHub release:** Manual UI step
- ✅ **Feature branch:** Ready for next phase

**Next Action:** Complete web smoke test in browser, then run `./scripts/complete-release.sh`

