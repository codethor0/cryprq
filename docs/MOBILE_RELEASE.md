# Mobile Release Path

## 🔐 Secrets Check

**GitHub → Repository → Settings → Secrets**

Verify the following secrets are configured:

### Android Signing
- `ANDROID_KEYSTORE_PATH` - Path to keystore file (or base64-encoded keystore)
- `ANDROID_KEYSTORE_PASSWORD` - Keystore password
- `ANDROID_KEY_ALIAS` - Key alias (e.g., `cryprq`)
- `ANDROID_KEY_PASSWORD` - Key password

### Google Play Store
- `PLAY_JSON_KEY` - Google Play Service Account JSON key (base64-encoded)

### iOS Signing
- `APP_STORE_CONNECT_API_KEY` - Apple App Store Connect API key (base64-encoded)
- `APP_STORE_CONNECT_API_ISSUER` - API key issuer ID
- `APP_STORE_CONNECT_API_KEY_ID` - API key ID

**Verification:**
```bash
gh secret list | grep -E "ANDROID_|PLAY_|APP_STORE"
```

---

## 🧪 Init + CI Smoke

### Setup
```bash
cd mobile
npm install
```

### Start Fake Backend
```bash
docker compose up -d fake-cryprq
```

### Android E2E Tests
```bash
# Build app
cd android
./gradlew assembleDebug

# Run Detox tests (headless)
cd ..
npx detox build -c android.emu.debug
npx detox test -c android.emu.debug --headless --record-logs all
```

**Expected:**
- ✅ App builds successfully
- ✅ Detox tests pass
- ✅ Artifacts uploaded on failure (screenshots/logs)

### iOS E2E Tests (macOS only)
```bash
# Build app
cd ios
xcodebuild -workspace CrypRQ.xcworkspace \
  -scheme CrypRQ \
  -configuration Debug \
  -sdk iphonesimulator \
  -derivedDataPath build

# Run Detox tests
cd ..
npx detox build -c ios.sim.debug
npx detox test -c ios.sim.debug --record-logs all
```

**Expected:**
- ✅ App builds successfully
- ✅ Detox tests pass
- ✅ Artifacts uploaded on failure

---

## 🏷️ Tag to Build Signed Artifacts

### Create Mobile Release Tag
```bash
git tag mobile-v1.0.0
git push origin mobile-v1.0.0
```

### Monitor CI
- Check GitHub Actions: `.github/workflows/mobile-release.yml`
- Wait for Android and iOS builds to complete
- Verify artifacts uploaded to Release:
  - `cryprq-mobile-*.aab` (Android)
  - `cryprq-mobile-*.ipa` (iOS)

### Store Uploads
**If secrets present:**
- Android AAB uploaded to Play Console (Internal track)
- iOS IPA uploaded to TestFlight

**If secrets missing:**
- Artifacts attached to GitHub Release
- Store uploads skipped (clear log message)

---

## 🎯 Staged Rollout

### Android

#### Phase 1: Internal Testing (Immediate)
1. Upload AAB to Play Console → Internal testing track
2. Add internal testers
3. Monitor for 24 hours

**Health Gates:**
- Crash-free sessions ≥ 99.5%
- Connect failures ≤ 1%

#### Phase 2: Closed Testing (After 24h)
1. Promote to Closed testing track
2. Add closed testers
3. Monitor for 48 hours

**Health Gates:**
- Same as Phase 1
- Average connection time < 10s

#### Phase 3: Production Rollout (After health gates pass)

**10% Rollout (24h):**
- Set rollout percentage to 10%
- Monitor health gates
- Check crash reports and user feedback

**50% Rollout (48h):**
- If 10% passes health gates, increase to 50%
- Continue monitoring

**100% Rollout:**
- If 50% passes health gates, roll out to 100%
- Full production release

**Rollback Procedure:**
If health gates fail at any phase:
1. Pause rollout in Play Console
2. Investigate issues
3. Deploy hotfix if needed
4. Restart from Phase 1

### iOS

#### Phase 1: TestFlight (Immediate)
1. Upload .ipa to TestFlight
2. Add 100 testers
3. Monitor for 24-48 hours

**Health Gates:**
- Crash-free sessions ≥ 99.5%
- Connect failures ≤ 1%
- Positive TestFlight feedback

#### Phase 2: App Review (After TestFlight green)
1. Submit for App Review
2. Wait for approval (typically 24-48h)
3. Monitor after release

**Rollback Procedure:**
If issues found:
1. Do not submit to App Review
2. Fix issues
3. Upload new TestFlight build
4. Re-test with TestFlight testers

---

## 📊 Health Gate Monitoring

### Key Metrics

**Crash Rate:**
- Target: < 0.5%
- Alert: > 1%
- Critical: > 2%

**Connect Failure Rate:**
- Target: < 1%
- Alert: > 2%
- Critical: > 5%

**Average Connection Time:**
- Target: < 10s
- Alert: > 20s

**Session Duration:**
- Target: > 5 minutes average
- Alert: < 1 minute average

### Monitoring Tools

**Android:**
- Google Play Console → Quality → Crashes & ANRs
- Firebase Crashlytics (if integrated)
- Google Play Console → Quality → Vitals

**iOS:**
- App Store Connect → TestFlight → Crashes
- Xcode Organizer → Crashes
- TestFlight feedback

---

## 🚨 Emergency Contacts

- **GitHub Issues:** [Repository URL]/issues
- **Email:** codethor@gmail.com
- **Support:** [Support URL]

---

**Last Updated:** 2025-01-15

