# Store Readiness Checklists

## Google Play Store (Internal Testing)

### Prerequisites
- [ ] App bundle (.aab) via Fastlane lane (`fastlane android beta`)
- [ ] Privacy policy URL (point to `repo/docs/privacy.md` or your site)
- [ ] Contact email: `codethor@gmail.com`

### Assets Required
- [ ] Screenshots: 1080×1920 (phone), dark + light themes
- [ ] Short description (80 chars max)
- [ ] Full description (4000 chars max)

### Data Safety Form
- [ ] Telemetry OFF by default
- [ ] List non-PII events when telemetry is ON:
  - Anonymous install ID
  - OS version
  - App version
  - Non-PII events (connect/disconnect/rotation counts)

### Content Rating
- [ ] Complete content rating questionnaire
- [ ] Age rating: Everyone

### Testing Track
- [ ] Internal testing track configured
- [ ] Testers added
- [ ] Release notes prepared

---

## Apple App Store (TestFlight)

### Prerequisites
- [ ] Archive (.ipa) via Fastlane lane (`fastlane ios beta`)
- [ ] App Store Connect API key configured (or manual upload)
- [ ] Privacy policy URL

### App Privacy
- [ ] What's collected when telemetry is ON (opt-in):
  - Device ID (anonymous)
  - Usage Data (non-PII)
  - Diagnostics (redacted)

### Background Modes Justification
- [ ] Background fetch for status updates
- [ ] Rationale: "Periodic status checks to notify users of connection state changes"

### Push Notification Text Review
- [ ] "Connected" notification text
- [ ] "Disconnected" notification text
- [ ] "Keys rotated at HH:MM:SS" notification text

### Assets Required
- [ ] Screenshots: 6.7" (iPhone 14 Pro Max), 6.1" (iPhone 14 Pro), dark + light themes
- [ ] App preview video (optional)
- [ ] Description (4000 chars max)
- [ ] Keywords (100 chars max)

### Review Notes
- [ ] "Controller mode only; no Network Extension used yet."
- [ ] Testing instructions (if needed)
- [ ] Demo account credentials (if required)

---

## Common Requirements

### Privacy Policy
- [ ] Published at accessible URL
- [ ] Covers data collection (telemetry opt-in)
- [ ] Explains log redaction
- [ ] Contact information included

### App Information
- [ ] App name: "CrypRQ"
- [ ] Subtitle (if applicable)
- [ ] Category: Utilities / Security
- [ ] Age rating: 4+ (Everyone)

### Support
- [ ] Support URL (GitHub issues or email)
- [ ] Marketing URL (if applicable)
- [ ] Privacy policy URL

### Version Information
- [ ] Version number matches `package.json` / `build.gradle`
- [ ] Build number incremented
- [ ] Release notes prepared

---

## Pre-Submission Checklist

### Desktop (Electron)
- [ ] All platforms tested (Windows, macOS, Linux)
- [ ] Code signing configured (macOS/Windows)
- [ ] Notarization configured (macOS)
- [ ] Installer tested on clean systems
- [ ] Update mechanism tested

### Mobile (React Native)
- [ ] Android AAB tested on multiple devices
- [ ] iOS archive tested on simulator and device
- [ ] Background refresh tested
- [ ] Notifications tested
- [ ] First-run flow tested
- [ ] Privacy consent flow tested

### Security
- [ ] No hardcoded secrets
- [ ] Log redaction verified
- [ ] HTTPS enforcement verified (REMOTE profile)
- [ ] Certificate pinning configured (if applicable)

### Compliance
- [ ] Privacy policy accessible
- [ ] Data collection opt-in respected
- [ ] Crash reporting opt-in respected
- [ ] No PII in logs (verified)

---

## Post-Submission

### Monitoring
- [ ] Monitor crash reports (if enabled)
- [ ] Monitor user feedback
- [ ] Monitor analytics (if telemetry enabled)
- [ ] Monitor diagnostics exports

### Updates
- [ ] Prepare hotfix process
- [ ] Prepare feature update process
- [ ] Version bumping process documented

---

## Quick Reference

### Fastlane Commands
```bash
## Android
cd mobile
fastlane android build    # Build AAB
fastlane android beta     # Build + upload to Internal testing

## iOS
fastlane ios build        # Archive
fastlane ios beta         # Archive + upload to TestFlight
```

### Version Bumping
```bash
## Desktop
cd gui
npm version patch|minor|major

## Mobile
## Update android/app/build.gradle (versionCode, versionName)
## Update ios/CrypRQ/Info.plist (CFBundleShortVersionString, CFBundleVersion)
```

### Contact
- Email: codethor@gmail.com
- GitHub: [Repository URL]
- Privacy Policy: [URL]

