# Quick Start Guide

## Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for CrypRQ CLI backend)

## Installation

```bash
cd gui
npm install
```

## Development

Start the development server:

```bash
npm run dev
```

This will:
1. Start Vite dev server on http://localhost:5173
2. Launch Electron window
3. Enable hot reload for React components

## Building

### Development Build

```bash
npm run build:vite      # Build React app
npm run build:electron  # Compile Electron main process
```

### Production Build

```bash
# Build for current platform
npm run build

# Build for macOS
npm run build:mac

# Build for Windows
npm run build:win
```

## Project Structure

```
gui/
├── src/
│   ├── components/      # React components
│   │   ├── Dashboard/   # Dashboard screen
│   │   ├── Peers/       # Peer management
│   │   ├── Settings/    # Settings screen
│   │   └── Layout/      # Sidebar, MainLayout
│   ├── services/        # Backend integration
│   ├── store/           # Zustand state management
│   ├── types/           # TypeScript types
│   ├── themes/          # Theme definitions
│   └── main.tsx         # React entry point
├── electron/            # Electron main process
│   ├── main.ts          # Window management, tray
│   └── preload.ts       # IPC bridge
├── public/              # Static assets
└── docs/                # Documentation
```

## Features

### ✅ Implemented

- Dashboard with connection status
- Peer management (add/remove)
- Settings screen
- Dark/light theme support
- System tray integration
- Sidebar navigation

### ⏳ TODO

- Backend integration with CrypRQ CLI
- Metrics polling from Prometheus endpoint
- Configuration file persistence
- Error handling and validation
- Icon assets for system tray

## Next Steps

1. Install dependencies: `npm install`
2. Run development server: `npm run dev`
3. See `docs/backend-integration.md` for CLI integration guide
4. See `docs/wireframes.md` for UI design specifications

