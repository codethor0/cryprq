# GitHub Actions Cleanup Instructions

## Goal
Remove all GitHub Actions workflows from the repo and clean out past workflow runs and artifacts in a rate-limit-safe way.

---

## STEP 1: Remove Workflow Files from Repo

**Status:** [OK] Already completed - `.github/workflows/` directory is empty or removed.

**If you need to do this manually:**

```bash
# Remove workflow files
rm -rf .github/workflows

# Stage and commit
git add .github/workflows
git commit -m "chore: remove GitHub Actions workflows"

# Push to GitHub
git push

# Verify clean status
git status
```

**Expected output:** `git status` should show no changes (or only unrelated changes).

---

## STEP 2: Clean Up Workflow Runs

**Script:** `./cleanup_runs_batch.sh`

**How it works:**
- Checks rate limit before each batch (stops if < 10 remaining)
- Fetches up to 50 run IDs per batch
- Deletes each run with 1-second delay
- Continues until all runs are deleted

**Usage:**

```bash
./cleanup_runs_batch.sh
```

**How often to run:**
- Run it **once** - it will loop until everything is deleted
- If rate limit is hit, it will exit gracefully - **run it again later** (wait ~1 hour)
- Keep re-running until you see: `[OK] ALL WORKFLOW RUNS DELETED`

**Success indicator:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[OK] ALL WORKFLOW RUNS DELETED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   Total deleted: X runs
```

**If rate limit is hit:**
- Script exits with message: `[WARN] Rate limit exhausted. Stopping.`
- Wait ~1 hour, then run the script again
- It will continue from where it left off

---

## STEP 3: Clean Up Artifacts

**Script:** `./cleanup_artifacts_batch.sh`

**How it works:**
- Same as runs script, but for artifacts
- Checks rate limit before each batch
- Fetches up to 50 artifact IDs per batch
- Deletes each artifact with 1-second delay

**Usage:**

```bash
./cleanup_artifacts_batch.sh
```

**How often to run:**
- Run it **once** - it will loop until everything is deleted
- If rate limit is hit, **run it again later** (wait ~1 hour)
- Keep re-running until you see: `[OK] ALL ARTIFACTS DELETED`

**Success indicator:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[OK] ALL ARTIFACTS DELETED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   Total deleted: X artifacts
```

---

## Quick Reference

**Check rate limit:**
```bash
gh api rate_limit --jq '.rate'
```

**Verify no runs left:**
```bash
gh api repos/codethor0/cryprq/actions/runs --jq '.total_count'
```

**Verify no artifacts left:**
```bash
gh api repos/codethor0/cryprq/actions/artifacts --jq '.total_count'
```

---

## Summary

1. [OK] Workflow files removed (already done)
2. [RUN] Run `./cleanup_runs_batch.sh` until all runs are deleted
3. [RUN] Run `./cleanup_artifacts_batch.sh` until all artifacts are deleted
4. Re-run scripts if rate limit is hit (wait ~1 hour between attempts)

