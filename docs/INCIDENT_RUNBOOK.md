# Quick Incident Runbook

## User Can't Connect

### Step 1: Gather Information
Request from user:
1. **Diagnostics ZIP** (Help → Export Diagnostics)
2. **App version** (Settings → About)
3. **OS version** (macOS/Windows/Linux + version)
4. **Steps to reproduce** (what were they doing?)
5. **Timestamp** (when did it occur?)

### Step 2: Analyze Diagnostics

```bash
## Extract diagnostics ZIP
unzip -q cryprq-diagnostics-*.zip -d /tmp/diag-analysis

## Check session summary
cat /tmp/diag-analysis/session-summary.json | jq '.sessions'

## Check last events
cat /tmp/diag-analysis/session-summary.json | jq '.last50Events[-10:]'

## Check for errors
cat /tmp/diag-analysis/logs/*.log | jq -c 'fromjson | select(.lvl=="error")' | tail -20
```

### Step 3: Common Issues & Fixes

#### PORT_IN_USE
**Symptoms:**
- Error in logs: `bind: address already in use`
- `exitCode: 1` in session summary

**Fix:**
1. Ask user to check Settings → Transport → UDP Port
2. Suggest changing port (e.g., 9999 → 10000)
3. Check for other CrypRQ instances:
   ```bash
   # macOS/Linux
   lsof -i :9999
   
   # Windows
   netstat -ano | findstr :9999
   ```
4. Retry connection

---

#### CLI_NOT_FOUND
**Symptoms:**
- Error: `BINARY_NOT_FOUND` or `ENOENT`
- Session fails to start immediately

**Fix:**
1. Verify CrypRQ binary is installed:
   ```bash
   # Check if binary exists
   which cryprq  # macOS/Linux
   where cryprq  # Windows
   ```
2. Check PATH environment variable
3. Reinstall app
4. If using custom binary path, verify Settings → Advanced → Binary Path

---

#### METRICS_TIMEOUT
**Symptoms:**
- Metrics not updating
- Connection appears stuck
- Timeout errors in logs

**Fix:**
1. Check network connectivity:
   ```bash
   # Test metrics endpoint
   curl http://127.0.0.1:9464/metrics  # LOCAL
   curl http://192.168.1.100:9464/metrics  # LAN
   ```
2. Verify metrics endpoint is accessible
3. Check firewall settings
4. Restart session
5. If using REMOTE profile, verify HTTPS endpoint is reachable

---

#### PEER_UNREACHABLE
**Symptoms:**
- Connection fails after "starting" state
- Reachability test fails
- Timeout in logs

**Fix:**
1. Verify peer multiaddr is correct
2. Test reachability:
   ```bash
   # Extract hostname and port from multiaddr
   # Test TCP connection
   nc -zv <hostname> <port>
   ```
3. Check firewall/NAT settings
4. Verify peer is online
5. Try different peer

---

## Rotation Confusion

### Step 1: Verify Rotation Events

```bash
## Check logs for rotation events
cat ~/.cryprq/logs/cryprq-*.log | jq -c 'fromjson | select(.event | startswith("rotation"))' | tail -10
```

**Expected events:**
- `rotation.scheduled` - Rotation scheduled
- `rotation.started` - Rotation in progress
- `rotation.completed` - Rotation finished

### Step 2: Check Countdown Sync

**Dashboard should show:**
- Countdown decrements every second
- Resyncs from metrics within 2s of rotation completion
- Status changes to "Rotating Keys" during rotation

**If countdown not updating:**
1. Check metrics endpoint is reachable
2. Verify `rotationTimer` in metrics response
3. Check Dashboard → rotation timer display
4. Restart app if needed

### Step 3: Rotation Not Completing

**Symptoms:**
- Rotation starts but never completes
- Connection drops during rotation
- `rotation.completed` event missing

**Fix:**
1. Check network stability (rotation requires peer communication)
2. Verify peer is reachable
3. Check logs for rotation errors:
   ```bash
   cat ~/.cryprq/logs/cryprq-*.log | jq -c 'fromjson | select(.event=="rotation.error")'
   ```
4. Increase rotation interval if network is unstable:
   - Settings → Key Rotation → Increase interval
5. Restart session

---

## Windows Launch Blocked

### Step 1: Verify Signature

```powershell
## On Windows
signtool verify /pa CrypRQ.exe
```

**Expected (signed):**
```
Successfully verified: CrypRQ.exe
```

**Expected (unsigned dev build):**
```
SignTool Error: No signature found
```

### Step 2: Handle Unsigned Build

**If unsigned (dev build):**
1. User must click "More info" → "Run anyway"
2. This is expected for unsigned dev builds
3. Replace with signed build when available

**Instructions for user:**
```
1. Right-click CrypRQ.exe → Properties
2. If "Unknown publisher" warning appears:
   - Click "More info"
   - Click "Run anyway"
3. Note: This is expected for unsigned dev builds
4. A signed build will be available in the next release
```

