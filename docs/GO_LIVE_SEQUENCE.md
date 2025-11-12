# Go-Live Sequence: Desktop 1.1.0

## 🔐 Secrets Check

**GitHub → Repository → Settings → Secrets**

Verify the following secrets are configured:

### macOS Signing
- `APPLE_ID` - Apple ID email
- `APPLE_APP_SPECIFIC_PASSWORD` - App-specific password (not your regular password)
- `APPLE_TEAM_ID` - Team ID (found in Apple Developer account)

### Windows Signing
- `CSC_LINK` - Path to code signing certificate (.pfx file) or base64-encoded certificate
- `CSC_KEY_PASSWORD` - Password for the certificate

**Verification:**
```bash
## Check if secrets are set (will show empty if not set)
gh secret list
```

---

## 📦 SBOM + Listing Validation

### Generate SBOM
```bash
./scripts/generate-sbom.sh
```

Expected output:
- `docs/sbom-gui.json` - Desktop GUI SBOM
- `docs/sbom-mobile.json` - Mobile SBOM (if mobile exists)

### Validate Store Listings
```bash
node store/validate.mjs
```

Expected output:
- ✅ All length checks pass
- ✅ Privacy URL format valid
- ⚠️ Screenshots may be missing (OK for initial release)

---

## 🏷️ Tag & Release

### Pre-Release Checks
```bash
cd gui
make test              # Run dockerized tests
make build-linux       # Build Linux artifacts locally (sanity check)
../scripts/smoke-tests.sh  # Quick smoke tests
```

### Create Release
```bash
cd ..
./scripts/release.sh 1.1.0
git push origin v1.1.0
git push origin main
```

**What happens:**
1. Version bumped in `gui/package.json`
2. `CHANGELOG.md` updated
3. Git tag `v1.1.0` created
4. GitHub Actions triggered:
   - Builds Linux, Windows, macOS artifacts
   - Signs/notarizes if secrets present
   - Creates GitHub Release with artifacts

**Monitor CI:**
- Check GitHub Actions: https://github.com/[your-repo]/actions
- Wait for all 3 platform builds to complete
- Verify artifacts uploaded to Release

---

## 🔒 Gatekeeper Checks

### macOS
```bash
## After release artifacts are downloaded
spctl --assess --type open --verbose dist-package/*.dmg
```

**Expected:** ✅ `dist-package/CrypRQ.dmg: accepted`

**If unsigned:** ⚠️ Will show "rejected" - this is OK for dev builds without secrets

### Windows
```bash
## On Windows machine (or via Wine)
signtool verify /pa dist-package/*.exe
```

**Expected:** ✅ `Successfully verified: CrypRQ.exe`

**If unsigned:** ⚠️ Will show "No signature" - this is OK for dev builds without secrets

---

## 📊 Post-Release Monitoring

### Observability Checks
```bash
./scripts/observability-checks.sh
```

**Expected output:**
- ✅ Desktop logs sanity check
- ✅ Redaction OK - No secrets found
- ✅ Structured log adoption metrics

### Export Diagnostics Once
```bash
## In the app: Help → Export Diagnostics
## Then verify:
unzip -q cryprq-diagnostics-*.zip -d /tmp/diag-check
grep -r -E "bearer |privKey=|authorization:" /tmp/diag-check || echo "✅ No secrets found"
```

**Expected:** ✅ No secrets found

---

## 📱 Mobile Release Path

### Secrets Check

**GitHub → Repository → Settings → Secrets**

- `ANDROID_KEYSTORE_PATH` - Path to keystore file
- `ANDROID_KEYSTORE_PASSWORD` - Keystore password
- `ANDROID_KEY_ALIAS` - Key alias
- `ANDROID_KEY_PASSWORD` - Key password
- `PLAY_JSON_KEY` - Google Play Service Account JSON key
- `APP_STORE_CONNECT_API_KEY` - Apple App Store Connect API key

### Init + CI Smoke

```bash
cd mobile
npm install

## Start fake backend
docker compose up -d fake-cryprq

## Android E2E
npx detox test -c android.emu.debug --headless

## iOS E2E (on macOS)
npx detox test -c ios.sim.debug --record-logs all
```

### Tag to Build Signed Artifacts

```bash
## Tag mobile release
git tag mobile-v1.0.0
git push origin mobile-v1.0.0
```

**Expected:**
- Android AAB appears in release artifacts
- iOS .ipa appears in release artifacts
- Store uploads gated by secrets (will skip if not present)

---

## 🎯 Staged Rollout

### Android
1. **Internal Testing** (immediate)
   - Upload AAB to Play Console → Internal testing track
   - Add internal testers

