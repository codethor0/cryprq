# CrypRQ CI/E2E Investigation Runbook

## Overview

This runbook helps maintainers investigate QUIC E2E and CI failures on a given commit. It provides a systematic approach to distinguish between environment limitations and actual product bugs.

**Related Documentation:**
- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: User-facing troubleshooting guide and support bundle collection
- `scripts/support_bundle.sh`: Automated support bundle collection script
- `scripts/ci_e2e_probe.sh`: Automated probe script that runs core parts of this runbook

## Step 0 – Confirm HEAD & Diff Scope

**Goal:** Confirm you're on the correct branch and understand what changed since the last known-good commit.

**Commands:**
```bash
git status -sb
git branch -a
git log -1 --oneline
git rev-parse HEAD
```

**Optional (if comparing to a known-good commit):**
```bash
git diff <good_commit>...HEAD
```

**What to look for:**
- Working tree is clean (no uncommitted changes)
- You're on `main` branch (or the branch being investigated)
- HEAD commit matches the commit that failed in CI
- If comparing diffs, note which files changed (especially QUIC/crypto core vs UI/docs)

## Step 1 – Run Static/Local Gates

**Goal:** Verify formatting, governance, and build health before running E2E tests.

### 1.1 Governance Shield

```bash
python3 scripts/governance_shield.py
```

**Classification:**
- **PASS:** All checks pass (formatting, no emojis, no prompt files, Node.js build)
- **FAIL (Product Bug):** Formatting violations, emojis in tracked files, prompt artifacts committed
- **FAIL (ENV LIMITATION):** Rust clippy/build/tests fail with SSL/TLS errors (e.g., "Could not resolve host: index.crates.io") or npm EPERM permission errors

**Note:** Governance shield may fail on Rust/TypeScript checks due to environment limitations. Verify that formatting, emoji, and prompt-file checks still pass.

### 1.2 Rust Formatting

```bash
cargo fmt --all --check
```

**Classification:**
- **PASS:** Exit code 0, no formatting violations
- **FAIL (Product Bug):** Exit code non-zero, formatting violations detected

### 1.3 Rust Clippy (Optional)

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Classification:**
- **PASS:** Exit code 0, no clippy warnings
- **FAIL (ENV LIMITATION):** SSL/TLS errors accessing crates.io index
- **FAIL (Product Bug):** Actual clippy warnings or code issues

### 1.4 Rust Tests (Optional)

```bash
cargo test --workspace --lib
```

**Classification:**
- **PASS:** All tests pass
- **FAIL (ENV LIMITATION):** SSL/TLS errors accessing crates.io index
- **FAIL (Product Bug):** Test failures unrelated to network/SSL

### 1.5 Web Build

```bash
cd web
npm run build
cd ..
```

**Classification:**
- **PASS:** Build succeeds, bundle generated
- **FAIL (ENV LIMITATION):** npm EPERM permission errors
- **FAIL (Product Bug):** TypeScript compilation errors, Vite build failures

## Step 2 – Run E2E Suite Locally (If Docker Available)

**Goal:** Reproduce E2E test failures locally to distinguish environment issues from product bugs.

### 2.1 Check Docker Availability

```bash
command -v docker && docker info
```

**If Docker is unavailable or `docker info` fails:**
- Classify as **ENV LIMITATION**
- Note: "Docker not available / permission denied"
- Skip E2E local run, proceed to Step 3

### 2.2 Run E2E Suite

**Canonical Command (from repo root):**
```bash
docker compose -f docker/e2e/docker-compose.e2e.yml up --build --abort-on-container-exit
```

**After completion:**
```bash
docker compose -f docker/e2e/docker-compose.e2e.yml down --remove-orphans
```

**What PASS looks like:**
- Exit code: 0
- Test runner output shows: "Passed: 5, Failed: 0, Skipped: 0"
- Logs show key events:
  - `event=listener_ready ... status=accepting_connections`
  - `event=handshake_complete ... encryption=libp2p-transport cryprq_handshake=none status=ready`
  - `event=connection_established ... encryption_active=true`
  - `event=key_rotation status=success ... interval_secs=300`

**Classification:**
- **PASS:** Exit code 0, all 5 tests pass
- **FAIL (ENV LIMITATION):** Docker daemon permission errors, container startup failures due to system limits
- **FAIL (Product Bug):** Exit code non-zero, test failures with application-level errors (handshake failures, encryption errors, etc.)

**Note:** If `docker compose` fails with permission errors or daemon connection issues, classify as ENV LIMITATION and record that E2E could not be run locally.

