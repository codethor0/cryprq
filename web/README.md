# CrypRQ Web UI

Modern React + TypeScript web interface for CrypRQ VPN.

## Quick Start

### Development

```bash
# Install dependencies
npm install

# Start backend server (spawns Rust binary)
node server/server.mjs

# In another terminal, start frontend dev server
npm run dev
```

Open http://localhost:5173 in your browser.

### Production Build

```bash
# Build frontend
npm run build

# Start production server
node server/server.mjs
```

## Architecture

- **Frontend**: React + TypeScript + Vite
- **Backend**: Node.js Express server that spawns the CrypRQ Rust binary
- **Real-time**: EventSource for streaming logs and status updates

## Features

- Connection management (Listener/Dialer modes)
- Real-time debug console with structured logs
- Encryption status monitoring
- VPN mode toggle (system-wide routing)

## Development

The web UI communicates with the Rust backend via the Node.js server, which:
1. Spawns the CrypRQ binary with appropriate arguments
2. Streams stdout/stderr logs via EventSource
3. Provides REST API endpoints for control

See main [README.md](../README.md) for full project documentation.
