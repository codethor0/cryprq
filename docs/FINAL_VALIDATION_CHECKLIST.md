# Final Post-Install Validation Checklist

**Run this before pressing the big green button** 🚀

## 1️⃣ Fast Sanity

```bash
./scripts/quick-smoke.sh || ./scripts/local-validate.sh
```

**Expected:**
- ✅ Docker fake backend responds on :9464
- ✅ Lint, typecheck, unit, E2E tests pass
- ✅ Desktop artifacts built

## 2️⃣ Observability and Telemetry Readiness

```bash
./scripts/observability-checks.sh
./scripts/sanity-checks.sh    # Answer 'y' for Check 6 to test flags/telemetry
```

**Expected:**
- ✅ Structured logs (JSONL v1) found
- ✅ Redaction OK (no secrets in logs)
- ✅ Flags file valid JSON
- ✅ Telemetry directory exists (if enabled)
- ✅ Telemetry redaction OK (if enabled)

## 3️⃣ Review PR Checklist or Create One Directly

```bash
gh pr create --fill --base main || echo "PR already open"
```

**Check PR includes:**
- ✅ All tests passing
- ✅ CI checks green
- ✅ Artifacts attached (if applicable)
- ✅ Changelog updated
- ✅ Documentation updated

## 4️⃣ (Optional) Trigger Full One-Shot Workflow

```bash
SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post
```

**This runs:**
- Quick-smoke → Full validation (if needed)
- Branch + PR creation
- CI mirror trigger
- Ship (go-live + verify-release)
- Post-actions (observability + sanity)
- Cleanup

---

## 🔍 After Release (Live Ops)

### Golden Path (Manual Check)

**Desktop:** Connect → Charts ≤3s → Rotate (toast ≤2s) → Disconnect

**Mobile:** Settings → Report Issue → Share sheet (<2MB, "Report Prepared")

### Every 2 Hours for Day-0

```bash
./scripts/observability-checks.sh
./scripts/sanity-checks.sh
```

### If Telemetry Enabled

```bash
tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl | grep event
```

**Watch for:**
- `connect` events (connection success)
- `rotation.completed` events (key rotations)
- `error` events (failures - investigate if spike)

---

## 🔁 Rollback (Emergency)

### Desktop

**Unmark "Latest" on GitHub release → re-promote prior stable tag**

1. Go to GitHub Releases
2. Find v1.1.0 release
3. Unmark "Latest" release
4. Re-promote previous stable version (e.g., v1.0.0)

### Mobile

**Pause rollout (Play Console) / expire build (TestFlight)**

**Android (Play Console):**
1. Open Play Console → Release → Production
2. Pause rollout
3. Revert to previous version if needed

**iOS (TestFlight):**
1. Open App Store Connect → TestFlight
2. Expire current build
3. Promote previous build if needed

---

## 💡 Optional Enhancements (Future)

- **Integrate telemetry counters into a lightweight dashboard** (Prometheus → JSON bridge)
- **Add Slack webhook for sanity-check summaries**
- **Auto-attach OPERATOR_CHEAT_SHEET.txt to PRs via GH Action**

---

## ✅ Pre-Release Checklist Summary

- [ ] Fast sanity passed (`quick-smoke.sh` or `local-validate.sh`)
- [ ] Observability checks passed (`observability-checks.sh`)
- [ ] Sanity checks passed (`sanity-checks.sh`)
- [ ] PR created/reviewed (all checks green)
- [ ] CI passed (GitHub Actions)
- [ ] Artifacts built and verified
- [ ] Changelog updated
- [ ] Documentation updated
- [ ] Golden path tested (manual)
- [ ] Rollback plan ready

**Ready to ship?** Run:

```bash
SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post
```

---

**Everything from validation to release, monitoring, and rollback is now automated and documented.**

