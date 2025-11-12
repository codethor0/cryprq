# Backend Integration Status

## ✅ Pack 1: CLI Integration (COMPLETE)

### Implemented Features

1. **Process Spawning**
   - ✅ `electron/main/session.ts` spawns CrypRQ CLI process
   - ✅ Binary discovery (dev paths, production paths, PATH fallback)
   - ✅ Command argument building (--listen, --peer, --metrics-addr)
   - ✅ Process lifecycle management

2. **JSON Event Parsing**
   - ✅ Parses structured JSON lines from CLI stdout
   - ✅ Falls back to raw log lines for non-JSON output
   - ✅ Buffers last 20 log lines for error reporting

3. **Prometheus Metrics Polling**
   - ✅ `electron/main/metrics.ts` polls `http://localhost:9464/metrics` every 2s
   - ✅ Parses metrics: bytesIn, bytesOut, rotationTimer, latency, peerId
   - ✅ Caches metrics for IPC access
   - ✅ Renderer also polls for UI updates

4. **IPC Handlers**
   - ✅ `session:start` - Start CLI process
   - ✅ `session:stop` - Stop CLI process (SIGTERM, fallback to SIGKILL)
   - ✅ `session:get` - Get current session state
   - ✅ `session:restart` - Restart session
   - ✅ `metrics:get` - Get cached metrics
   - ✅ `metrics:start` / `metrics:stop` - Control polling

5. **Event Streaming**
   - ✅ `session:event` - Structured JSON events
   - ✅ `session:log` - Raw log lines (info/error)
   - ✅ `session:ended` - Process exit notification
   - ✅ `session:error` - Process errors

## ✅ Pack 2: IPC & Crash Recovery (COMPLETE)

### Implemented Features

1. **Heartbeat Mechanism**
   - ✅ `electron/main/heartbeat.ts` monitors renderer health
   - ✅ Renderer pings every 5 seconds
   - ✅ Main process detects missed beats (3+ = stalled)
   - ✅ Sends `heartbeat:stalled` event to windows

2. **Crash Recovery**
   - ✅ Process exit detection with exit code/signal
   - ✅ Last 20 log lines included in error events
   - ✅ Session state tracking (idle/starting/running/stopping/errored)
   - ✅ Error events sent to all windows

3. **Single Instance Lock**
   - ✅ `app.requestSingleInstanceLock()` prevents multiple instances
   - ✅ Second launch focuses existing window
   - ✅ Restores minimized windows

4. **Window Management**
   - ✅ macOS: Close button minimizes to tray
   - ✅ Tray icon with context menu
   - ✅ Window restore on activate

## Integration Points

### Backend Service (`src/services/backend.ts`)

- ✅ Replaced mock with real Electron IPC calls
- ✅ Event listeners for session events
- ✅ Metrics polling integration
- ✅ Heartbeat ping integration
- ✅ Error handling and propagation

### Store Integration (`src/store/useAppStore.ts`)

- ✅ Backend events update connection status
- ✅ Logs streamed to store
- ✅ Error handling in connect/disconnect

### UI Components

- ✅ Dashboard shows real connection status
- ✅ Connect/Disconnect buttons trigger real actions
- ✅ Error messages displayed to user

## Testing Checklist

### Pack 1 Acceptance Tests

- [ ] Connect → process starts
- [ ] Disconnect → process exits within 1s
- [ ] Dashboard updates: status, rotation countdown, peer ID, latency & throughput
- [ ] Logs panel streams lines in real time (≥100 lines without UI jank)
- [ ] Invalid CLI path → actionable error
- [ ] Port busy → actionable error with "Retry/Change Port" options

### Pack 2 Acceptance Tests

- [ ] Kill child process → UI shows error and restart works
- [ ] Force-close renderer → app survives and can be reopened from tray
- [ ] Second instance launch → focuses existing window
- [ ] Heartbeat missed → stall detection works

## Next Steps

### Pack 3: Error Handling & Validations (TODO)
- Central error map (code → title, help text)
- Settings validation (ports, rotation interval, multiaddr)
- Toast notifications for transient errors
- Modal dialogs for blockers

### Pack 4: System Tray (TODO)
- Complete tray menu with Connect/Disconnect
- Last peer display
- Recent peers (5)
- Status badges (macOS/Windows)
- Minimize-to-tray preference

### Pack 5: Logs & Diagnostics (TODO)
- Log viewer with level filter
- Search functionality
- "Follow" toggle
- Log persistence to disk
- Export diagnostics (zip of logs + settings + system info)

## Known Issues

1. **Binary Path Discovery**: May need adjustment based on actual build output structure
2. **Metrics Endpoint**: Assumes CLI exposes metrics on `127.0.0.1:9464` - verify flag name
3. **JSON Event Format**: Assumes CLI outputs JSON lines - may need format negotiation
4. **Error Messages**: Currently basic - need user-friendly messages (Pack 3)

## Code Structure

```
gui/
├── electron/
│   ├── main.ts              # Window management, single instance
│   ├── main/
│   │   ├── session.ts       # CLI process spawning & management
│   │   ├── metrics.ts       # Prometheus polling
│   │   └── heartbeat.ts     # Renderer health monitoring
│   └── preload.ts           # IPC bridge (context isolation)
├── src/
│   ├── services/
│   │   └── backend.ts       # Backend service (IPC client)
│   └── store/
│       └── useAppStore.ts   # Zustand store with backend integration
```

## Usage Example

```typescript
// Start session
await backend.connect('/ip4/127.0.0.1/udp/9999/quic-v1')

// Or connect to peer
await backend.connect('12D3KooW...', undefined)

// Get status
const status = await backend.getStatus()

// Stop session
await backend.disconnect()

// Restart session
await backend.restartSession('/ip4/0.0.0.0/udp/9999/quic-v1')
```

