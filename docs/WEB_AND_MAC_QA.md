# Web Cross-Browser QA & macOS App Packaging

## Overview

This guide covers:
- **Web Cross-Browser QA**: Playwright tests across Chromium/Firefox/WebKit
- **macOS App Packaging**: Signed .app bundle for Apple Silicon
- **CI Integration**: Automated testing and preview deployment

## Web Cross-Browser QA

### Local Testing

```bash
# Install Playwright browsers
npm run test:web:install

# Run tests (assumes bridge + site already running)
npm run test:web
```

### Manual Setup

1. **Start bridge server**:
```bash
cd web/server
BRIDGE_PORT=8787 CRYPRQ_BIN=../../target/aarch64-apple-darwin/release/cryprq node server.mjs
```

2. **Start web client** (in another terminal):
```bash
cd web
npm run dev
```

3. **Run Playwright**:
```bash
npm run test:web
```

### Test Coverage

- **Chromium**: Desktop Chrome
- **Firefox**: Desktop Firefox
- **WebKit**: Desktop Safari

Tests verify:
- Listener starts via bridge
- Dialer connects to listener
- Events stream shows handshake/peer/rotation events

## macOS App Packaging

### Basic Packaging

```bash
make mac-app
# or
bash scripts/package-mac-app.sh
```

Output: `artifacts/macos-app/CrypRQ.app`

### With Signing

```bash
SIGNING_IDENTITY="Developer ID Application: Your Name" make mac-app
```

### With Notarization

```bash
NOTARY_APPLE_ID="your@email.com" \
NOTARY_TEAM_ID="TEAM123" \
NOTARY_API_KEY_ID="KEY123" \
NOTARY_API_KEY_ISSUER="issuer-id" \
NOTARY_PROFILE="notary-profile" \
make mac-app
```

### Testing the App

```bash
bash scripts/test-mac-app.sh
```

This verifies:
- App launches successfully
- Binary executes correctly
- Listener + dialer connection works

## CI Integration

### Web Preview Workflow

The `.github/workflows/web-preview.yml` workflow:
- Builds macOS binary (for bridge server)
- Builds web client
- Starts bridge server
- Serves built site
- Runs Playwright tests across all browsers
- Uploads artifacts
- Optionally deploys to GitHub Pages

### Triggering

Runs automatically on:
- Pull requests touching `web/**`
- Pushes to `main` branch touching `web/**`

### Manual Trigger

```bash
gh workflow run web-preview.yml
```

## Configuration

### Environment Variables

**Web Tests:**
- `BRIDGE_URL` - Bridge server URL (default: `http://localhost:8787`)
- `SITE_URL` - Web client URL (default: `http://localhost:5173`)
- `CRYPRQ_PORT` - CrypRQ UDP port (default: 9999)

**macOS App:**
- `APP_NAME` - App name (default: `CrypRQ`)
- `BUNDLE_ID` - Bundle identifier (default: `io.cryrpq.app`)
- `ICON_ICNS` - Icon file path (default: `branding/AppIcon.icns`)
- `SIGNING_IDENTITY` - Code signing identity (optional)
- `NOTARY_APPLE_ID` - Apple ID for notarization (optional)
- `NOTARY_TEAM_ID` - Team ID for notarization (optional)
- `NOTARY_API_KEY_ID` - API key ID (optional)
- `NOTARY_API_KEY_ISSUER` - API key issuer (optional)
- `NOTARY_PROFILE` - Keychain profile name (optional)

## Artifacts

### Web QA

- `artifacts/web-qa/bridge.log` - Bridge server logs
- `artifacts/web-qa/http.log` - HTTP server logs
- `playwright-report/` - Playwright HTML report

### macOS App

- `artifacts/macos-app/CrypRQ.app` - Packaged app bundle
- `artifacts/macos-app/CrypRQ.app.zip` - Notarization zip (if notarized)

## Troubleshooting

### Playwright Tests Fail

1. Check bridge server is running: `curl http://localhost:8787/events`
2. Verify web client is accessible: `curl http://localhost:5173`
3. Check Playwright report: `npx playwright show-report`

### macOS App Won't Launch

1. Verify binary exists: `ls -la artifacts/macos-app/CrypRQ.app/Contents/MacOS/cryprq`
2. Check permissions: `chmod +x artifacts/macos-app/CrypRQ.app/Contents/MacOS/cryprq`
3. Test binary directly: `artifacts/macos-app/CrypRQ.app/Contents/MacOS/cryprq --version`

### Notarization Fails

1. Verify API key is valid
2. Check keychain profile exists: `xcrun notarytool history --keychain-profile "$NOTARY_PROFILE"`
3. Review notarization logs in Xcode Organizer

## See Also

- `tests/web.spec.ts` - Playwright test suite
- `scripts/package-mac-app.sh` - macOS app packager
- `scripts/test-mac-app.sh` - macOS app tester
- `.github/workflows/web-preview.yml` - CI workflow
- `playwright.config.ts` - Playwright configuration

