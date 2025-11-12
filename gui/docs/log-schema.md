# CrypRQ GUI Log Schema v1

## Format: JSONL (JSON Lines)

One JSON object per line, newline-delimited.

## Schema v1

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info"|"warn"|"error"|"debug",
  "src": "cli"|"ipc"|"metrics"|"app",
  "event": "session.state"|"session.error"|"cli.raw"|"cli.json"|"metrics.tick"|"peer.action"|"rotation.started"|"rotation.completed"|"rotation.scheduled"|"log.invalid",
  "msg": "human summary (short)",
  "data": { ... }
}
```

### Required Fields

- `v` (number): Schema version (currently 1)
- `ts` (string): ISO 8601 timestamp with milliseconds
- `lvl` (string): Log level: `info`, `warn`, `error`, `debug`
- `src` (string): Source: `cli`, `ipc`, `metrics`, `app`
- `event` (string): Event type (see below)
- `msg` (string): Human-readable summary (max 200 chars)

### Optional Fields

- `data` (object): Event-specific data (redacted of secrets)

## Event Types

### `session.state`
Session state transition.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "app",
  "event": "session.state",
  "msg": "state=connecting",
  "data": {
    "state": "connecting"|"running"|"stopping"|"idle"|"errored",
    "peerId": "Qm...",
    "exitCode": 0
  }
}
```

### `session.error`
Session error.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "error",
  "src": "app",
  "event": "session.error",
  "msg": "Process error: EADDRINUSE",
  "data": {
    "error": "Address already in use",
    "code": "PORT_IN_USE"
  }
}
```

### `cli.raw`
Raw CLI output (stdout/stderr).

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info"|"error",
  "src": "cli",
  "event": "cli.raw",
  "msg": "INFO: Starting listener on /ip4/0.0.0.0/udp/9999/quic-v1",
  "data": {}
}
```

### `cli.json`
Parsed JSON event from CLI.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "cli",
  "event": "cli.json",
  "msg": "Connected to peer",
  "data": {
    "type": "connected",
    "peerId": "Qm...",
    "timestamp": "2025-11-11T18:43:21.345Z"
  }
}
```

### `metrics.tick`
Metrics update.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "metrics",
  "event": "metrics.tick",
  "msg": "Metrics updated",
  "data": {
    "bytesIn": 1024,
    "bytesOut": 2048,
    "latencyMs": 25,
    "rotationTimer": 300,
    "peerId": "Qm..."
  }
}
```

### `rotation.started`
Key rotation started.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "cli",
  "event": "rotation.started",
  "msg": "Key rotation started",
  "data": {}
}
```

### `rotation.completed`
Key rotation completed.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "cli",
  "event": "rotation.completed",
  "msg": "Key rotation completed",
  "data": {
    "nextInSeconds": 300
  }
}
```

### `rotation.scheduled`
Rotation timer update.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "metrics",
  "event": "rotation.scheduled",
  "msg": "Rotation scheduled",
  "data": {
    "nextInSeconds": 300
  }
}
```

### `peer.action`
Peer management action.

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "info",
  "src": "app",
  "event": "peer.action",
  "msg": "Peer added",
  "data": {
    "action": "add"|"remove"|"connect"|"disconnect",
    "peerId": "Qm..."
  }
}
```

### `log.invalid`
Invalid log entry (validation failure).

```json
{
  "v": 1,
  "ts": "2025-11-11T18:43:21.345Z",
  "lvl": "error",
  "src": "app",
  "event": "log.invalid",
  "msg": "Invalid log entry",
  "data": {
    "reason": "Missing required field: ts"
  }
}
```

## Redaction Rules

All `msg` and `data` fields are redacted before writing:

- `bearer\s+\S+` → `bearer ***REDACTED***`
- `token(=|:)\S+` → `token=***REDACTED***`
- `privKey\S*` → `privKey***REDACTED***`
- `authorization\s*:\s*\S+` → `authorization: ***REDACTED***`

Redaction is applied recursively to nested objects.

## File Naming

- Format: `cryprq-YYYY-MM-DD.log`
- Location: `~/.cryprq/logs/`
- Rotation: When file exceeds 10MB, renamed with timestamp suffix
- Retention: Last 7 days

## Validation

At append time, validate:
- `v` is number
- `ts` is valid ISO 8601
- `lvl` is one of: `info`, `warn`, `error`, `debug`
- `src` is one of: `cli`, `ipc`, `metrics`, `app`
- `event` is non-empty string
- `msg` is non-empty string (max 200 chars)

On validation failure, wrap as `log.invalid` entry.

