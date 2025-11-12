# Optional Add-Ons for Post-Release

These are ready-to-drop enhancements that can be implemented when needed. All are designed as 60-minute wins.

##  Feature Flag Shim

**Purpose**: Instant rollback without rebuild

**Implementation**:
- JSON/env toggles for tray behaviors & chart modules
- Files: `gui/electron/main/feature-flags.ts`, `gui/src/config/features.json`

**Example Structure**:
```json
{
  "tray": {
    "showRecentPeers": true,
    "quickConnect": true
  },
  "charts": {
    "enabled": true,
    "smoothing": true,
    "latencyAlerts": true
  }
}
```

**Usage**:
```typescript
import { featureFlags } from '@/config/features'
if (featureFlags.charts.enabled) {
  // Render charts
}
```

##  Telemetry v0

**Purpose**: Opt-in JSONL event counter (connect/disconnect/rotation, no PII)

**Implementation**:
- Event counters only (no PII)
- Files: `gui/src/services/telemetry.ts`, `mobile/src/services/telemetry.ts`
- Storage: `~/.cryprq/telemetry.jsonl` (desktop), encrypted storage (mobile)

**Event Schema**:
```json
{
  "ts": "2025-01-15T1200Z",
  "event": "session.connect",
  "success": true,
  "profile": "LOCAL",
  "latency_ms": 45
}
```

**Events**:
- `session.connect` (success, profile, latency_ms)
- `session.disconnect` (reason)
- `rotation.completed` (duration_ms)
- `session.crash` (exit_code)

**Privacy**:
- Opt-in only (Settings > Privacy > Telemetry)
- No PII (no peer IDs, endpoints, or user data)
- Local storage only (no network calls unless explicitly enabled)

##  Mini Prometheus→JSON Bridge

**Purpose**: Export health metrics to web dashboard

**Implementation**:
- Read Prometheus metrics at `http://localhost:9464/metrics`
- Convert to JSON format
- Serve at `http://localhost:9464/health.json`
- Files: `cli/cmd/health-json/main.go` or `gui/electron/main/health-bridge.ts`

**JSON Format**:
```json
{
  "timestamp": "2025-01-15T1200Z",
  "metrics": {
    "crash_free_rate": 0.995,
    "connect_success_rate": 0.99,
    "median_latency_ms": 120,
    "redaction_checks_passed": 1.0,
    "report_issue_success_rate": 0.98
  }
}
```

**Health Dashboard**:
- Simple HTML dashboard reading `/health.json`
- Real-time metrics visualization
- File: `gui/public/health-dashboard.html`

##  Post-Ship Observability Metrics

**Targets** (when telemetry v0 is enabled):

| Area | Metric | Target |
|------|--------|--------|
| Stability | Crash-free sessions | ≥ 99.5% |
| Connectivity | Connect success | ≥ 99% |
| Performance | Median latency | < 150ms |
| Security | Redaction / Audit checks | 100% pass |
| UX | Report-issue success rate | ≥ 98% |

**Implementation**:
- Add to `scripts/observability-checks.sh` once telemetry v0 is toggled on
- Query telemetry JSONL file for metrics
- Compare against targets
- Alert on threshold breaches

##  Quick Implementation Guide

### Feature Flags (30 min)
1. Create `gui/src/config/features.json`
2. Create `gui/src/utils/featureFlags.ts` loader
3. Add feature flag checks to tray and charts components
4. Document in Settings UI

### Telemetry v0 (60 min)
1. Create `gui/src/services/telemetry.ts`
2. Implement event counter (JSONL writer)
3. Add opt-in toggle in Settings > Privacy
4. Add telemetry checks to `observability-checks.sh`
5. Document privacy guarantees

### Prometheus→JSON Bridge (45 min)
1. Create bridge service (read Prometheus, write JSON)
2. Serve `/health.json` endpoint
3. Create simple HTML dashboard
4. Document in README

##  When to Implement

- **Feature Flags**: When you need instant rollback capability
- **Telemetry v0**: When you need health metrics for monitoring
- **Prometheus Bridge**: When you need a simple web dashboard

All are optional and can be added incrementally post-release.

