# Release Now — v1.0.1-web-preview

**Quick execution guide — follow these steps in order.**

---

## Step 1: Web Smoke Test

```bash
# Option A: Use helper script
./scripts/web-smoke-test.sh

# Option B: Manual
docker compose -f docker-compose.web.yml up --build
# Then: Open http://localhost:8787, send /tmp/testfile.bin, verify hash
./scripts/update-web-validation.sh WEB-1 PASS "$(date +%Y-%m-%d)" "testfile.bin" "6e2e53bc5d2a187becf1d734d7cea4488042784f188cf4615054d2f2a39db7ec" "matches CLI minimal sanity"
```

---

## Step 2: Complete Release (Automated)

```bash
./scripts/complete-release.sh
```

**This script will:**
1. Run preflight checks (dry-run, then real)
2. Create and push tag `v1.0.1-web-preview`
3. Guide you through GitHub release creation
4. Switch to `feature/handshake-and-identity` branch
5. Merge main into feature branch

---

## Step 3: GitHub Release (Manual UI)

After tag is created, the script will pause and guide you:
1. Go to GitHub → Releases → Draft new release
2. Select tag: `v1.0.1-web-preview`
3. Title: `CrypRQ v1.0.1 — Web-Only Preview (Test Mode)`
4. Copy body from: `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
5. Publish

---

## Step 4: Next Phase

After release, you'll be on `feature/handshake-and-identity` branch.

Open: `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`

Use that to guide:
- Real handshake implementation
- Peer identity & auth
- Remove test-mode hacks
- Harden security docs

---

## Quick Command Reference

```bash
# Full automated flow (after web test)
./scripts/complete-release.sh

# Or step-by-step
./scripts/web-smoke-test.sh          # Step 1
./scripts/update-web-validation.sh ... # After web test
./scripts/preflight-and-tag.sh        # Step 2
# GitHub release (manual)             # Step 3
git checkout feature/handshake-and-identity && git merge main  # Step 4
```

---

**Status:** All scripts ready. Execute when ready.

