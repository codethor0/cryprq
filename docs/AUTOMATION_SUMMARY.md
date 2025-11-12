# Automation Summary

**Complete automation from validation to release, monitoring, and rollback**

## 🎯 Single Entry Point

**One command to rule them all:**

```bash
./scripts/one-shot.sh --ship --post
```

**This single command:**
- ✅ Validates (quick-smoke → full validation)
- ✅ Creates PR (branch + PR + CI trigger)
- ✅ Ships (go-live + verify-release)
- ✅ Monitors (observability + sanity checks)
- ✅ Cleans up (Docker + artifacts)

## 📦 Components

### 1. One-Shot Orchestration (`scripts/one-shot.sh`)

**Single entry for validation → CI → optional release**

**Features:**
- Preconditions check (Docker, Node.js, Git)
- Quick-smoke → Full validation escalation
- Branch + PR creation (idempotent)
- CI mirror trigger
- Optional ship (guarded)
- Post-actions (observability + sanity)
- Cleanup
- Compact summary output

**Usage:**
```bash
## Basic (validate + PR)
./scripts/one-shot.sh

## With ship
SHIP=true ./scripts/one-shot.sh --ship

## Full workflow
SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post
```

### 2. Quick-Smoke / Cleanup / Sanity / Observability Scripts

**Fast safety checks**

**Scripts:**
- `scripts/quick-smoke.sh` - Fast local sanity (30s)
- `scripts/local-validate.sh` - Full validation (comprehensive)
- `scripts/cleanup.sh` - Docker + artifact cleanup
- `scripts/sanity-checks.sh` - Pre-release sanity checks
- `scripts/observability-checks.sh` - Post-release monitoring

**Usage:**
```bash
## Quick sanity
./scripts/quick-smoke.sh

## Full validation
./scripts/local-validate.sh

## Cleanup
./scripts/cleanup.sh

## Pre-release sanity
./scripts/sanity-checks.sh

## Post-release monitoring
./scripts/observability-checks.sh
```

### 3. Feature Flags + Telemetry v0

**Runtime rollback & early health visibility**

**Feature Flags:**
- `config/flags.json` - Runtime toggles (hot-reload)
- ENV override: `CRYPRQ_FLAGS='{"enableCharts":false}'`
- Priority: `defaults < file < env`

**Telemetry v0:**
- Opt-in (Settings → Privacy → Enable telemetry)
- Events: `app.open`, `connect`, `disconnect`, `rotation.completed`, `error`
- Storage: `~/.cryprq/telemetry/events-YYYY-MM-DD.jsonl`
- Redaction: All secrets automatically sanitized

**Usage:**
```bash
## Disable features at runtime
CRYPRQ_FLAGS='{"enableCharts":false}' npm run dev

## Watch telemetry (if enabled)
tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl
```

### 4. Operator & Sanity Docs

**Everything copy-paste ready for PRs and Day-0**

**Documents:**
- `OPERATOR_CHEAT_SHEET.txt` - Copy-paste ready cheat sheet
- `docs/OPERATOR_CHEAT_SHEET.md` - Formatted version
- `docs/POST_INSTALL_SANITY.md` - Post-install sanity card
- `docs/FINAL_VALIDATION_CHECKLIST.md` - Pre-release checklist
- `docs/ONE_SHOT_WORKFLOW.md` - One-shot workflow guide
- `docs/TROUBLESHOOTING.md` - Fast triage guide

**Usage:**
- Copy `OPERATOR_CHEAT_SHEET.txt` → paste in PR comments
- Reference `FINAL_VALIDATION_CHECKLIST.md` before release
- Use `TROUBLESHOOTING.md` for common issues

## 🔄 Complete Workflow

### Pre-Release

```bash
## 1. Fast sanity
./scripts/quick-smoke.sh || ./scripts/local-validate.sh

## 2. Observability + sanity
./scripts/observability-checks.sh
./scripts/sanity-checks.sh

## 3. Create PR
gh pr create --fill --base main

## 4. Full one-shot (optional)
SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post
```

### Post-Release (Day-0)

```bash
## Every 2 hours
./scripts/observability-checks.sh
./scripts/sanity-checks.sh

## Watch telemetry (if enabled)
tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl | grep event
```

### Rollback (Emergency)

**Desktop:** Unmark "Latest" on GitHub release → re-promote prior stable

**Mobile:** Pause rollout (Play Console) / expire build (TestFlight)

## 📊 Monitoring

### Golden Path (Manual Check)

**Desktop:** Connect → Charts ≤3s → Rotate (toast ≤2s) → Disconnect

**Mobile:** Settings → Report Issue → Share sheet (<2MB, "Report Prepared")

### Automated Checks

- Structured logs (JSONL v1)
- Redaction (no secrets)
- Telemetry events (if enabled)
- Feature flags (runtime toggles)

## 🎉 Summary

**Everything from validation to release, monitoring, and rollback is now automated and documented.**

**At this point, running:**

```bash
./scripts/one-shot.sh --ship --post
```

**…is all it takes to test, build, release, verify, and monitor CrypRQ 1.1.0 across desktop and mobile.**

## 📚 Quick Reference

**Most-used commands:**
- `./scripts/one-shot.sh` - Validate + PR
- `SHIP=true ./scripts/one-shot.sh --ship` - Validate + PR + Ship
- `./scripts/quick-smoke.sh` - Fast sanity (30s)
- `./scripts/observability-checks.sh` - Post-release monitoring

**Documentation:**
- `OPERATOR_CHEAT_SHEET.txt` - Copy-paste ready
- `docs/FINAL_VALIDATION_CHECKLIST.md` - Pre-release checklist
- `docs/TROUBLESHOOTING.md` - Common issues

**Feature flags:**
- `config/flags.json` - Runtime toggles
- `CRYPRQ_FLAGS='{"enableCharts":false}' npm run dev` - ENV override

**Telemetry:**
- Settings → Privacy → Enable telemetry (opt-in)
- `tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl` - Watch events

---

**You're fully production-ready with an on-demand parachute.** 🚀

