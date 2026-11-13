# CrypRQ Release Checklist

> **Historical document.** Written for the v1.0.x web preview, which ran an unauthenticated test-mode path. Statements about readiness or encryption in it do not describe the current code; see [README](../README.md) and [Security Notes](SECURITY_NOTES.md).

## Overview

This document defines a repeatable process for cutting a CrypRQ release (Git tag + GitHub release), minimizing regressions and ensuring consistency.

**Related Documentation:**
- `docs/CI_E2E_INVESTIGATION_RUNBOOK.md`: How to investigate CI/E2E failures
- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: User-facing troubleshooting guide
- `docs/WEB_TESTER_GUIDE.md`: Web Tester usage guide
- `docs/QUICKSTART_CLI.md`: CLI quickstart guide

**Related Scripts:**
- `scripts/quick_smoke.sh`: Combined fmt + governance + E2E + web up smoke test
- `scripts/ci_e2e_probe.sh`: CI/E2E probe script with classification
- `scripts/support_bundle.sh`: Automated support bundle collection
- `scripts/release_probe.sh`: Release pre-flight probe (runs quick_smoke + ci_e2e_probe)

## Pre-Release Gates

### STEP 0 – Confirm Branch and HEAD

**Commands:**
```bash
git status -sb
git branch -a
git fetch origin
git rev-parse HEAD
git rev-parse origin/main
```

**Conditions:**
- Working tree must be clean (no uncommitted changes)
- You must be on `main` branch
- HEAD should be at the desired release commit (usually equal to `origin/main` or a known commit to tag)

**If conditions are not met:**
- Clean working tree: `git stash` or commit changes
- Not on main: `git checkout main`
- HEAD != origin/main: `git pull origin main` or verify the commit is intentional

### STEP 1 – Run Quick Smoke

**Command:**
```bash
./scripts/quick_smoke.sh
```

**Expectations:**
- Rust formatting check passes (`cargo fmt --all --check`)
- Governance shield passes (with ENV LIMITATION allowed for clippy/build/tests/TypeScript types if they match known SSL/TLS or npm EPERM patterns)
- Web build succeeds (`npm run build` in `web/` directory)
- E2E harness passes if Docker is available (5/5 tests passing, exit code 0)
  - If Docker daemon is not accessible, classify as ENV LIMITATION and note in release notes
- Web stack comes up and responds on http://localhost:8787 (HTTP 200 OK)

**If quick_smoke.sh fails:**
- Review output to distinguish ENV LIMITATION from product bugs
- For product bugs: fix issues before proceeding
- For ENV LIMITATION: document in release notes and proceed if core gates (fmt, web build) pass

### STEP 2 – Run CI/E2E Probe

**Command:**
```bash
./scripts/ci_e2e_probe.sh
```

**Interpretation:**
- The script prints a summary of static gates and E2E status
- Each check is classified as:
  - **PASS**: Check succeeded
  - **FAIL**: Check failed due to product issue
  - **ENV_LIMITATION**: Check failed due to environment (SSL/TLS, Docker permissions, npm EPERM)

**Requirements:**
- `governance_shield`: Should be PASS (formatting, no emojis, no prompt files, Node.js build)
- `cargo fmt`: Must be PASS
- `npm build`: Must be PASS
- `e2e run`: Should be PASS if Docker is available; ENV_LIMITATION is acceptable if Docker daemon is not accessible

**If ci_e2e_probe.sh shows FAIL for critical gates:**
- Do not proceed with release until issues are resolved
- Use `docs/CI_E2E_INVESTIGATION_RUNBOOK.md` to investigate failures

### STEP 3 – Manual Spot Checks (Web Tester)

**Goal:** Verify Web Tester functionality with real interaction.

**Optional: VPN Host-Mode Validation (for releases that change `--vpn` behavior):**
- Consider running `./scripts/vpn_host_probe.sh` on at least one macOS and one Linux host
- Confirm host-mode behavior and classification (PASS vs ENV LIMITATION vs FAIL)
- See `docs/VPN_HOST_TEST_PLAN.md` for detailed host-mode testing procedures
- Note: This requires root/admin privileges and TUN support; classify missing prerequisites as ENV LIMITATION
- **VPN Leak & Routing Checks (if VPN behavior changed):**
  - Run `./scripts/vpn_host_probe.sh` and `./scripts/vpn_leak_probe.sh` on at least one supported host
  - Document results and classification using `docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md`
  - Verify IP, DNS, IPv6, and routing behavior matches documented architecture
  - Include leak probe output in release validation documentation

**Guided Tests:**
1. **Local Loopback P2P Test:**
   - Use Guided Tests panel to pre-fill Listener mode
   - Click Connect and verify listener starts
   - Verify Connection Health panel shows "Encrypted tunnel active"
   - Verify Debug Console shows `listener_ready` and `handshake_complete` events

2. **Two-Tab Web Test:**
   - Tab 1: Listener mode, port 10000, Connect
   - Tab 2: Dialer mode, port 10000, peer `/ip4/127.0.0.1/udp/10000/quic-v1`, Connect
   - Verify both tabs show connection established
   - Verify Advanced Metrics panel updates with connection/ping/rotation stats

3. **File Transfer Test:**
   - With connection established, use "Send File Securely" to transfer a small test file
   - Verify file transfer state machine works (idle → preparing → in_progress → completed)
   - Verify Test Summary panel shows file transfer metrics

