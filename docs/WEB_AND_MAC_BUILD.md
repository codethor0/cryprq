# Web Client and macOS Apple Silicon Build Guide

## Overview

This guide covers building and testing:
- **Web Client**: Minimal Vite + React + TypeScript client with debug console
- **macOS Apple Silicon**: Native aarch64-apple-darwin binary
- **End-to-End Smoke Tests**: Connectivity and rotation verification

## Quick Start

```bash
bash scripts/build-web-and-mac.sh
```

## What Gets Built

### Web Client (`web/`)

- **Client**: Vite + React + TypeScript SPA
- **Bridge Server**: Express.js server (`web/server/`) that spawns `cryprq` binary
- **Debug Console**: Fixed bottom panel showing sanitized logs
- **Endpoints**:
  - `POST /connect` - Start listener or dialer
  - `GET /events` - SSE stream of events

### macOS Binary

- **Target**: `aarch64-apple-darwin`
- **Output**: `target/aarch64-apple-darwin/release/cryprq`
- **Launch Script**: `macos/launch.sh` (if no GUI packager)

## Architecture

```

  Web Client  (Vite React TS)
  (Browser)  

        HTTP/SSE

  Bridge Server   (Express.js)
  (Node.js)      

        spawns

  cryprq binary   (macOS ARM64)

```

## Smoke Tests

### 1. macOS Local Test

- Builds `aarch64-apple-darwin` binary
- Runs listener on UDP/9999
- Connects dialer to listener
- Verifies handshake/connection

### 2. Web Client Test

- Builds web client
- Starts bridge server
- Starts Vite dev server
- Tests `/connect` endpoints
- Verifies events stream

### 3. Docker QA (Optional)

- Builds Docker image
- Runs listener + dialer
- Verifies QUIC handshake
- Checks rotation/zeroization logs

## Usage

### Web Client

1. **Start bridge server**:
```bash
cd web/server
BRIDGE_PORT=8787 CRYPRQ_BIN=/path/to/cryprq node server.mjs
```

2. **Start web client** (in another terminal):
```bash
cd web
npm run dev
```

3. **Open browser**: `http://localhost:5173`

4. **Test**:
   - Open two tabs/windows
   - Tab 1: Select "listener", click "Connect"
   - Tab 2: Select "dialer", set peer to listener address, click "Connect"
   - Watch debug console for events

### macOS Binary

**Using launch script**:
```bash
# Listener
bash macos/launch.sh listener 9999

# Dialer (in another terminal)
bash macos/launch.sh dialer 9999 /ip4/127.0.0.1/udp/9999/quic-v1
```

**Direct binary**:
```bash
# Listener
target/aarch64-apple-darwin/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

# Dialer
target/aarch64-apple-darwin/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1
```

## Debug Console

The web client includes a fixed bottom debug console that shows:
- **Status events**: Connection state changes
- **Rotation events**: Key rotation timers
- **Peer events**: Handshake/connection events
- **Error events**: Failures and warnings

All logs are sanitized (no secrets or keys displayed).

## Artifacts

Build outputs are saved to:
- `artifacts/web-test/` - Web client build logs and test results
- `artifacts/macos-test/` - macOS build logs and smoke test results

## Configuration

Environment variables:
- `PORT_WEB` - Vite dev server port (default: 5173)
- `PORT_BRIDGE` - Bridge server port (default: 8787)
- `CRYPRQ_PORT` - CrypRQ UDP port (default: 9999)
- `ROTATE_SECS` - Accelerated rotation for testing (default: 10)

## Troubleshooting

### Web client won't connect

1. Check bridge server is running: `curl http://localhost:8787/events`
2. Verify `CRYPRQ_BIN` points to correct binary
3. Check browser console for errors
4. Verify Vite proxy configuration

### macOS binary not found

```bash
# Build it manually
rustup target add aarch64-apple-darwin
cargo build --release -p cryprq --target aarch64-apple-darwin
```

### Docker QA fails

- Ensure Docker is running
- Check port 9999 is not in use
- Verify Docker image builds successfully

## Integration with GUI

If you have an Electron/Tauri GUI under `gui/`, you can:
1. Build macOS app bundle instead of using `macos/launch.sh`
2. Point bridge server to GUI binary
3. Use GUI's existing IPC instead of bridge server

## See Also

- `scripts/build-web-and-mac.sh` - Main build script
- `macos/launch.sh` - macOS launch helper
- `web/server/server.mjs` - Bridge server implementation
- `web/src/DebugConsole.tsx` - Debug console component

