# Release Summary: Desktop 1.1.0 & Mobile Bootstrap

##  Completed Tasks

### Desktop 1.1.0 Release

1. **Preflight & Release Infrastructure**
   -  Makefile targets (`test`, `build-linux`, `build-mac`, `build-win`)
   -  Release script (`scripts/release.sh`)
   -  Release workflow (`.github/workflows/release.yml`)
   -  Smoke test script (`scripts/smoke-tests.sh`)
   -  Release documentation (`README_RELEASE.md`)

2. **Production Hardening**
   -  Kill-switch: "Disconnect on app quit" toggle (default ON)
   -  Endpoint allowlist: `remoteEndpointAllowlist` setting (empty = no restrictions)
   -  Settings UI updated with kill-switch checkbox

3. **Store Readiness**
   -  Store readiness checklists (`docs/STORE_READINESS.md`)
   -  Post-release monitoring plan (`docs/POST_RELEASE_MONITORING.md`)

### Mobile Bootstrap

1. **Core Infrastructure**
   -  React Native app structure
   -  Screens: Dashboard, Peers, Settings, Logs, Privacy, About, FirstRun, Developer
   -  Services: Backend, Security, Notifications, Crash Reporting
   -  CI workflows (`.github/workflows/mobile-ci.yml`)
   -  Fastlane lanes (`mobile/fastlane/`)

2. **Security & Store Readiness**
   -  Encrypted storage (MMKV)
   -  HTTPS enforcement (REMOTE profile)
   -  Log redaction
   -  First-run consent flow
   -  Privacy controls

##  Ready for Release

### Desktop Release Steps

```bash
## 1. Preflight (local)
cd gui
make test              # Run dockerized tests
make build-linux       # Build Linux artifacts locally
../scripts/smoke-tests.sh  # Quick smoke tests

## 2. Cut release
./scripts/release.sh 1.1.0
git push origin v1.1.0
git push origin main

## 3. CI will automatically:
##    - Build artifacts for Linux/Windows/macOS
##    - Create GitHub Release
##    - Attach artifacts and CHANGELOG

## 4. Post-release spot checks
##    - Tray: connect → rotate → disconnect
##    - Fault inject: devsimulateExit
##    - Diagnostics export: verify <10MB, no secrets
```

### Mobile Bootstrap Steps

```bash
## 1. Initialize & build
cd mobile
npm install

## Android
cd android && ./gradlew assembleDebug

## iOS (macOS only)
xcodebuild -workspace ios/CrypRQ.xcworkspace -scheme CrypRQ -configuration Debug -sdk iphonesimulator

## 2. Start fake backend
docker compose up -d fake-cryprq

## 3. Run E2E tests
## Android
npx detox build -c android.emu.debug
npx detox test -c android.emu.debug --headless --record-logs all

## iOS
npx detox build -c ios.sim.debug
npx detox test -c ios.sim.debug --record-logs all
```

##  Store Submission Checklists

See `docs/STORE_READINESS.md` for:
- Google Play Store checklist
- Apple App Store checklist
- Common requirements
- Pre-submission checklist

##  Monitoring

See `docs/POST_RELEASE_MONITORING.md` for:
- Desktop diagnostics verification
- Mobile crash reporting checks
- Issue reporting flows
- Weekly review checklist

##  Production Hardening Features

### Kill-Switch
- **Location**: Settings > Window Behavior > "Disconnect on app quit"
- **Default**: ON
- **Behavior**: When enabled, automatically disconnects active session on app quit

### Endpoint Allowlist
- **Location**: Settings (future: Security section)
- **Default**: Empty (no restrictions)
- **Behavior**: For REMOTE profile, validate endpoints against allowlist

### Rate Limiting (Future)
- Error toasts: Cap to 1/10s
- Rotation toasts: Already deduplicated

### Metrics Smoothing (Future)
- EMA for latency/throughput
- Reduces chart flicker

##  Next Steps

1. **Desktop Release**
   - Run preflight checks
   - Cut release tag
   - Monitor CI builds
   - Run post-release smoke tests

2. **Mobile**
   - Complete React Native initialization
   - Run E2E tests
   - Prepare store assets
   - Submit to stores

3. **Future Enhancements**
   - Endpoint allowlist UI
   - Rate limiting for toasts
   - EMA smoothing for metrics
   - Desktop "Report Issue" button
   - Mobile "Report Issue" flow

##  Known Issues

- `main.ts` kill-switch implementation needs verification (file path issue resolved)
- Endpoint allowlist validation not yet implemented in UI
- Mobile "Report Issue" flow not yet implemented

##  Support

- Email: codethor@gmail.com
- GitHub: [Repository URL]
- Privacy Policy: [URL]

