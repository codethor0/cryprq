# Release Execution Summary — v1.0.1-web-preview

**Purpose:** Step-by-step execution guide for cutting the web-only preview release.

**Status:** All infrastructure ready. Follow these steps to execute.

---

## Step 1: Web Smoke Test (Strongly Recommended)

**Before tagging, run WEB-1/WEB-2:**

### 1.1 Start Web Stack

```bash
# From repo root
docker compose -f docker-compose.web.yml up --build
```

### 1.2 Access Web UI

- **URL:** `http://localhost:8787` (backend API) or `http://localhost:5173` (Vite dev server)
- **Ports:** See `docker-compose.web.yml`:
  - Backend API: `8787`
  - Vite dev: `5173`
  - UDP: `9999`

### 1.3 Perform File Transfer

1. Open Web UI in browser
2. Send a small test file via UI
3. Confirm file appears in configured output directory
4. Verify SHA-256 matches:

```bash
sha256sum /path/to/source-file /path/to/received-file
# Hashes must match
```

### 1.4 Update Validation Doc

Edit `docs/WEB_VALIDATION_RUN.md`:

- **WEB-1:** Mark as ✅ PASS (or ⚠️ WARN with notes)
- **WEB-2:** Mark as ✅ PASS (or ⚠️ WARN with notes)
- Add date, executor name, and one-line note

**Example:**
```markdown
| WEB-1  | Minimal Web Loopback File Transfer   | ✅ PASS | 2025-11-14: Basic smoke test passed, hash verified |
| WEB-2  | Medium File Web Transfer             | ✅ PASS | 2025-11-14: 10MB file transferred successfully |
```

---

## Step 2: Run Preflight Script

### 2.1 Dry Run (Preview)

```bash
./scripts/preflight-and-tag.sh --dry-run
```

**Expected output:**
- ✅ CLI validation: PASS
- ⚠️ Web validation: TODO (if not done yet) or PASS
- ✅ Security disclaimers: All present
- ✅ Git status: Clean
- Preview of tag creation

### 2.2 Execute (Creates Tag)

```bash
./scripts/preflight-and-tag.sh
```

**What it does:**
1. Confirms CLI validation ✅
2. Warns if WEB-1/WEB-2 still TODO (allows override)
3. Verifies all test-mode / NOT FOR PRODUCTION disclaimers
4. Ensures git is clean
5. Creates and pushes tag `v1.0.1-web-preview`
6. Prints GitHub release instructions

**Result:** Tag is live on remote.

---

## Step 3: Create GitHub Release

### 3.1 Navigate to GitHub

1. Go to **GitHub → Releases → Draft new release**

### 3.2 Configure Release

- **Tag:** `v1.0.1-web-preview` (select from dropdown)
- **Title:** `CrypRQ v1.0.1 — Web-Only Preview (Test Mode, Non-Production)`

### 3.3 Release Body

1. Open `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`
2. Copy **entire contents**
3. Paste into GitHub release description field

### 3.4 Double-Check Security Warnings

Verify the top of the body clearly states:
- ✅ static keys
- ✅ no handshake
- ✅ no peer auth
- ✅ **NOT FOR PRODUCTION**

### 3.5 Publish

Click **"Publish release"**

**Result:** Web-only preview release is officially published.

---

## Step 4: Start Next Phase (Handshake + Identity)

### 4.1 Create Feature Branch

```bash
# From repo root
git checkout -b feature/handshake-and-identity
```

### 4.2 Drive Work with Master Prompt

Use `docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md` to guide:

#### 4.2.1 Real Handshake
- Implement `CRYPRQ_CLIENT_HELLO` / `SERVER_HELLO` / `CLIENT_FINISH` on the wire
- Plug in v1.0.1 key schedule (HKDF, epochs, proper `keys_ir`/`keys_ri`)

#### 4.2.2 Peer Identity & Authentication
- Decide on identity scheme (Ed25519 / libp2p peer IDs / PSK)
- Ensure handshake authenticates peers, not just encrypts

#### 4.2.3 Fix Direction Correctness
- Remove "both sides are initiator, decrypt with `keys_outbound`" hack
- Implement proper initiator/responder role semantics per spec

#### 4.2.4 Remove Test-Mode Hacks
- Remove static keys
- Remove "bypass handshake" logic
- Tighten `SECURITY_NOTES.md` to differentiate:
  - `v1.0.1-web-preview` (test-mode)
  - Future secure builds (with real handshake + auth)

---

## Quick Reference: Key Commands

```bash
# Web smoke test
docker compose -f docker-compose.web.yml up --build
# Then: Open http://localhost:8787, send file, verify hash

# Preflight (dry run)
./scripts/preflight-and-tag.sh --dry-run

# Preflight (execute)
./scripts/preflight-and-tag.sh

# Create next-phase branch
git checkout -b feature/handshake-and-identity
```

---

## Files to Update During Release

- `docs/WEB_VALIDATION_RUN.md` — Mark WEB-1/WEB-2 as PASS after smoke test
- `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md` — Copy to GitHub release

---

## Success Criteria

✅ Web smoke test completed (WEB-1/WEB-2 marked PASS)  
✅ Preflight script passes all checks  
✅ Tag `v1.0.1-web-preview` created and pushed  
✅ GitHub release published with correct body  
✅ Feature branch `feature/handshake-and-identity` created  

**At this point:** Preview release is live, and next-phase work can begin.

---

**Last Updated:** (Update when executing release)

