# Ultra-Short Operator Loop

**Copy-paste ready commands for Day-0 monitoring**

##  Every 2h for First 24h

```bash
## Observability checks
./scripts/observability-checks.sh

## Sanity checks
./scripts/sanity-checks.sh

## Spot-check redaction on a fresh diagnostics zip
grep -R -E "bearer |privKey=|authorization:" path/to/exported/zip_unpacked || echo " redaction OK"
```

##  Golden-Path Sanity (60 seconds)

### Desktop
1. **Connect** → Charts appear ≤3–5s
2. **Rotate** → Toast ≤2s ("Keys rotated securely at HH:MM:SS")
3. **Disconnect** → Status changes immediately

### Mobile
1. **Settings → Report Issue** → Share sheet opens
2. **Verify**: ZIP <2MB, "Report Prepared" shown

##  If Something's Off

### Quick Rollback

**Desktop**:
1. Go to GitHub Releases
2. Find v1.1.0 → Edit → Uncheck "Set as the latest release"
3. Find previous stable (e.g., v1.0.1) → Edit → Check "Set as the latest release"
4. Save

**Mobile**:
- **Android**: Play Console → Release → Pause rollout
- **iOS**: App Store Connect → TestFlight → Expire build

### Triage

**Step 1: Get Diagnostics**
- Ask user for diagnostics ZIP
- Desktop: Help → Report Issue → Export Diagnostics
- Mobile: Settings → Report Issue → Share

**Step 2: Check Session Summary**
```bash
## Extract ZIP
unzip cryprq-diagnostics-*.zip -d /tmp/diag

## Check session-summary.json
cat /tmp/diag/session-summary.json | jq '.sessions, .stateDurations, .last50Events[-10:]'
```

**Look for**:
- State timeline anomalies
- Exit codes ≠ 0
- Long durations in "connecting" or "errored" states
- Frequent state transitions

**Step 3: Search JSONL Logs**
```bash
## Find error events around timestamp
TIMESTAMP="2025-01-15T1200Z"  # From user report
jq -c "select(.ts >= \"$TIMESTAMP\" and .ts <= \"$(date -u -d \"$TIMESTAMP +1 hour\" +%Y-%m-%dT%H:%M:%SZ)\") | select(.event == \"session.error\" or .event == \"cli.raw\" or .lvl == \"error\")" /tmp/diag/logs/*.log
```

**Common Issues**:
- `PORT_IN_USE`: Check Settings → Transport → UDP Port
- `PROCESS_EXITED`: Check exit code in session-summary.json
- `METRICS_UNREACHABLE`: Check endpoint URL (LOCAL/LAN/REMOTE)
- `INVALID_ENDPOINT`: Check REMOTE endpoint allowlist

##  Quick Reference

**All Checks**:
```bash
./scripts/observability-checks.sh && ./scripts/sanity-checks.sh
```

**Redaction Check**:
```bash
grep -R -E "bearer |privKey=|authorization:" ~/.cryprq/logs || echo " OK"
```

**Session State Timeline**:
```bash
jq -c 'fromjson | select(.event=="session.state") | [.ts,.data.state]' ~/.cryprq/logs/*.log | tail -20
```

**Recent Errors**:
```bash
jq -c 'fromjson | select(.lvl=="error") | [.ts,.event,.msg]' ~/.cryprq/logs/*.log | tail -10
```

##  Success Criteria

-  Observability checks: All PASS
-  Redaction: No secrets in logs
-  Golden path: Works consistently
-  Support tickets: 0 critical issues
-  Crash-free rate: ≥99.5%

##  Escalation

- **Level 1**: On-Call Engineer (run checks, triage diagnostics)
- **Level 2**: Engineering Lead (>1% crash rate, >5 tickets/2h)
- **Level 3**: CTO/Founder (data breach, service outage)

---

**Full Details**: See `docs/DAY0_ONCALL_CARD.md`

