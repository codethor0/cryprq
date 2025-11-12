# Docker Validation & Builds

## Quick Start

### Run Tests
```bash
cd gui
make test
```

### Build Linux Artifacts
```bash
cd gui
make build-linux
## Artifacts in gui/dist/
```

### Build Windows Artifacts (Unsigned)
```bash
cd gui
make build-win
## Artifacts in gui/dist/
```

## What's Included

- **Test Image**: Playwright + Xvfb for headless testing
- **Linux Builder**: Produces AppImage and .deb packages
- **Windows Builder**: Produces unsigned NSIS installer via Wine
- **Fake Backend**: Simulates CrypRQ CLI for E2E testing
- **CI/CD**: GitHub Actions workflow for automated builds

See `docs/DOCKER_VALIDATION.md` for complete documentation.

