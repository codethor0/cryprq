# Docker Validation & Cross-Platform Builds

This document describes the Docker-based validation and build system for the CrypRQ GUI.

## Overview

The Docker setup enables:
- **Consistent testing** across development environments
- **Cross-platform builds** (Linux, Windows, macOS)
- **E2E testing** with a fake CrypRQ backend
- **CI/CD integration** via GitHub Actions

## Prerequisites

- Docker Desktop installed and running
- Docker Compose v2+ (included with Docker Desktop)

## Quick Start

### Run Tests Locally

```bash
cd gui
make test
# or
docker compose -f docker-compose.yml up --build --abort-on-container-exit --exit-code-from tests
```

### Build Linux Artifacts

```bash
cd gui
make build-linux
# or
docker compose -f docker-compose.yml --profile linux-build up --build
```

Artifacts will be in `gui/dist/`:
- `CrypRQ-*.AppImage` - AppImage bundle
- `cryprq-gui_*.deb` - Debian package

### Build Windows Artifacts (Unsigned)

```bash
cd gui
make build-win
# or
docker compose -f docker-compose.yml --profile win-build up --build
```

Artifacts will be in `gui/dist/`:
- `CrypRQ Setup *.exe` - NSIS installer (unsigned)

## Docker Images

### Test Image (`Dockerfile.test`)

Based on `mcr.microsoft.com/playwright:v1.47.0-jammy`:
- Includes Chromium and fonts
- Xvfb for headless display
- Node.js and npm
- Runs lint, typecheck, unit tests, and E2E tests

### Builder Images

**Linux Builder** (`Dockerfile.builder`):
- Based on `electronuserland/builder:18-jammy`
- Includes electron-builder and dependencies
- Produces AppImage and .deb packages

**Windows Builder** (`Dockerfile.win`):
- Based on `electronuserland/builder:wine-mono`
- Uses Wine to build Windows NSIS installers
- Produces unsigned .exe files

## Fake CrypRQ Backend

The `fake-cryprq` service simulates the CrypRQ CLI for E2E testing:

- **Metrics endpoint**: `http://localhost:9464/metrics`
  - Returns Prometheus-format metrics
  - Updates bytes in/out, latency, rotation timer

- **JSONL events**: Emits to stdout every 1-2 seconds
  - `{"type":"status","status":"connected","peerId":"..."}`
  - `{"type":"rotation","nextInSeconds":300}`
  - `{"type":"metric","latencyMs":25,"bytesIn":12345,"bytesOut":6789}`

## E2E Test Scenarios

### Connection Flow
- Dashboard shows disconnected state initially
- Connect button triggers connection
- Rotation timer visible when connected

### Error Handling
- Metrics timeout shows non-blocking toast
- Port in use shows blocking modal with CTA
- CLI missing shows modal with file picker option

### Crash Recovery
- Session ended shows restart option
- Restart session works after crash

## CI/CD Workflow

### On Pull Requests
- Runs `tests_linux` job only
- Validates lint, typecheck, unit tests, E2E tests
- Uploads test results as artifacts

### On Version Tags (`v*`)
- Runs all test jobs
- Builds Linux artifacts (AppImage, .deb)
- Builds Windows artifacts (unsigned .exe)
- Builds macOS artifacts (.dmg) on macOS runner
- Uploads all artifacts to GitHub Actions

## NPM Scripts

```bash
# Development
npm run dev              # Start dev server
npm run dev:xvfb         # Start dev server with Xvfb (headless)

# Testing
npm run lint             # Run ESLint
npm run typecheck        # TypeScript type checking
npm run test:unit        # Run unit tests (Vitest)
npm run test:playwright  # Run E2E tests (Playwright)
npm run test:ci          # Run all tests (lint + typecheck + unit + e2e)
npm run e2e:headless     # Run E2E tests with Xvfb

# Building
npm run build:linux      # Build Linux artifacts
npm run build:win-unsigned # Build unsigned Windows installer
npm run build:mac        # Build macOS DMG (macOS only)
```

## Known Limitations

### macOS Builds
- **Cannot be built in Docker**: macOS .dmg requires macOS runner
- **Code signing**: Requires macOS Developer ID certificate
- **Notarization**: Requires App Store Connect API key
- Currently builds unsigned DMG on macOS GitHub Actions runner

### Windows Builds
- **Unsigned**: Wine-based builds produce unsigned installers
- **Signing**: Requires Windows code signing certificate
- **Post-build**: Signing must be done on Windows or macOS

### Linux Builds
- **AppImage**: Works on most Linux distributions
- **Debian package**: Requires Debian/Ubuntu-based system
- **RPM**: Not currently built (can be added)

## Troubleshooting

### Tests Fail in Docker
- Ensure Docker Desktop is running
- Check that ports 9464 (fake-cryprq) and 5173 (dev server) are available
- Verify Xvfb is working: `docker compose exec tests xvfb-run echo "test"`

### Build Artifacts Missing
- Check `gui/dist/` directory exists
- Verify volume mount in docker-compose.yml
- Check build logs for errors

### Playwright Tests Timeout
- Increase timeout in `playwright.config.ts`
- Check that fake-cryprq service is healthy
- Verify network connectivity between containers

## Future Enhancements

1. **Real CLI Integration**: Replace fake-cryprq with actual CrypRQ CLI in container
2. **Screenshot Testing**: Enable Playwright screenshots on failure
3. **Performance Testing**: Add Lighthouse CI for performance benchmarks
4. **Visual Regression**: Add Percy or Chromatic for visual testing
5. **Windows Signing**: Add Windows code signing step in CI
6. **macOS Notarization**: Add notarization step for macOS builds

## References

- [Playwright Documentation](https://playwright.dev)
- [Electron Builder](https://www.electron.build/)
- [Docker Compose](https://docs.docker.com/compose/)
- [GitHub Actions](https://docs.github.com/en/actions)