2. **Closed Testing** (after 24h)
   - Promote to Closed testing track
   - Add closed testers

3. **Production Rollout** (after health gates pass)
   - 10% for 24h → check health gates
   - 50% for 48h → check health gates
   - 100% → full rollout

**Health Gates:**
- Crash-free sessions ≥ 99.5%
- Connect failures ≤ 1%

### iOS
1. **TestFlight** (immediate)
   - Upload .ipa via Fastlane or App Store Connect
   - Add 100 testers
   - Monitor for 24-48h

2. **App Review** (after TestFlight green)
   - Submit for App Review
   - Wait for approval (typically 24-48h)

---

## 🧩 Optional Polish (Already Implemented)

✅ **Endpoint Allowlist** - UI validation in Settings → Security  
✅ **Error Toast Rate-Limit** - Max 1 error toast per 10s  
✅ **EMA Smoothing** - Settings → Charts slider (0-0.4)  
✅ **Report Issue** - Desktop Help menu + modal  

---

## 🔒 One-Time Sanity Checks

### Kill-Switch Test
1. Start CrypRQ
2. Connect to a peer
3. Quit app (Cmd+Q / Alt+F4)
4. **Expected:** Session stops within ≤1s, no orphaned process

**Verify:**
```bash
## Check for orphaned cryprq processes
ps aux | grep cryprq | grep -v grep
## Should return nothing if kill-switch worked
```

### HTTPS Enforcement
**Desktop:**
1. Settings → Security → Manage allowlist → Add `example.com`
2. Try to connect with REMOTE endpoint `http://example.com`
3. **Expected:** Inline error shown, connection blocked

**Mobile:**
1. Settings → Profile → REMOTE
2. Enter `http://example.com`
3. **Expected:** Validation error shown

### Redaction Check
```bash
## Export diagnostics
## Then:
unzip -q cryprq-diagnostics-*.zip -d /tmp/redact-check
if grep -r -E "bearer |privKey=|authorization:" /tmp/redact-check; then
  echo "❌ Secrets leaked!"
  exit 1
else
  echo "✅ Redaction OK"
fi
```

**Expected:** ✅ Redaction OK

### Crash Symbols
**macOS:**
- Check GitHub Actions logs for "Uploading dSYM" step
- Verify dSYM files in release artifacts

**Android:**
- Check GitHub Actions logs for "Uploading ProGuard mapping" step
- Verify `mapping.txt` in release artifacts

---

## 🧭 Quick Incident Runbook

### User Can't Connect

**Steps:**
1. Request diagnostics ZIP from user
2. Extract and open `session-summary.json`
3. Check `last50Events` for state sequence
4. Look for `exitCode` in session entries

**Common Issues:**

**PORT_IN_USE:**
- Ask user to check Settings → Transport → UDP Port
- Suggest changing to an open port (e.g., 9999 → 10000)
- Retry connection

**CLI_NOT_FOUND:**
- Verify CrypRQ binary is installed
- Check PATH environment variable
- Reinstall app

**METRICS_TIMEOUT:**
- Check network connectivity
- Verify metrics endpoint accessible
- Restart session

### Rotation Confusion

**Steps:**
1. Check logs for `rotation.completed` events
2. Verify countdown resynced from metrics within 2s
3. Check Dashboard rotation timer updates

**If rotation not completing:**
- Check network stability
- Verify peer is reachable
- Increase rotation interval if needed

### Windows Launch Blocked

**Symptoms:** Windows Defender/SmartScreen blocks launch

**Steps:**
1. Verify signature:
   ```powershell
   signtool verify /pa CrypRQ.exe
   ```

2. **If unsigned (dev build):**
   - User must click "More info" → "Run anyway"
   - Note: This is expected for unsigned dev builds
   - Replace with signed build when available

3. **If signed but blocked:**
   - Check certificate validity
   - Verify timestamp server response
   - May need to rebuild with valid certificate

---

## 📋 Release Checklist

- [ ] Secrets configured in GitHub
- [ ] SBOM generated
- [ ] Store listings validated
- [ ] Pre-release tests pass
- [ ] Release tag created
- [ ] CI builds complete
- [ ] Artifacts uploaded to GitHub Release
- [ ] Gatekeeper checks pass (macOS)
- [ ] Signature verification passes (Windows)
- [ ] Observability checks run
- [ ] Diagnostics export verified (no secrets)
- [ ] Kill-switch tested
- [ ] HTTPS enforcement tested
- [ ] Redaction verified
- [ ] Crash symbols uploaded

---

## 🚨 Emergency Contacts

- **GitHub Issues:** [Repository URL]/issues
- **Email:** codethor@gmail.com
- **Support:** [Support URL]

---

**Last Updated:** 2025-01-15

