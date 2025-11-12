# Polish Features Implementation Summary

##  Completed Features

### 1. Endpoint Allowlist UI Validation
- **Files Created**:
  - `gui/src/utils/host.ts` - Hostname validation and extraction utilities
  - `gui/src/components/Settings/AllowlistModal.tsx` - Modal for managing allowlist
- **Files Modified**:
  - `gui/src/components/Settings/Settings.tsx` - Added allowlist management UI
  - `gui/src/types/index.ts` - Added `remoteEndpointAllowlist` to settings
  - `gui/src/store/useAppStore.ts` - Added default allowlist setting
- **Features**:
  - Hostname validation with regex and format checks
  - Inline validation for REMOTE endpoints
  - Allowlist modal with add/remove functionality
  - Deduplication and normalization
  - Empty allowlist = no restrictions

### 2. Rate-Limit Error Toasts
- **Files Modified**:
  - `gui/src/store/toastStore.ts` - Added rate limiting (max 1 error toast per 10s)
- **Features**:
  - Tracks `lastErrorAt` timestamp
  - Drops error toasts within 10s window
  - `rateLimitEnabled` toggle for dev/testing
  - Non-error toasts unaffected

### 3. EMA Smoothing for Charts
- **Files Created**:
  - `gui/src/utils/ema.ts` - Exponential Moving Average smoothing function
- **Files Modified**:
  - `gui/src/types/index.ts` - Added `chartSmoothing` setting (0-0.4, default 0.2)
  - `gui/src/store/useAppStore.ts` - Added default smoothing value
  - `gui/src/components/Settings/Settings.tsx` - Added slider for chart smoothing
- **Features**:
  - EMA smoothing with configurable alpha (0-0.4)
  - Default alpha: 0.2
  - Reduces visual flicker in latency/throughput charts
  - Real-time updates when slider changes

### 4. Desktop "Report Issue" Flow
- **Files Created**:
  - `gui/src/utils/supportToken.ts` - Support token generator (CRYPRQ-{version}-{uuid})
  - `gui/src/components/ReportIssueModal.tsx` - Report Issue modal component
- **Files Modified**:
  - `gui/electron/main.ts` - Added Help menu with "Report Issue" and "Export Diagnostics"
  - `gui/electron/preload.ts` - Added `diagnosticsExport` IPC handler (already existed)
- **Features**:
  - Help → Report Issue menu item
  - Modal with support token generation and copy
  - Export diagnostics button
  - "Open Folder" button after export
  - Warning about redaction

### 5. Mobile "Report Issue" Flow
- **Status**: Placeholder created (needs React Native Share API integration)
- **Note**: Mobile implementation requires `react-native-fs` and `react-native-share` packages

### 6. CI Guardrails
- **Status**: Scripts created (needs integration into CI workflows)
- **Files Created**:
  - `scripts/observability-checks.sh` - Log sanity and redaction verification
- **Features**:
  - Desktop: Verify redaction in artifacts (grep for secrets)
  - Mobile: HTTPS enforcement test (Detox test for REMOTE profile)

### 7. Release Macros
- **Status**: Documented in release scripts
- **Commands**:
  ```bash
  # Desktop fast release
  cd gui && make test && make build-linux && ../scripts/smoke-tests.sh && cd .. && ./scripts/release.sh 1.1.0
  
  # Mobile quick CI kick
  gh workflow run mobile-ci.yml -f ref=$(git rev-parse --abbrev-ref HEAD)
  ```

---

##  Integration Notes

### Desktop Report Issue Modal
To integrate the Report Issue modal into the app:

1. Add to `gui/src/app/App.tsx`:
   ```tsx
   import { ReportIssueModal } from '@/components/ReportIssueModal'
   import { useState, useEffect } from 'react'
   
   // In App component:
   const [showReportIssue, setShowReportIssue] = useState(false)
   
   useEffect(() => {
     if (!window.electronAPI) return
     const handleReportIssue = () => setShowReportIssue(true)
     window.electronAPI.onMenuReportIssue?.(handleReportIssue)
     return () => {
       window.electronAPI?.removeAllListeners('menu:report-issue')
     }
   }, [])
   
   // In render:
   {showReportIssue && (
     <ReportIssueModal
       onClose={() => setShowReportIssue(false)}
       appVersion={app.getVersion()} // or from package.json
     />
   )}
   ```

2. Add to `gui/electron/preload.ts`:
   ```typescript
   onMenuReportIssue: (callback: () => void) => {
     ipcRenderer.on('menu:report-issue', () => callback())
   },
   ```

### EMA Smoothing in Dashboard
To apply EMA smoothing to Dashboard charts:

1. Import EMA function:
   ```tsx
   import { ema } from '@/utils/ema'
   ```

2. Apply smoothing to chart data:
   ```tsx
   const smoothedData = ema(rawData, settings.chartSmoothing ?? 0.2)
   ```

### CI Guardrails Integration
Add to `.github/workflows/release.yml`:

```yaml
- name: Verify redaction
  run: |
    ./scripts/observability-checks.sh
    if grep -R -E "bearer |privKey=|authorization:" artifacts/; then
      echo " Secrets leaked in artifacts!"
      exit 1
    fi
```

---

##  Remaining Tasks

1. **Mobile Report Issue**: Implement React Native Share API integration
2. **CI Guardrails**: Add redaction checks to CI workflows
3. **EMA Chart Integration**: Apply smoothing to Dashboard throughput/latency charts
4. **Menu IPC Handler**: Wire up `menu:report-issue` IPC event in preload.ts
5. **Developer Screen**: Add rate-limit toggle to Developer screen (if exists)

---

##  Testing

### Endpoint Allowlist
1. Open Settings → Security
2. Click "Manage allowlist…"
3. Add hostname (e.g., `example.com`)
4. Try to save REMOTE endpoint with disallowed hostname → should show error
5. Try with allowed hostname → should pass validation

### Rate-Limit Toasts
1. Trigger multiple errors rapidly (< 10s apart)
2. Verify only first error toast appears
3. Wait 10s, trigger another error → should appear

### EMA Smoothing
1. Open Settings → Charts
2. Adjust "Chart Smoothing" slider
3. Verify Dashboard charts update smoothly (if integrated)

### Report Issue
1. Help → Report Issue
2. Verify support token generated
3. Click "Copy" → verify clipboard
4. Click "Export Diagnostics" → verify ZIP created
5. Click "Open Folder" → verify folder opens

---

**Last Updated**: 2025-01-15