## Step 3 – Map Local Behavior to CI

**Goal:** Compare local E2E command with CI workflow to identify discrepancies.

### 3.1 Inspect CI Workflow

Open `.github/workflows/e2e-tests.yml` and locate the E2E test job.

**Key information to extract:**
- Which command is used to run E2E (e.g., `docker compose -f docker-compose.e2e.yml up ...`)
- Working directory (e.g., `cd docker/e2e`)
- Environment variables or Docker setup steps

### 3.2 Compare with Local Command

**CI Command (from `.github/workflows/e2e-tests.yml`):**
```bash
cd docker/e2e
docker compose -f docker-compose.e2e.yml up --build --abort-on-container-exit
```

**Canonical Local Command (from repo root):**
```bash
docker compose -f docker/e2e/docker-compose.e2e.yml up --build --abort-on-container-exit
```

**These are equivalent** (CI runs from `docker/e2e`, local runs from repo root with full path).

**If commands differ:**
- Note the discrepancy (compose file path, service names, env vars)
- This may indicate a likely cause of CI failure
- Update scripts/docs to use the canonical command

**If commands match:**
- CI failure is likely due to:
  - Environment (Docker runner limits, resource constraints)
  - Real regression (if local E2E also fails)

## Step 4 – Attach Artifacts

**Goal:** Collect diagnostic information for bug reports or CI failure analysis.

### 4.1 Support Bundle

```bash
./scripts/support_bundle.sh
```

This collects:
- Git metadata (status, branches, recent commits)
- Health checks (governance shield, formatting)
- Docker state (containers, logs)
- Web build artifacts summary
- Relevant documentation

**Output:** `support_bundles/cryprq_support_YYYYMMDD_HHMMSS.tar.gz`

### 4.2 Web Tester Artifacts

If testing via Web Tester:
- Export logs from Debug Console ("Download Logs" button)
- Generate Bug Report Draft from the UI (pre-filled with session state)
- Take screenshots of Connection Health, Advanced Metrics, and Test Summary panels

### 4.3 E2E Logs

If E2E tests failed:
```bash
docker compose -f docker/e2e/docker-compose.e2e.yml logs listener
docker compose -f docker/e2e/docker-compose.e2e.yml logs dialer
docker compose -f docker/e2e/docker-compose.e2e.yml logs test-runner
```

## ENV LIMITATION vs Product Bug

### Examples of ENV LIMITATION

These are environment-related failures that should not be treated as product bugs:

- **Docker daemon permission errors:**
  - "permission denied while trying to connect to the Docker daemon socket"
  - "Cannot connect to the Docker daemon"
- **SSL/TLS certificate issues:**
  - "Could not resolve host: index.crates.io"
  - "SSL certificate problem"
  - "Failed to download from https://index.crates.io"
- **npm permission errors:**
  - "EPERM: operation not permitted"
  - Filesystem permission errors in node_modules
- **Container resource limits:**
  - "Cannot allocate memory"
  - "No space left on device"
  - Timeout errors due to slow CI runners

### Examples of Potential Product Bugs

These indicate actual issues in the codebase:

- **QUIC handshake failures:**
  - Application-level errors in handshake logic
  - ML-KEM/X25519 encryption errors
  - Connection establishment failures not due to network/Docker
- **E2E test failures:**
  - Test assertions failing
  - Expected events not occurring
  - File transfer integrity failures
- **Web build errors:**
  - TypeScript compilation errors
  - Vite build failures
  - Actual code issues (not npm permission problems)
- **Rust compilation errors:**
  - Actual code errors (not SSL/TLS network issues)
  - Type mismatches, borrow checker errors

## Quick Reference: Canonical E2E Command

**From repo root:**
```bash
docker compose -f docker/e2e/docker-compose.e2e.yml up --build --abort-on-container-exit
docker compose -f docker/e2e/docker-compose.e2e.yml down --remove-orphans
```

**Note:** Docker daemon availability and permissions can cause failures that are environment-related, not product bugs. If Docker is unavailable, classify as ENV LIMITATION.

## Additional Resources

- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: User-facing troubleshooting guide
- `scripts/support_bundle.sh`: Automated support bundle collection
- `scripts/ci_e2e_probe.sh`: Automated probe script for CI/E2E investigation
- `docs/VPN_TEST_PLAN.md`: VPN and P2P tunnel test procedures
- `docs/FILE_TRANSFER_TEST_PLAN.md`: File transfer testing procedures

