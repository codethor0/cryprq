# Continuous Web QA + macOS Packaging

## Overview

The `scripts/qa-web-and-mac.sh` script runs on every change to validate:
- **Web Client**: Cross-browser Playwright tests (Chromium/Firefox/WebKit)
- **macOS App**: Packaging and smoke testing

## Quality Gates

### Blocking Failures

The script **blocks and fails** on:
1. **Playwright test failures** - Any browser fails
2. **Missing handshake/rotation events** - Bridge logs don't show expected events
3. **macOS app launch failure** - App won't start
4. **macOS app connection failure** - Listener + dialer don't connect

### Success Criteria

All gates must pass:
- ✅ Playwright tests pass on all three browsers
- ✅ Bridge logs show handshake/peer/rotation events
- ✅ macOS app launches successfully
- ✅ macOS app passes listener + dialer connection test

## Usage

### Standalone

```bash
bash scripts/qa-web-and-mac.sh
```

### Via Makefile

```bash
make qa-web-mac
```

### Integrated with Dev Watcher

The dev-watcher automatically runs this gate when `web/` directory exists.

## What It Does

1. **Builds macOS binary** (for bridge server)
2. **Builds web client** (`web/dist`)
3. **Starts bridge server** (port 8787)
4. **Serves built site** (port 5173)
5. **Runs Playwright tests** (Chromium/Firefox/WebKit)
6. **Verifies bridge logs** (handshake/peer/rotation events)
7. **Packages macOS app** (`artifacts/macos-app/CrypRQ.app`)
8. **Tests macOS app** (launch + connection)

## Artifacts

### Web QA

- `artifacts/web-qa/web_build.txt` - Web client build log
- `artifacts/web-qa/mac_build.txt` - macOS binary build log
- `artifacts/web-qa/bridge.log` - Bridge server logs
- `artifacts/web-qa/http.log` - HTTP server logs
- `artifacts/web-qa/playwright.txt` - Playwright test output
- `artifacts/web-qa/summary.md` - Quality gates summary
- `playwright-report/` - HTML test report

### macOS App

- `artifacts/macos-app/CrypRQ.app` - Packaged app bundle
- `artifacts/macos-app/package.log` - Packaging log
- `artifacts/macos-app/smoke_test.log` - Smoke test log
- `artifacts/macos-app/CrypRQ.app.zip` - Notarization zip (if notarized)

## CI Integration

The `.github/workflows/qa-web-mac.yml` workflow:
- Runs on macOS runners (required for Apple Silicon builds)
- Runs Playwright tests
- Packages macOS app
- Uploads all artifacts
- Fails if any quality gate fails

## Configuration

Environment variables:
- `PORT_WEB` - HTTP server port (default: 5173)
- `PORT_BRIDGE` - Bridge server port (default: 8787)
- `CRYPRQ_PORT` - CrypRQ UDP port (default: 9999)
- `ROTATE_SECS` - Accelerated rotation for testing (default: 10)

## Troubleshooting

### Playwright Tests Fail

1. Check bridge server: `curl http://localhost:8787/events`
2. Check HTTP server: `curl http://localhost:5173`
3. Review Playwright report: `npx playwright show-report`
4. Check logs: `artifacts/web-qa/playwright.txt`

### Bridge Logs Missing Events

1. Verify bridge server started: `ps aux | grep server.mjs`
2. Check bridge logs: `cat artifacts/web-qa/bridge.log`
3. Ensure CrypRQ binary is executable: `ls -la target/aarch64-apple-darwin/release/cryprq`

### macOS App Fails

1. Check app exists: `ls -la artifacts/macos-app/CrypRQ.app`
2. Test binary directly: `artifacts/macos-app/CrypRQ.app/Contents/MacOS/cryprq --version`
3. Review smoke test log: `cat artifacts/macos-app/smoke_test.log`

## Integration with Dev Watcher

The dev-watcher (`scripts/dev-watch.sh`) automatically includes this gate when:
- `web/` directory exists
- `scripts/qa-web-and-mac.sh` is executable

This ensures web QA and macOS packaging run on every change.

## See Also

- `scripts/qa-web-and-mac.sh` - Main QA script
- `scripts/package-mac-app.sh` - macOS app packager
- `scripts/test-mac-app.sh` - macOS app tester
- `tests/web.spec.ts` - Playwright test suite
- `.github/workflows/qa-web-mac.yml` - CI workflow

