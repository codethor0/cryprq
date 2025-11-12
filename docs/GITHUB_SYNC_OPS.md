# "All Wired with GitHub" – Standard Ops Prompt

**Objective:** Ensure local ↔ GitHub are fully synchronized, builds are healthy, CI is green, and (optionally) ship the release. Produce a concise, auditable summary including PR URL, workflow runs, and artifact paths.

## Preconditions

- Docker daemon running
- Node.js 18+
- GitHub CLI (`gh`) installed and authenticated (script will prompt if not)
- Repo root contains `scripts/`, `Makefile`, and `.github/workflows/*`

## Run Modes

### Default (sync + CI only)

```bash
./scripts/github-sync.sh
```

### Ship after CI

```bash
SHIP=true ./scripts/github-sync.sh
```

### Ship + post monitoring

```bash
SHIP=true RUN_POST=true ./scripts/github-sync.sh
```

### Makefile alias

```bash
make github-sync
```

## What the Script Does (10 Steps)

### 0) Env & Repo Hygiene

- Print versions (git, node, npm, docker)
- Verify git remotes
- Fetch all remotes (`git fetch --all --prune`)
- Create feature branch if on main/master
- Ensure `.gitignore` includes `artifacts/`

### 1) GitHub Checks

- Verify GitHub CLI auth (prompt login if needed)
- Verify required workflows exist:
  - `.github/workflows/release.yml`
  - `.github/workflows/release-verify.yml`
  - `.github/workflows/mobile-ci.yml`
  - `.github/workflows/local-validate-mirror.yml`

### 2) Local Validation

- Run `quick-smoke.sh` first
- Escalate to `local-validate.sh` on failure
- Fail loudly if validation fails

### 3) Sync & Push

- Stage all changes (exclude artifacts)
- Commit with standard message
- Push branch to origin

### 4) PR Handling

- Open PR if missing
- Attach `OPERATOR_CHEAT_SHEET.txt` as comment
- Capture PR URL

### 5) CI Trigger

- Trigger `local-validate-mirror.yml` workflow
- Trigger `mobile-ci.yml` workflow
- Record workflow run status

### 6) CI Await

- Poll PR checks every 60s (max 20 minutes)
- Fail loudly if any checks are red
- Proceed if all checks pass

### 7) Optional Ship

- Only if `SHIP=true`
- Run `go-live.sh 1.1.0`
- Run `verify-release.sh`
- Verify GitHub Release assets exist
- List asset names and sizes
- Check signing/notarization status

### 8) Optional Post

- Only if `RUN_POST=true`
- Run `observability-checks.sh`
- Run `sanity-checks.sh`

### 9) Cleanup

- Run `cleanup.sh`
- Stop Docker containers
- Kill Electron processes
- Prune temp artifacts

### 10) Summary

- Print branch name
- Print PR URL
- List artifact paths
- List report paths
- Show CI status
- Include next steps (golden path)

## Acceptance Criteria

- ✅ Local quick-smoke or full validation passes
- ✅ Branch pushed; PR open with cheat sheet comment
- ✅ CI workflows triggered and checks pass (or are reported)
- ✅ If `SHIP=true`: GitHub Release v1.1.0 exists with signed/notarized artifacts and verify-release success
- ✅ Summary printed with artifact paths and links

## Operator Notes

### Golden Path (Manual)

**Desktop:** Connect → Charts ≤3–5s → Rotate (toast ≤2s) → Disconnect

**Mobile:** Settings → Report Issue → share sheet (<2MB, "Report Prepared")

### Rollback

**Desktop:** Unmark "Latest" on GitHub Release; re-promote prior stable

**Mobile:** Pause Play rollout / expire TestFlight build

## Common Invocations

```bash
## Sync + CI only
./scripts/github-sync.sh

## Ship after PR review
SHIP=true ./scripts/github-sync.sh

## Ship + monitoring
SHIP=true RUN_POST=true ./scripts/github-sync.sh

## Makefile alias
make github-sync
```

## Tiny QoL (Optional)

### Shell Alias

Add to your shell profile (`~/.zshrc`, `~/.bashrc`, etc.):

```bash
alias cryprq-sync='SHIP=true RUN_POST=true ./scripts/github-sync.sh'
```

Then use:

```bash
cryprq-sync
```

### Quick Reference

**Most common:**
```bash
make github-sync              # Sync + CI
SHIP=true make github-sync    # Sync + CI + Ship
```

**Full workflow:**
```bash
SHIP=true RUN_POST=true ./scripts/github-sync.sh
```

## Example Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 GITHUB SYNC SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Branch: chore/github-sync-20250115-2030
PR: https://github.com/org/cryprq/pull/123
Desktop artifacts: artifacts/desktop/darwin/CrypRQ-1.1.0.dmg,artifacts/desktop/darwin/CrypRQ-1.1.0-mac.zip
Reports: artifacts/reports/playwright-report/index.html,artifacts/reports/test-results.json
CI mirror: triggered (see PR Checks tab)
CI status: ✅ All checks passed
Release: v1.1.0 pushed; verify assets on GitHub Releases page

Next: Golden path
  Desktop: Connect → Charts ≤3s → Rotate (toast ≤2s) → Disconnect
  Mobile: Settings → Report Issue → share sheet (<2MB, 'Report Prepared')

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

**You're fully wired with GitHub. Run `make github-sync` to sync, verify CI, and optionally ship.** 🚀

