# One-Shot Workflow

**End-to-end orchestration**: quick-smoke → validate → push/PR → (optional) ship → cleanup

## Usage

### Basic (validation + PR)

```bash
./scripts/one-shot.sh
```

### With Ship (after PR review)

```bash
SHIP=true ./scripts/one-shot.sh --ship
# Or
./scripts/one-shot.sh --ship
```

### With Post-Actions (observability + sanity)

```bash
RUN_POST=true ./scripts/one-shot.sh --post
# Or
./scripts/one-shot.sh --post
```

### Full Workflow

```bash
SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post
```

## Steps

1. **Preconditions Check**
   - Docker running
   - Node.js 18+
   - Git available

2. **Quick-Smoke** (fast confidence)
   - Runs `./scripts/quick-smoke.sh`
   - Fast local sanity check

3. **Full Validation** (if quick-smoke failed)
   - Runs `./scripts/local-validate.sh`
   - Comprehensive validation

4. **Summarize Outputs**
   - Lists desktop artifacts
   - Lists reports

5. **Branch + PR** (idempotent)
   - Creates feature branch if on main/master
   - Commits changes
   - Pushes branch
   - Creates PR (if gh CLI available)

6. **CI Mirror** (optional)
   - Triggers `local-validate-mirror.yml` workflow
   - Aligns PR with local validation

7. **Ship** (optional, guarded)
   - Runs `./scripts/go-live.sh VERSION`
   - Runs `./scripts/verify-release.sh`
   - Only if `SHIP=true` or `--ship` flag

8. **Post-Actions** (optional)
   - Runs `./scripts/observability-checks.sh`
   - Runs `./scripts/sanity-checks.sh`
   - Only if `RUN_POST=true` or `--post` flag

9. **Cleanup**
   - Runs `./scripts/cleanup.sh`
   - Stops Docker containers
   - Cleans artifacts

## Output

The script prints a compact summary with:
- Current branch
- PR URL (if created)
- Artifact paths
- Reports paths
- Next suggested steps

## Acceptance Criteria

- ✅ Docker fake backend responded on :9464 during tests
- ✅ Lint, typecheck, unit, and Playwright E2E passed locally or in full validation
- ✅ Desktop artifacts exist under `artifacts/desktop/<platform>/`
- ✅ PR opened against main (if GH CLI available) and mirror CI kicked off
- ✅ If shipped: release artifacts are signed/notarized and verify-release reported OK

## Golden Path (for PR testing)

**Connect → charts ≤3–5s → Rotate (toast ≤2s) → Disconnect**

## Monitoring (first 24h)

**Every 2h:**
```bash
./scripts/observability-checks.sh
./scripts/sanity-checks.sh
```

## Rollback

- **Desktop**: Unmark "Latest" release on GitHub
- **Mobile**: Pause rollout in Play Console / App Store Connect

## Example Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 ONE-SHOT SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Branch: chore/local-validate-20250115-1930
PR: https://github.com/org/cryprq/pull/123

Artifacts:
  • artifacts/desktop/darwin/CrypRQ-1.1.0.dmg
  • artifacts/desktop/darwin/CrypRQ-1.1.0-mac.zip

Reports:
  • artifacts/reports/playwright-report/index.html
  • artifacts/reports/test-results.json

Next Steps:
  1. Review PR: https://github.com/org/cryprq/pull/123
  2. Wait for CI to pass
  3. Merge PR when ready
  4. To ship: SHIP=true ./scripts/one-shot.sh --ship

Golden Path (for PR testing):
  Connect → charts ≤3–5s → Rotate (toast ≤2s) → Disconnect

Every 2h (first 24h):
  ./scripts/observability-checks.sh
  ./scripts/sanity-checks.sh

Rollback:
  Unmark desktop 'Latest' release / pause mobile rollout
```

## Tips

**For PR comments:**
```
Golden path: Connect → charts ≤3–5s → Rotate (toast ≤2s) → Disconnect

Every 2h (first 24h):
  ./scripts/observability-checks.sh
  ./scripts/sanity-checks.sh

Rollback: unmark desktop "Latest" release / pause mobile rollout.
```

