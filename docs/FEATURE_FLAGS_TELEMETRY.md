# Feature Flags & Telemetry v0

**Safety Net Patch Pack** - Runtime feature toggles and opt-in telemetry for Day-0 health checks.

## Feature Flags

### Overview

Runtime feature toggles via JSON file or ENV variable. Hot-reloads without rebuild.

### Files

- `config/flags.json` - Default flags (committed)
- `gui/src/flags/index.ts` - Flag loader with ENV override
- `gui/src/store/useFlags.ts` - Zustand store with hot-reload

### Flags

```json
{
  "enableCharts": true,
  "enableTrayEnhancements": true,
  "enableNewToasts": true
}
```

### Usage

**Edit flags.json** (hot-reloads):
```bash
## Edit config/flags.json
vim config/flags.json
## Save - UI updates automatically
```

**ENV override** (per-process):
```bash
CRYPRQ_FLAGS='{"enableCharts":false}' npm run dev
```

**Priority**: `defaults < file < env`

### Wired Features

- **Dashboard**: Charts component gated by `enableCharts`
- **Tray**: Enhancements gated by `enableTrayEnhancements` (when implemented)
- **Toasts**: New toast system gated by `enableNewToasts` (when implemented)

## Telemetry v0

### Overview

Opt-in event counters (no PII). Stored locally in JSONL format.

### Files

- `gui/electron/main/telemetry.ts` - Event emitter
- `gui/src/components/Settings/Settings.tsx` - Privacy toggle
- `gui/electron/main.ts` - App lifecycle hooks
- `gui/electron/main/session.ts` - Session event hooks

### Events

- `app.open` - App launched
- `app.quit` - App quit
- `connect` - Session connected
- `disconnect` - Session disconnected
- `rotation.completed` - Key rotation completed
- `error` - Error occurred (with code/exitCode)

### Storage

**Location**: `~/.cryprq/telemetry/events-YYYY-MM-DD.jsonl`

**Format**:
```json
{"v":1,"ts":"2025-01-15T1000.000Z","event":"connect","appVersion":"1.1.0","platform":"darwin","data":{}}
```

**Redaction**: All strings are sanitized (bearer tokens, private keys, authorization headers)

### Enable Telemetry

1. Open Settings → Privacy
2. Check "Enable telemetry (opt-in)"
3. Events start writing to `~/.cryprq/telemetry/events-*.jsonl`

**Default**: OFF (opt-in only)

### Observability

**Parse telemetry** (when enabled):
```bash
## Count events
jq -cr 'fromjson | select(.event=="connect")' ~/.cryprq/telemetry/events-*.jsonl | wc -l

## Connect success rate
CONNECT=$(jq -cr 'fromjson | select(.event=="connect")' ~/.cryprq/telemetry/events-*.jsonl | wc -l)
ERROR=$(jq -cr 'fromjson | select(.event=="error")' ~/.cryprq/telemetry/events-*.jsonl | wc -l)
echo "Success rate: $(( ($CONNECT - $ERROR) * 100 / $CONNECT ))%"
```

**Observability script** (`scripts/observability-checks.sh`):
- Telemetry parsing is commented out by default
- Uncomment when telemetry is enabled
- Includes connect/disconnect/error/rotation counters

## How to Flip a Feature at Runtime

### Method 1: Edit flags.json

```bash
## Edit config/flags.json
vim config/flags.json

## Change enableCharts to false
{
  "enableCharts": false,
  "enableTrayEnhancements": true,
  "enableNewToasts": true
}

## Save - UI hot-reloads (if watching enabled)
```

### Method 2: ENV Override

```bash
## Disable charts for this run
CRYPRQ_FLAGS='{"enableCharts":false}' npm run dev

## Disable multiple flags
CRYPRQ_FLAGS='{"enableCharts":false,"enableNewToasts":false}' npm run dev
```

## Turn on Telemetry v0

1. **Settings → Privacy → Enable telemetry**
2. **Confirm**: `~/.cryprq/telemetry/events-YYYY-MM-DD.jsonl` starts filling

**Verify**:
```bash
## Check telemetry directory exists
ls -la ~/.cryprq/telemetry/

## View recent events
tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl
```

## Privacy & Security

- **No PII**: Only event types and metadata (redacted)
- **Local storage**: All data stored locally
- **Opt-in**: Default OFF, user must enable
- **Redaction**: Secrets automatically sanitized
- **No network**: Telemetry never leaves device

## Acceptance Criteria

-  Flags load from `config/flags.json`
-  ENV `CRYPRQ_FLAGS` overrides at runtime
-  Toggling `flags.json` updates UI behavior without rebuild
-  With telemetry OFF (default), no files are created
-  With telemetry ON, events write to JSONL; strings are redacted
-  Observability script can read counters when uncommented

## Future Enhancements

- **Mobile flags**: Extend to React Native
- **Remote flags**: Feature flag service integration (optional)
- **Telemetry aggregation**: Local dashboard for health KPIs
- **Export telemetry**: One-click export for support

