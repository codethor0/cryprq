# Day-0 On-Call Card

**Release**: CrypRQ 1.1.0  
**Date**: [Release Date]  
**On-Call**: [Name]

## 🟢 Golden Path (60 seconds)

**Desktop**:
1. Connect → Charts appear ≤3–5s
2. Rotate → Toast ≤2s ("Keys rotated securely at HH:MM:SS")
3. Disconnect → Status changes immediately

**Mobile**:
1. Settings → Report Issue → Share sheet opens
2. Verify: ZIP <2MB, "Report Prepared" shown

**Quick Check**:
```bash
## See docs/GOLDEN_PATH.md for detailed steps
```

## 🔍 Quick Checks (Every 2h for First 24h)

**Copy-paste loop**:
```bash
## Observability checks
./scripts/observability-checks.sh

## Sanity checks
./scripts/sanity-checks.sh

## Spot-check redaction on fresh diagnostics zip
grep -R -E "bearer |privKey=|authorization:" path/to/exported/zip_unpacked || echo "✅ redaction OK"
```

**Expected**: All checks PASS, no secrets found

### 3. Support Inbox
**Check**: Support inbox (email/GitHub Issues)  
**Expected**: 0 unresolved "can't connect" reports  
**Action**: If >0, escalate immediately

### 4. Crash Reports (if enabled)
**Check**: Crash reporting dashboard (Sentry/Bugsnag)  
**Expected**: Crash-free rate ≥99.5%  
**Action**: If below threshold, investigate immediately

## 🚨 If Something's Off

### Quick Rollback

### Desktop Rollback

**Option 1: Unmark Latest Release (Fastest)**
1. Go to GitHub Releases
2. Find v1.1.0 release
3. Click "Edit release"
4. Uncheck "Set as the latest release"
5. Edit previous stable release (e.g., v1.0.1)
6. Check "Set as the latest release"
7. Save

**Option 2: Cut Back-Out Tag**
```bash
## If you need to cut a new release from previous version
git checkout v1.0.1  # or last stable tag
./scripts/go-live.sh 1.0.2  # or appropriate version
```

**Verification**:
- Users downloading from GitHub will get previous stable version
- CI will rebuild previous stable artifacts

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

**Step 3: Search JSONL Logs**
```bash
## Find error events around timestamp
TIMESTAMP="2025-01-15T12:00:00Z"  # From user report
jq -c "select(.ts >= \"$TIMESTAMP\" and .ts <= \"$(date -u -d \"$TIMESTAMP +1 hour\" +%Y-%m-%dT%H:%M:%SZ)\") | select(.event == \"session.error\" or .event == \"cli.raw\" or .lvl == \"error\")" /tmp/diag/logs/*.log
```

**Common Issues**:
- `PORT_IN_USE`: Check Settings → Transport → UDP Port
- `PROCESS_EXITED`: Check exit code in session-summary.json
- `METRICS_UNREACHABLE`: Check endpoint URL (LOCAL/LAN/REMOTE)
- `INVALID_ENDPOINT`: Check REMOTE endpoint allowlist

### Mobile Rollback

**Android (Play Console)**:
1. Go to Play Console → Release → Production/Internal
2. Find v1.1.0 release
3. Click "Pause rollout" or "Halt release"
4. Previous version will automatically serve to users

**iOS (TestFlight)**:
1. Go to App Store Connect → TestFlight
2. Find v1.1.0 build
3. Click "Expire build"
4. Previous build will automatically serve to testers

**Verification**:
- Android: Check Play Console → Release dashboard
- iOS: Check TestFlight → Builds list

## 📞 Escalation Path

**Level 1: On-Call Engineer**
- Check observability checks
- Review diagnostics
- Check support inbox

**Level 2: Engineering Lead**
- If crash rate >1%
- If >5 support tickets in 2h
- If critical security issue

**Level 3: CTO/Founder**
- If data breach suspected
- If service-wide outage
- If rollback fails

## 📋 Day-0 Checklist

**Hour 0 (Immediate)**:
- [ ] Run `./scripts/observability-checks.sh`
- [ ] Verify golden path works
- [ ] Check support inbox
- [ ] Monitor crash reports

**Hour 2**:
- [ ] Re-run observability checks
- [ ] Check diagnostics redaction
- [ ] Review support tickets
- [ ] Check GitHub Issues

**Hour 4**:
- [ ] Re-run observability checks
- [ ] Check crash-free rate
- [ ] Review error logs
- [ ] Check metrics dashboard

**Hour 6**:
- [ ] Re-run observability checks
- [ ] Review support tickets
- [ ] Check for any anomalies

**Hour 8**:
- [ ] Re-run observability checks
- [ ] Review crash reports
- [ ] Check metrics trends

**Hour 12**:
- [ ] Re-run observability checks
- [ ] Review all support tickets
- [ ] Check for patterns in errors

**Hour 24**:
- [ ] Final observability check
- [ ] Review all metrics
- [ ] Document any issues
- [ ] Hand off to regular monitoring

## 🎯 Success Criteria

**Day-0 Success**:
- ✅ All observability checks PASS
- ✅ Crash-free rate ≥99.5%
- ✅ 0 critical support tickets
- ✅ No security incidents
- ✅ Golden path works consistently

**If any criteria fails**: Escalate immediately

## 📝 Notes

- Keep this card handy during first 24h
- Update with any issues encountered
- Document resolution steps for future reference

