# Post-Install Sanity Card

**Non-invasive checks** - Telemetry stays OFF by default. Run anytime.

## Feature Flags Quick Check

### 1) Confirm defaults are applied

```bash
jq . config/flags.json
```

**Expected output:**
```json
{
  "enableCharts": true,
  "enableTrayEnhancements": true,
  "enableNewToasts": true
}
```

### 2) Temp-disable charts via ENV (no rebuild)

```bash
CRYPRQ_FLAGS='{"enableCharts":false}' npm run dev
```

**Expected behavior:**
- Dashboard should hide charts component
- Revert by restarting without ENV

**Verify:**
- Open Dashboard → Charts should not appear
- Check browser console for flag state (if dev tools open)

## Telemetry v0 Opt-In Smoke (Local Only)

### 1) Enable in-app

**Settings → Privacy → Enable telemetry**

### 2) Watch file creation

```bash
tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl
```

**Expected:**
- File should appear within seconds of enabling
- Directory created: `~/.cryprq/telemetry/`

### 3) Exercise: Connect → Rotate → Disconnect

**Actions:**
1. Connect to a peer
2. Wait for rotation (or trigger manually if available)
3. Disconnect

**Expected telemetry events:**
```jsonl
{"v":1,"ts":"2025-01-15T10:30:00.000Z","event":"app.open","appVersion":"1.1.0","platform":"darwin","data":{}}
{"v":1,"ts":"2025-01-15T10:30:05.000Z","event":"connect","appVersion":"1.1.0","platform":"darwin","data":{}}
{"v":1,"ts":"2025-01-15T10:35:00.000Z","event":"rotation.completed","appVersion":"1.1.0","platform":"darwin","data":{}}
{"v":1,"ts":"2025-01-15T10:40:00.000Z","event":"disconnect","appVersion":"1.1.0","platform":"darwin","data":{}}
{"v":1,"ts":"2025-01-15T10:45:00.000Z","event":"app.quit","appVersion":"1.1.0","platform":"darwin","data":{}}
```

**Verify:**
- Minimal, redacted lines
- No PII (peer IDs, tokens, etc. should be redacted)
- Events: `app.open`, `connect`, `rotation.completed`, `disconnect`, `app.quit`

## Observability Hooks (Keep Commented Until Ready)

**Uncomment the telemetry section in `scripts/observability-checks.sh` when you want Day-0 KPIs.**

**Counters to rely on:**
- `connect` - Connection events
- `rotation.completed` - Rotation events
- `error` - Error events (all redacted, no PII)

**Example (when uncommented):**
```bash
./scripts/observability-checks.sh
```

## Day-0 Flip Switches (Instant Rollback)

### Kill a feature at runtime (no rebuild)

**Edit `config/flags.json` → save; UI hot-reloads.**

```bash
## Disable charts
vim config/flags.json
## Change: "enableCharts": false
## Save → UI updates automatically
```

**Verify:**
- Charts disappear from Dashboard (if `enableCharts: false`)
- No rebuild required
- Restart app to revert

### Emergency: Start with ENV override

```bash
CRYPRQ_FLAGS='{"enableCharts":false,"enableTrayEnhancements":false}' npm run dev
```

**Use cases:**
- Quick feature disable for testing
- Emergency rollback without code changes
- A/B testing different flag combinations

## Quick Smoke Test (30 seconds)

**Run this anytime to verify flags + telemetry:**

```bash
## 1. Check flags file exists and is valid JSON
jq . config/flags.json > /dev/null && echo "✅ Flags file valid"

## 2. Check telemetry directory exists (if telemetry was enabled)
[ -d ~/.cryprq/telemetry ] && echo "✅ Telemetry directory exists" || echo "ℹ️  Telemetry not enabled yet"

## 3. Check latest telemetry file (if exists)
if [ -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl ]; then
  COUNT=$(wc -l < ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl)
  echo "✅ Telemetry active: $COUNT events today"
else
  echo "ℹ️  No telemetry events today (opt-in)"
fi
```

## Production Readiness Checklist

- ✅ Feature flags load from `config/flags.json`
- ✅ ENV override works (`CRYPRQ_FLAGS`)
- ✅ Telemetry OFF by default
- ✅ Telemetry opt-in works (Settings → Privacy)
- ✅ Telemetry events are redacted (no PII)
- ✅ Flags hot-reload without rebuild
- ✅ Emergency ENV override works

## Troubleshooting

**Flags not loading?**
- Check `config/flags.json` exists and is valid JSON
- Check ENV variable format: `CRYPRQ_FLAGS='{"key":value}'`
- Restart app if file was edited

**Telemetry not writing?**
- Check telemetry is enabled: Settings → Privacy → Enable telemetry
- Check directory permissions: `~/.cryprq/telemetry/`
- Check disk space

**Hot-reload not working?**
- File watching may not work in all environments
- Restart app if flags don't update
- Use ENV override for immediate effect

---

**You're fully production-ready, with an on-demand parachute.**

