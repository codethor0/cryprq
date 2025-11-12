# Post-Release Monitoring Plan

## Desktop Diagnostics

### Structured Logs Verification

Confirm JSONL v1 adoption in real logs:

```bash
# Check log schema compliance
jq -c 'fromjson | select(.event=="session.state") | .ts,.data' ~/.cryprq/logs/cryprq-*.log | head

# Verify no secrets in logs
grep -r "bearer\|privKey=" ~/.cryprq/logs/ || echo "✓ No secrets found"

# Count structured log entries
jq -c 'fromjson | select(.v==1)' ~/.cryprq/logs/cryprq-*.log | wc -l

# Check rotation events
jq -c 'fromjson | select(.event | startswith("rotation"))' ~/.cryprq/logs/cryprq-*.log | tail -20
```

### Key Metrics to Monitor

1. **Session State Transitions**
   - Average time to connect
   - Failure rate
   - Rotation success rate

2. **Structured Log Adoption**
   - Percentage of logs using v1 schema
   - Invalid log entries (should be minimal)

3. **Diagnostics Export Usage**
   - Export frequency
   - File sizes (<10MB)
   - Secret redaction effectiveness

### One-Liners for Quick Checks

```bash
# Last 10 session state changes
jq -c 'fromjson | select(.event=="session.state") | {ts,state:.data.state}' ~/.cryprq/logs/cryprq-*.log | tail -10

# Error rate (last 24h)
jq -c 'fromjson | select(.lvl=="error")' ~/.cryprq/logs/cryprq-$(date +%Y-%m-%d).log | wc -l

# Rotation count (last 24h)
jq -c 'fromjson | select(.event | startswith("rotation"))' ~/.cryprq/logs/cryprq-$(date +%Y-%m-%d).log | wc -l
```

---

## Mobile Crash Reporting

### Opt-In Gate Verification

Verify opt-in gate works (OFF → no egress):

1. **Test with toggle OFF**
   ```bash
   # Check network traffic (no calls to crash vendor)
   # Monitor app logs (no crash reporting initialization)
   ```

2. **Test with toggle ON**
   ```bash
   # Trigger test crash
   # Verify redacted report sent
   # Check crash vendor dashboard
   ```

### Monitoring Checklist

- [ ] Crash reporting toggle defaults to OFF
- [ ] No network calls when toggle is OFF
- [ ] Redacted reports when toggle is ON
- [ ] PII masked in crash metadata
- [ ] Endpoint URLs redacted
- [ ] Tokens redacted

---

## Issue Reporting Flow

### Desktop: "Report an Issue" Button

**Location**: Settings > Help > "Report an Issue"

**Flow**:
1. User clicks "Report an Issue"
2. Gathers:
   - Recent 200 log lines (redacted)
   - System info (OS, arch, app version)
   - Connection status
   - Recent errors
3. Opens diagnostics export dialog
4. User saves ZIP and shares via email/support

**Implementation**:
- Reuse `diagnostics:export` IPC handler
- Add "Report an Issue" button in Settings
- Show warning: "Sensitive data already redacted"

### Mobile: "Report an Issue" Flow

**Location**: Settings > Help > "Report an Issue"

**Flow**:
1. User taps "Report an Issue"
2. Gathers:
   - Recent 200 log lines (redacted)
   - System info (OS, device, app version)
   - Connection status
   - Recent errors
3. Creates ZIP (<2MB)
4. Presents share sheet (Android/iOS)
5. User shares via email/messaging

**Implementation**:
- Create `mobile/src/services/diagnostics.ts`
- Use `react-native-share` or native share sheet
- Verify no bearer/privKey tokens in ZIP

---

## Monitoring Dashboard (Future)

### Metrics to Track

1. **Adoption**
   - Active users
   - Daily/weekly active users
   - Platform distribution

2. **Reliability**
   - Crash rate
   - Session failure rate
   - Average session duration

3. **Performance**
   - Connection time
   - Rotation success rate
   - Throughput averages

4. **Errors**
   - Top error types
   - Error frequency
   - Error resolution rate

### Alert Thresholds

- Crash rate > 1%
- Session failure rate > 5%
- Average connection time > 10s
- Diagnostics export failures

---

## Weekly Review Checklist

- [ ] Review crash reports (if enabled)
- [ ] Review diagnostics exports (if shared)
- [ ] Check structured log adoption rate
- [ ] Verify secret redaction effectiveness
- [ ] Review user feedback
- [ ] Check error trends
- [ ] Review performance metrics

---

## Emergency Response

### Critical Issues

1. **Data Leak**
   - Immediately disable telemetry
   - Review logs for PII
   - Notify affected users
   - Patch and release hotfix

2. **High Crash Rate**
   - Review crash reports
   - Identify common patterns
   - Release hotfix
   - Communicate with users

3. **Security Vulnerability**
   - Assess severity
   - Patch immediately
   - Release security update
   - Notify users

### Communication Plan

- GitHub Issues for bug reports
- Email (codethor@gmail.com) for security issues
- Release notes for updates
- In-app notifications for critical updates

