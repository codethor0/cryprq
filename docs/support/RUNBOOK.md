# Support Runbook

## What to Request from Users

When users report issues, request:

1. **App Version**
   - Desktop: Settings > About
   - Mobile: Settings > About

2. **Operating System**
   - Desktop: OS version (macOS, Windows, Linux)
   - Mobile: Android version / iOS version

3. **Steps to Reproduce**
   - What were you doing when the issue occurred?
   - Can you reproduce it consistently?

4. **Diagnostics Export**
   - Desktop: Help > Export Diagnostics
   - Mobile: Settings > Help > Report an Issue

5. **Timestamp**
   - When did the issue occur?
   - Timezone?

6. **Screenshots/Logs**
   - Error messages
   - Screenshots of UI issues

---

## How to Read Diagnostics

### Desktop Diagnostics ZIP

Structure:
```
diagnostics-YYYY-MM-DD-HHMMSS.zip
 system-info.json
 settings.json (redacted)
 session-summary.json
 metrics-snapshot.json
 logs/
    cryprq-YYYY-MM-DD.log (JSONL)
 README.txt
```

### Key Files

#### `session-summary.json`
```json
{
  "timestamp": "2025-01-15T1000Z",
  "last50Events": [...],
  "sessions": {
    "total": 5,
    "failures": 1,
    "meanTimeToConnect": 3.2
  },
  "rotations": {
    "count": 12,
    "averageInterval": 300
  },
  "stateDurations": {
    "connecting": 2.1,
    "connected": 1800.5,
    "errored": 0
  }
}
```

**How to read**:
- Check `stateDurations` for abnormal states
- Review `last50Events` for recent state transitions
- Check `sessions.failures` for failure rate

#### `logs/cryprq-*.log` (JSONL)

Each line is a JSON object:
```json
{"v":1,"ts":"2025-01-15T1000Z","lvl":"error","src":"cli","event":"session.error","msg":"Connection failed","data":{"error":"TIMEOUT"}}
```

**How to read**:
```bash
## Filter by event type
jq -c 'fromjson | select(.event=="session.error")' logs/cryprq-*.log

## Filter by level
jq -c 'fromjson | select(.lvl=="error")' logs/cryprq-*.log

## Get state transitions
jq -c 'fromjson | select(.event=="session.state") | {ts,state:.data.state}' logs/cryprq-*.log
```

---

## Common Issues & Fixes

### PORT_IN_USE

**Symptoms**: Cannot start session, port already in use

**Diagnosis**:
```bash
## Check session-summary.json for error events
jq '.last50Events[] | select(.event=="session.error")' session-summary.json
```

**Fix**:
1. Change UDP port in Settings
2. Check for other CrypRQ instances running
3. Restart app

---

### CLI_NOT_FOUND

**Symptoms**: Session fails to start, binary not found

**Diagnosis**:
```bash
## Check logs for "BINARY_NOT_FOUND"
jq -c 'fromjson | select(.msg | contains("BINARY_NOT_FOUND"))' logs/cryprq-*.log
```

**Fix**:
1. Verify CrypRQ binary is installed
2. Check PATH environment variable
3. Reinstall app

---

### METRICS_TIMEOUT

**Symptoms**: Metrics not updating, connection appears stuck

**Diagnosis**:
```bash
## Check metrics-snapshot.json timestamp
jq '.timestamp' metrics-snapshot.json

## Check for timeout events
jq -c 'fromjson | select(.event=="metrics.timeout")' logs/cryprq-*.log
```

**Fix**:
1. Check network connectivity
2. Verify metrics endpoint is accessible
3. Restart session

---

### ROTATION_FAILED

**Symptoms**: Key rotation not completing, connection drops

**Diagnosis**:
```bash
## Check rotation events
jq -c 'fromjson | select(.event | startswith("rotation"))' logs/cryprq-*.log

## Check session summary
jq '.rotations' session-summary.json
```

**Fix**:
1. Check network stability
2. Verify peer is reachable
3. Increase rotation interval if needed

---

## Triage Workflow

### Step 1: Initial Assessment
1. Read user description
2. Check app version (known issues?)
3. Review diagnostics ZIP (if provided)

### Step 2: Diagnosis
1. Check `session-summary.json` for patterns
2. Review `last50Events` for recent issues
3. Search logs for error events

### Step 3: Reproduction
1. Try to reproduce locally
2. Check if issue is environment-specific
3. Test with fake backend if applicable

### Step 4: Resolution
1. Apply fix (if known)
2. Escalate if needed
3. Document solution

---

## Escalation Path

### Level 1: Support Team
- Common issues
- User-facing problems
- Configuration issues

### Level 2: Engineering Team
- Bugs requiring code changes
- Performance issues
- Security concerns

### Level 3: Critical Issues
- Data leaks
- Security vulnerabilities
- Widespread outages

---

## Tools & Commands

### Desktop Log Analysis
```bash
## Last 10 errors
jq -c 'fromjson | select(.lvl=="error")' ~/.cryprq/logs/cryprq-*.log | tail -10

## State transitions
jq -c 'fromjson | select(.event=="session.state") | {ts,state:.data.state}' ~/.cryprq/logs/cryprq-*.log | tail -20

## Rotation events
jq -c 'fromjson | select(.event | startswith("rotation"))' ~/.cryprq/logs/cryprq-*.log | tail -10
```

### Redaction Check
```bash
## Verify no secrets leaked
if grep -R -E "bearer |privKey=|authorization:" ~/.cryprq/logs; then
  echo " Secrets leaked!"
else
  echo " Redaction OK"
fi
```

### Mobile Log Analysis
```bash
## Android logs
adb logcat | grep -i cryprq

## iOS logs (device)
idevicesyslog | grep -i cryprq
```

---

## Response Templates

### Initial Response
```
Thank you for reporting this issue. To help us diagnose the problem, please provide:

1. App version: [from Settings > About]
2. OS version: [your OS]
3. Steps to reproduce: [what you were doing]
4. Diagnostics export: [Help > Export Diagnostics]
5. Timestamp: [when it occurred]

We'll investigate and get back to you soon.
```

### Follow-up (Issue Reproduced)
```
We've reproduced the issue and are working on a fix. Expected resolution: [timeframe].

In the meantime, you can try: [workaround]
```

### Follow-up (Issue Resolved)
```
The issue has been fixed in version [X.Y.Z]. Please update to the latest version.

If you continue to experience issues, please let us know.
```

---

**Last Updated**: 2025-01-15

