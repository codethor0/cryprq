# CrypRQ Desktop GUI

Cross-platform desktop application for CrypRQ VPN built with Electron, React, and TypeScript.

## Features

- **Dashboard**: Connection status, peer ID, rotation timer, throughput metrics
- **Peer Management**: Add/remove peers, connect/disconnect
- **Settings**: Configure rotation interval, logging, transport, theme
- **Dark/Light Theme**: System preference detection
- **System Tray**: Quick access menu (Windows/macOS)

## Development

### Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for CrypRQ CLI backend)

### Setup

```bash
cd gui
npm install
```

### Run Development Server

```bash
npm run dev
```

This will:
1. Start Vite dev server on http://localhost:5173
2. Launch Electron window
3. Enable hot reload

### Build

```bash
## Build for current platform
npm run build

## Build for macOS
npm run build:mac

## Build for Windows
npm run build:win
```

## Architecture

### Frontend (React)

- **Components**: Dashboard, Peers, Settings screens
- **State Management**: Zustand store (`src/store/useAppStore.ts`)
- **Routing**: React Router for navigation
- **Theming**: Light/dark theme support

### Backend Integration

The `src/services/backend.ts` module provides an interface to the CrypRQ CLI:

- Spawn/manage `cryprq` process
- Parse CLI output and metrics
- Handle connection lifecycle

### Electron Main Process

- Window management
- System tray integration
- IPC communication bridge

## Backend Integration (TODO)

Currently, the backend service uses mock data. To integrate with CrypRQ CLI:

1. **Spawn Process**: Use Node.js `child_process` to launch `cryprq`
2. **Parse Output**: Read stdout/stderr for status updates
3. **Metrics Endpoint**: Query `http://localhost:9464/metrics` (Prometheus)
4. **Configuration**: Read/write peer config files

Example integration:

```typescript
import { spawn } from 'child_process'

async connect(multiaddr?: string) {
  const args = multiaddr 
    ? ['--peer', multiaddr]
    : ['--listen', '/ip4/0.0.0.0/udp/9999/quic-v1']
  
  this.process = spawn('cryprq', args, {
    cwd: process.cwd(),
  })
  
  this.process.stdout.on('data', (data) => {
    // Parse log output for status updates
  })
}
```

## Project Structure

```
gui/
├── src/
│   ├── components/      # React components
│   │   ├── Dashboard/
│   │   ├── Peers/
│   │   ├── Settings/
│   │   └── Layout/
│   ├── services/         # Backend integration
│   ├── store/            # Zustand state
│   ├── types/            # TypeScript types
│   ├── themes/           # Theme definitions
│   └── main.tsx          # Entry point
├── electron/             # Electron main process
│   ├── main.ts
│   └── preload.ts
├── public/               # Static assets
└── package.json
```

## Wireframes

See `docs/wireframes/` for UI mockups (to be created).

## Docker Validation & Builds

For Docker-based testing and cross-platform builds:

```bash
## Run all tests in Docker
make test

## Build Linux artifacts (AppImage, .deb)
make build-linux

## Build Windows artifacts (unsigned .exe)
make build-win
```

See `docs/DOCKER_VALIDATION.md` for complete documentation.

## Next Steps

1. ✅ Basic UI structure and routing
2. ✅ State management with Zustand
3. ✅ Backend integration with CLI process
4. ✅ Error handling and validation
5. ✅ Docker validation & builds
6. ⏳ System tray enhancements (Pack 4)
7. ⏳ Logs viewer & diagnostics (Pack 5)
8. ⏳ Complete installer packages

