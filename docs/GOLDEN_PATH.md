# Golden Path Verification

**60-second sanity check for post-release**

## Desktop Golden Path

### Steps
1. **Launch app**: `cd gui && npm run dev`
2. **Connect**: Click "Connect" button
   - ✅ Status changes to "Connected" within ≤2s
   - ✅ Charts appear ("Throughput (last 60s)" and "Latency") within ≤3–5s
3. **Wait for rotation**: Monitor rotation timer
   - ✅ Toast appears within ≤2s: "Keys rotated securely at HH:MM:SS"
   - ✅ Countdown resets
4. **Disconnect**: Click "Disconnect" button
   - ✅ Status changes to "Disconnected" immediately
   - ✅ Charts stop updating

### Expected Behavior
- **Connection**: Smooth transition, no errors
- **Charts**: Render within 3–5s, update at ~1 Hz
- **Rotation**: Toast appears promptly, no duplicates
- **Disconnect**: Immediate status change, no orphan processes

## Mobile Golden Path

### Steps
1. **Open app**: Launch CrypRQ Mobile
2. **Navigate**: Settings → Report Issue
3. **Generate**: Tap "Generate & Share Diagnostics"
   - ✅ Share sheet opens
   - ✅ ZIP file <2MB
   - ✅ "Report Prepared" alert shown after share

### Expected Behavior
- **Share sheet**: Opens immediately
- **ZIP size**: <2MB (redacted)
- **Confirmation**: Alert confirms report prepared
- **Redaction**: No secrets visible in share preview

## Troubleshooting

### Desktop: Charts Don't Appear
```bash
## Check fake backend
curl http://localhost:9464/metrics

## Check connection status
## Look for "Connected" status in Dashboard
```

### Desktop: Rotation Toast Missing
- Check rotation timer is counting down
- Check console for errors
- Verify `toastStore` is working

### Mobile: Share Sheet Doesn't Open
- Check permissions (if required)
- Verify diagnostics generation succeeded
- Check console logs

## Quick Verification Commands

```bash
## Desktop: Check fake backend
curl -s http://localhost:9464/metrics | head -5

## Desktop: Check logs
tail -20 ~/.cryprq/logs/cryprq-*.log | jq -c 'fromjson | select(.event=="session.state")'

## Mobile: Check app logs (via adb/logcat)
adb logcat | grep -i cryprq | tail -20
```

---

**Time**: ~60 seconds per platform  
**Frequency**: Every 2h for first 24h, then daily