**What to verify:**
- Connection Health panel shows reasonable values (handshake direction, ping latency, epoch, rotation interval)
- Advanced Metrics panel updates with connection attempts, ping stats, key rotations, file transfers, errors
- File transfer state machine works for at least one small file
- No console errors in browser DevTools

### STEP 4 – Collect Diagnostics (Optional but Recommended)

**Support Bundle:**
```bash
./scripts/support_bundle.sh
```

**Output:** `support_bundles/cryprq_support_YYYYMMDD_HHMMSS.tar.gz`

**Web Tester Artifacts:**
- Export logs from Debug Console ("Download Logs" button)
- Export session timeline from Debug Console ("Download Timeline (JSON)" button)
- Generate Bug Report Draft from the UI (even for a "clean" release, as a snapshot)
- Take screenshots of Connection Health, Advanced Metrics, and Test Summary panels

**Validation Run Report (Optional):**
```bash
./scripts/validation_run_report.sh
```

**Output:** `docs/VALIDATION_RUN_YYYYMMDD_HHMMSSZ_<SHA>.md`

**Regression Guard Baseline Comparison (Optional but Recommended):**
- If using Web Tester:
  - Mark a known-good session (e.g., from previous stable release) as a baseline in Session History
  - Run a fresh validation test in the Web Tester
  - Use the Regression Guard panel to compare the current session against the baseline
  - Ensure status is **PASS**, or understand and document any **WARN** signals (e.g., performance noise or ENV LIMITATION)
  - If Regression Guard surfaces new **PRODUCT SUSPECT** diagnostics or clear **FAIL** signals, investigate before release (may be a release blocker)
  - Click "Copy Report (Markdown)" to include the regression guard report in release documentation

**Validation Packet (Optional but Recommended):**
- Generate a Validation Packet from the Web Tester for the final smoke run
- Ideally include:
  - A golden baseline comparison (Regression Guard)
  - Scenario Playbook
  - Diagnostics summary
- This makes regressions easier to debug later and provides a comprehensive snapshot of the release validation state

**Purpose:** These artifacts serve as a baseline for the release and can help diagnose issues reported after release. The validation run report provides a consolidated summary of all probe outputs. The regression guard report helps detect regressions compared to known-good baselines. The validation packet combines all session information into a single, shareable document.

## Tagging and GitHub Release

### STEP 5 – Choose Version/Tag

**Naming Guidance:**
- Semantic versioning: `v1.0.3` (major.minor.patch)
- Functional tags: `v1.0.3-web-ui-metrics` (descriptive suffix)
- Pre-release tags: `v1.0.3-alpha.1`, `v1.0.3-beta.1`

**Tag Format:**
- Use annotated tags: `git tag -a`
- Include short description in tag message

**Example:**
```bash
export TAG=v1.0.3-web-ui-metrics
git tag -a "$TAG" -m "CrypRQ $TAG release - Web UI metrics and bug report draft generator"
```

### STEP 6 – Create and Push Tag

**Commands:**
```bash
export TAG=vX.Y.Z-suffix
git tag -a "$TAG" -m "CrypRQ $TAG release"
git push origin main
git push origin "$TAG"
```

**Verification:**
- Tag appears in `git tag -l`
- Tag is pushed to `origin` (visible on GitHub)

### STEP 7 – Create GitHub Release

**GitHub Release Requirements:**
- Reference the tag created in STEP 6
- Summarize major changes since last release
- Link to relevant documentation:
  - `docs/WEB_TESTER_GUIDE.md`
  - `docs/QUICKSTART_CLI.md`
  - `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`
- Include any known limitations or ENV LIMITATION notes
- Attach support bundle if relevant

**Release Title Example:**
```
CrypRQ v1.0.3-web-ui-metrics – Web UI metrics and bug report draft generator
```

**Release Body Structure:**
- Overview: Brief summary of release focus
- What Changed: List of major features/fixes
- Validation: Reference to validation run or probe results
- How to Use: Links to quickstart guides
- Known Limitations: Any ENV LIMITATION or known issues

## Post-Release Notes

### Verify CI/CD

After pushing the tag:
- Check GitHub Actions workflows for the tag's commit
- Verify CI and E2E workflows are green (or note any ENV LIMITATION failures)
- If CI fails, use `docs/CI_E2E_INVESTIGATION_RUNBOOK.md` to investigate

### Update External Documentation

If applicable:
- Update any external websites or documentation
- Update package registries if CrypRQ is published
- Announce release in relevant channels

## Quick Reference: Release Probe

Before starting the release process, run:

```bash
./scripts/release_probe.sh
```

This script:
- Checks git state (branch, working tree, HEAD)
- Runs `scripts/quick_smoke.sh`
- Runs `scripts/ci_e2e_probe.sh`
- Prints a summary indicating readiness for release

The probe is informational; it does not block release but provides a clear "ready/not ready" assessment.

## Additional Resources

- `docs/VALIDATION_RUN_RELEASE_12f7bd6.md`: Example validation run for v1.0.2-web-ui-fix
- `docs/CI_E2E_INVESTIGATION_RUNBOOK.md`: How to investigate CI/E2E failures
- `scripts/release_probe.sh`: Automated release pre-flight probe

