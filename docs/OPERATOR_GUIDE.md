# Operator Guide — Cutting v1.0.1-web-preview Release

**Purpose:** Short, operator-level game plan for cutting the web-only preview release and starting the next phase.

**Status (November 10, 2026): release blocked.** The dependency gate retains
five findings documented in `SECURITY_NOTES.md`, and the final authenticated
protocol/receiver/VPN combination has not passed release verification. The
historical procedure below is conditional on resolving those gates.

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

### Step 5: Implement Real Handshake

Follow the protocol specification for:

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

## Hygiene Drift Audit System

> **Removed in November 2026.** The scheduled `hygiene-drift-audit.yml` workflow and its automatic issue creation no longer exist, and neither does the duplicate `ci-stabilizer.yml`. The same governance shield runs on every pull request and nightly as the `Governance Checks` job of `ci.yml`, and format, clippy, build and tests run in that workflow's other jobs. The text below is kept as a description of the retired setup.

**Purpose:** Automated daily checks to prevent repository hygiene violations (prompt files, agent reports, emojis in code/docs).

**Schedule:** Runs daily at 3 AM UTC via `.github/workflows/hygiene-drift-audit.yml`

**Configuration:**
- **Secret:** `ENABLE_HYGIENE_DRIFT_ALERTS` (set to `true` to enable issue creation on violations)
- **Current status:** Enabled (`ENABLE_HYGIENE_DRIFT_ALERTS=true`)

### How It Works

1. **Daily Audit:** The workflow runs `scripts/governance_shield.py` to scan for:
   - Prompt/agent/validation files (e.g., `*PROMPT*.md`, `*AGENT*.md`, `*REPORT*.md`)
   - Emojis in code and documentation files
   - Other hygiene violations

2. **Issue Creation:** If violations are detected AND `ENABLE_HYGIENE_DRIFT_ALERTS=true`:
   - Creates a GitHub issue with label `drift-audit`
   - Includes full governance shield output
   - Deduplicates: only one open issue per day (adds comments to existing issue if triggered multiple times)

3. **CI Gate:** The same checks run in CI on every PR via the "Governance Checks" job, blocking merges until violations are resolved.

### Disabling Alerts

To temporarily disable issue creation (e.g., during maintenance):

```bash
gh secret set ENABLE_HYGIENE_DRIFT_ALERTS --repo codethor0/cryprq --body false
```

To re-enable:

```bash
gh secret set ENABLE_HYGIENE_DRIFT_ALERTS --repo codethor0/cryprq --body true
```

**Note:** Disabling alerts does not disable the CI gate. The governance checks will still run and block merges.

### Running an Alert Drill

To verify the system end-to-end:

```bash
# 1. Create a test branch with violations
git checkout -b test/hygiene-drift-alert-drill
printf "TEST\n" > TEST_AGENT_REPORT.md
printf "emoji-test: [EMOJI_TEST]\n" > TEST_EMOJI_VIOLATION.txt
git add TEST_AGENT_REPORT.md TEST_EMOJI_VIOLATION.txt
git commit -S -m "test: hygiene drift alert drill (do not merge)"
git push -u origin test/hygiene-drift-alert-drill

# 2. Trigger the workflow
gh workflow run hygiene-drift-audit.yml --repo codethor0/cryprq --ref test/hygiene-drift-alert-drill

# 3. Wait for completion and verify issue creation
gh run watch <RUN_ID> --repo codethor0/cryprq
gh issue list --repo codethor0/cryprq --label drift-audit --state open

# 4. Cleanup
gh issue close <ISSUE_NUMBER> --repo codethor0/cryprq --comment "Drill complete; no action required."
git checkout main
git push origin --delete test/hygiene-drift-alert-drill
```

**Important:** Do not delete the test branch until the workflow run is created and completes, otherwise the workflow dispatch will fail.

### Changing the Schedule

To change the daily run time, edit `.github/workflows/hygiene-drift-audit.yml`:

```yaml
schedule:
  - cron: '0 3 * * *'  # Change to desired UTC time (HH MM format)
```

Common times:
- `'0 3 * * *'` = 3 AM UTC
- `'0 9 * * *'` = 9 AM UTC
- `'0 0 * * *'` = Midnight UTC

---

## MSRV/Dependency Pin Policy

**Purpose:** Prevent CI flapping and maintain deterministic builds by pinning dependencies that require newer toolchain features.

**Current MSRV:** Rust 1.88.0, pinned in `rust-toolchain.toml` and the CI/Docker builders. The workspace remains on edition 2021. The approved Rust 1.88 migration retains the `base64ct = "=1.6.0"` pin.

**Policy:**
- Any crate requiring `edition2024` (or other toolchain bumps) must be introduced only via an explicit MSRV/toolchain upgrade PR with CI proof.
- Dependency pins (e.g., `base64ct = "=1.6.0"` in `node/Cargo.toml`) must remain in place until that specific pin change is approved and validated. A toolchain upgrade does not automatically remove a pin.
- `Cargo.lock` must be committed and kept in sync with explicit version pins in `Cargo.toml` files.

**Current Pins:**
- `base64ct = "=1.6.0"` in `node/Cargo.toml` (prevents `base64ct 1.8.x` which requires `edition2024`)

