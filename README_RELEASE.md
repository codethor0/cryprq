# CrypRQ Release Process

## Desktop Release

### Prerequisites
- Clean working directory (commit or stash changes)
- On `main` or `master` branch
- All tests passing locally

### Steps

1. **Prepare Release**
   ```bash
   ./scripts/release.sh 1.1.0
   ```
   This will:
   - Update `gui/package.json` version
   - Verify CHANGELOG.md has entry for the version
   - Commit version bump
   - Create annotated tag

2. **Push Tag**
   ```bash
   git push origin v1.1.0
   git push origin main  # or master
   ```

3. **CI Builds Artifacts**
   - GitHub Actions will automatically:
     - Build for Linux (AppImage, .deb)
     - Build for Windows (.exe unsigned)
     - Build for macOS (.dmg)
     - Create GitHub Release with artifacts
     - Attach CHANGELOG section

4. **Smoke Tests**
   After artifacts are available, run on each platform:
   ```bash
   ./scripts/smoke-tests.sh
   ```

### Smoke Test Checklist

1. **Start → Connect → Rotate → Disconnect**
   - Launch app
   - Click Connect
   - Verify tray icon shows 'connected'
   - Wait for rotation (or simulate)
   - Verify tray icon shows 'rotating' then 'connected'
   - Click Disconnect
   - Verify tray icon shows 'disconnected'

2. **Fault Injection**
   - Start app and connect
   - Use dev hook: `window.electronAPI.devSessionSimulateExit({code: 1})`
   - Verify error modal appears
   - Verify structured logs contain error entry
   - Verify diagnostics timeline updated

3. **Diagnostics Export**
   - Export diagnostics zip
   - Verify zip < 10MB
   - Verify secrets redacted:
     ```bash
     unzip diagnostics.zip -d /tmp/diag
     grep -r "bearer\|privKey" /tmp/diag || echo "No secrets found "
     ```
   - Verify `session-summary.json` present
   - Verify `metrics-snapshot.json` present

## Mobile Release

### Prerequisites
- React Native project initialized
- Dependencies installed (`npm install` in `mobile/`)
- Android SDK and Xcode configured

### Steps

1. **Tag Mobile Release**
   ```bash
   git tag mobile-v1.0.0
   git push origin mobile-v1.0.0
   ```

2. **CI Builds Artifacts**
   - GitHub Actions will automatically:
     - Build Android AAB
     - Build iOS archive/IPA (if signing configured)
     - Upload artifacts

3. **Fastlane Builds (Local)**
   ```bash
   cd mobile
   
   # Android
   fastlane android build
   fastlane android beta
   
   # iOS
   fastlane ios build
   fastlane ios beta  # Requires App Store Connect API key
   ```

### Mobile QA

Run the checklist in `mobile/docs/QA.md`:
- First-run consent flow
- Connect → Rotate → Disconnect
- Endpoint profiles (LOCAL/LAN/REMOTE)
- Peer management
- Background notifications
- Logs modal
- Developer screen
- Accessibility
- Security checks

## Version Numbering

- **Desktop**: Semantic versioning (e.g., `1.1.0`)
- **Mobile**: Semantic versioning (e.g., `1.0.0`)
- **Tags**: `v1.1.0` for desktop, `mobile-v1.0.0` for mobile

## Artifacts

### Desktop
- Linux: `.AppImage`, `.deb`
- Windows: `.exe` (unsigned)
- macOS: `.dmg`

### Mobile
- Android: `.aab` (release), `.apk` (beta)
- iOS: `.ipa` (if signing configured)

All artifacts are attached to GitHub Releases automatically.

