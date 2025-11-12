#  Production Ready - Final Checklist

## Final Green Button

```bash
./scripts/go-live.sh 1.1.0 && ./scripts/verify-release.sh
```

##  Quick Reference Cards

- **Operator Loop**: `docs/OPERATOR_LOOP.md` (copy-paste commands)
- **Golden Path**: `docs/GOLDEN_PATH.md` (60-second verification)
- **Day-0 On-Call**: `docs/DAY0_ONCALL_CARD.md` (full guide)
- **Local Validation**: `docs/LOCAL_VALIDATION.md` (pre-flight checks)

##  Every 2h for First 24h

```bash
./scripts/observability-checks.sh
./scripts/sanity-checks.sh
```

##  Golden Path (60 seconds)

**Desktop**: Connect → Charts ≤3–5s → Rotate (toast ≤2s) → Disconnect  
**Mobile**: Settings → Report Issue → Share sheet (<2MB, "Report Prepared")

##  Quick Rollback

- **Desktop**: GitHub Release → Unmark "Latest" → Re-promote previous
- **Mobile**: Play Console → Pause | TestFlight → Expire

##  Triage

1. Get diagnostics ZIP
2. Check `session-summary.json` → state timeline & exit codes
3. Search JSONL for `session.error`/`cli.raw` around timestamp

---

**Status**:  **READY FOR PRODUCTION**