### Step 3: Handle Signed but Blocked

**If signed but still blocked:**
1. Check certificate validity:
   ```powershell
   signtool verify /pa /v CrypRQ.exe
   ```
2. Verify timestamp server response
3. Check Windows Defender exclusions
4. May need to rebuild with valid certificate
5. Contact support if issue persists

---

## macOS Gatekeeper Block

### Step 1: Verify Notarization

```bash
spctl --assess --type open --verbose CrypRQ.dmg
```

**Expected (notarized):**
```
CrypRQ.dmg: accepted
```

**Expected (not notarized):**
```
CrypRQ.dmg: rejected
```

### Step 2: Handle Not Notarized

**If not notarized (dev build):**
1. User must right-click → Open → "Open" button
2. This is expected for unsigned/not notarized dev builds
3. Replace with notarized build when available

**Instructions for user:**
```
1. Right-click CrypRQ.dmg → Open
2. If "unidentified developer" warning appears:
   - Click "Open" button (not "Move to Trash")
3. Note: This is expected for unsigned dev builds
4. A notarized build will be available in the next release
```

### Step 3: Handle Notarized but Blocked

**If notarized but still blocked:**
1. Check notarization status:
   ```bash
   xcrun stapler validate CrypRQ.dmg
   ```
2. Verify stapling:
   ```bash
   xcrun stapler staple CrypRQ.dmg
   ```
3. Check System Preferences → Security & Privacy
4. May need to rebuild and re-notarize
5. Contact support if issue persists

---

## Diagnostics Export Issues

### Export Fails

**Symptoms:**
- "Export Diagnostics" button does nothing
- Error message appears
- ZIP file not created

**Fix:**
1. Check disk space:
   ```bash
   df -h ~  # macOS/Linux
   ```
2. Check write permissions:
   ```bash
   ls -ld ~/.cryprq/logs
   ```
3. Check logs for errors:
   ```bash
   cat ~/.cryprq/logs/cryprq-*.log | jq -c 'fromjson | select(.event=="diagnostics.error")'
   ```
4. Try manual export:
   - Copy `~/.cryprq/logs/` directory
   - Create ZIP manually
   - Include `~/.cryprq/settings.json` (redact secrets first)

### ZIP Too Large

**Symptoms:**
- Export succeeds but ZIP > 10MB
- Slow to upload/share

**Fix:**
1. Check log file sizes:
   ```bash
   ls -lh ~/.cryprq/logs/
   ```
2. Rotate old logs:
   ```bash
   # Keep only last 7 days
   find ~/.cryprq/logs -name "*.log" -mtime +7 -delete
   ```
3. Re-export diagnostics
4. If still large, manually select recent log files

---

## Performance Issues

### High CPU Usage

**Symptoms:**
- App uses > 50% CPU
- System becomes sluggish

**Fix:**
1. Check for excessive logging:
   - Settings → Logging → Reduce log level to "warn" or "error"
2. Check metrics polling interval:
   - Verify metrics polling is not too frequent (< 1s)
3. Check for stuck processes:
   ```bash
   ps aux | grep cryprq
   ```
4. Restart app
5. If persists, export diagnostics and contact support

### High Memory Usage

**Symptoms:**
- App uses > 500MB RAM
- System becomes slow

**Fix:**
1. Check log buffer size:
   - Settings → Logging → Reduce log retention
2. Clear logs:
   - Settings → Logging → Clear Logs
3. Restart app
4. If persists, check for memory leaks in diagnostics

---

## Network Issues

### Can't Connect to Peer

**Symptoms:**
- Connection fails immediately
- "Peer unreachable" error

**Fix:**
1. Test peer reachability:
   ```bash
   # Extract hostname:port from multiaddr
   # Test TCP connection
   nc -zv <hostname> <port>
   ```
2. Check firewall settings
3. Verify NAT traversal (if behind NAT)
4. Try different peer
5. Check peer is online and running CrypRQ

### Connection Drops Frequently

**Symptoms:**
- Connection established but drops after few minutes
- Frequent reconnection attempts

**Fix:**
1. Check network stability:
   ```bash
   ping -c 10 <peer-hostname>
   ```
2. Check for rotation issues (see Rotation Confusion section)
3. Increase rotation interval if network is unstable
4. Check firewall/NAT timeout settings
5. Verify peer is stable

---

## Escalation

If issue cannot be resolved with above steps:

1. **Gather full diagnostics:**
   - Export diagnostics ZIP
   - Include system info (OS version, hardware)
   - Include steps to reproduce

2. **Contact support:**
   - Email: codethor@gmail.com
   - Include diagnostics ZIP
   - Include support token from Report Issue modal

3. **Check known issues:**
   - GitHub Issues: [Repository URL]/issues
   - Check for similar reported issues

---

**Last Updated:** 2025-01-15