**Upgrading a Pinned Dependency:**
1. Create a dedicated PR for the toolchain/MSRV upgrade.
2. Update `rust-toolchain.toml` to the required version.
3. Verify CI passes with the new toolchain.
4. Change or remove an explicit version pin only if that specific change is approved; otherwise retain it.
5. Update `Cargo.lock` and commit.
6. Merge only after all CI checks pass.

**Rationale:** Prevents CI failures from dependency resolution pulling incompatible versions (e.g., `base64ct 1.8.1` requiring `edition2024` when using Rust 1.83.0).

---

## Quick Reference

**Release Runbook:** `docs/CUT_THE_RELEASE.md` 
**Pre-Tag Checklist:** `docs/PRE_TAG_CHECKLIST.md` 
**Release Body:** `docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md`

---

## GREEN BASELINE CHECKLIST

**Purpose:** Quick audit checklist of all enforced invariants. Use this to verify the repository baseline state in one minute.

### Branch Management
- [ ] **Default branch:** `main` (verify: `gh repo view codethor0/cryprq --json defaultBranchRef --jq '.defaultBranchRef.name'`)
- [ ] **Auto-delete merged branches:** Enabled (verify: `gh repo view codethor0/cryprq --json deleteBranchOnMerge --jq '.deleteBranchOnMerge'` should be `true`)
- [ ] **Remote branches:** Only `main` + open PR branches (verify: `git for-each-ref --format='%(refname:short)' refs/remotes/origin | sed 's#^origin/##' | grep -v '^origin$' | sort -u`)

### Branch Protection (main)
- [ ] **Required status checks:** All must pass (verify: `gh api repos/codethor0/cryprq/branches/main/protection --jq '.required_status_checks.contexts'`)
  - Quick Check (fmt + clippy)
  - Build and Test
  - Security Checks
  - Governance Checks
  - CodeQL
- [ ] **Strict status checks:** Enabled (verify: `gh api repos/codethor0/cryprq/branches/main/protection --jq '.required_status_checks.strict'` should be `true`)
- [ ] **Required reviews:** 1 (verify: `gh api repos/codethor0/cryprq/branches/main/protection --jq '.required_pull_request_reviews.required_approving_review_count'`)
- [ ] **Signed commits required:** Enabled (verify: `gh api repos/codethor0/cryprq/branches/main/protection --jq '.required_signatures.enabled'` should be `true`)
- [ ] **Conversation resolution:** Required (verify: `gh api repos/codethor0/cryprq/branches/main/protection --jq '.required_conversation_resolution.enabled'` should be `true`)

### Hygiene Gate (CI)
- [ ] **Governance Checks job:** Runs on every PR (verify: `.github/workflows/ci.yml` contains "Governance Checks" job)
- [ ] **Governance shield script:** `scripts/governance_shield.py` exists and checks:
  - No prompt/agent/validation files
  - No emojis in code/docs
  - Rust formatting, clippy, build, tests
  - Node.js build, TypeScript types

### Hygiene Drift Audit
- [ ] **Workflow exists:** `.github/workflows/hygiene-drift-audit.yml` present
- [ ] **Schedule:** Daily at 3 AM UTC (verify: `grep -A 1 "schedule:" .github/workflows/hygiene-drift-audit.yml`)
- [ ] **Secret flag:** `ENABLE_HYGIENE_DRIFT_ALERTS` set to `true` (verify: `gh secret list --repo codethor0/cryprq | grep ENABLE_HYGIENE_DRIFT_ALERTS`)
- [ ] **Issue creation:** Gated by secret (verify: workflow has `Check if alerts are enabled` step)

### Dependency Pins
- [ ] **base64ct:** Pinned at `=1.6.0` in `node/Cargo.toml` (verify: `grep "base64ct" node/Cargo.toml`)
- [ ] **Cargo.lock:** Committed and locked to `base64ct 1.6.0` (verify: `grep -A 2 'name = "base64ct"' Cargo.lock | grep version`)

### Tag Naming Convention
- [ ] **Format:** `v<major>.<minor>.<patch>-green-baseline[.<revision>]` (e.g., `v1.0.2-green-baseline.1`)
- [ ] **Tags are signed:** All baseline tags must be signed (verify: `git tag -v <tag-name>`)

### Documentation
- [ ] **OPERATOR_GUIDE.md:** Contains sections for:
  - Hygiene Drift Audit System
  - MSRV/Dependency Pin Policy
  - GREEN BASELINE CHECKLIST (this section)

### Quick Verification Commands
```bash
# Branch state
git for-each-ref --format='%(refname:short)' refs/remotes/origin | sed 's#^origin/##' | grep -v '^origin$' | sort -u

# Branch protection
gh api repos/codethor0/cryprq/branches/main/protection --jq '{strict: .required_status_checks.strict, checks: .required_status_checks.contexts, signed: .required_signatures.enabled}'

# Hygiene drift audit secret
gh secret list --repo codethor0/cryprq | grep ENABLE_HYGIENE_DRIFT_ALERTS

# Dependency pins
grep "base64ct" node/Cargo.toml
grep -A 2 'name = "base64ct"' Cargo.lock | grep version

# Latest CI status
gh run list --repo codethor0/cryprq --limit 1 --json conclusion,url --jq '.[0]'
```

---

**Last Updated:** 2025-12-29 (Added hygiene drift audit documentation, MSRV/dependency pin policy, and GREEN BASELINE CHECKLIST)

