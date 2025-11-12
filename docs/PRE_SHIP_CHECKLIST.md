# 10-Minute Pre-Ship Checklist

## Desktop Notarization/Signing

### macOS
- [ ] Apple ID credentials ready (`APPLE_ID`, `APPLE_APP_SPECIFIC_PASSWORD`, `APPLE_TEAM_ID`)
- [ ] Notarize `.dmg` (automated in CI)
- [ ] Staple notarization ticket (`scripts/staple.sh`)
- [ ] Verify Gatekeeper: `spctl --assess --type open --verbose CrypRQ.dmg`

### Windows
- [ ] Code-sign `.exe` (Authenticode/.pfx)
- [ ] `CSC_LINK` and `CSC_KEY_PASSWORD` secrets configured
- [ ] Timestamp server set: `http://timestamp.digicert.com`
- [ ] Verify signature: `signtool verify /pa CrypRQ.exe`

### Linux
- [ ] AppImage and .deb packages built
- [ ] GPG signing (optional, for .deb)

---

## Mobile Signing

### Android
- [ ] Release keystore present (`android/app/release.keystore` or env var)
- [ ] `key.properties` configured (or env vars):
  ```properties
  storeFile=release.keystore
  storePassword=***
  keyAlias=cryprq
  keyPassword=***
  ```
- [ ] ProGuard/R8 mapping upload path documented
- [ ] `build.gradle` signing configs wired
- [ ] `minifyEnabled true`, `shrinkResources true` in release build

### iOS
- [ ] App Store Connect API key configured
- [ ] Certificates and provisioning profiles ready
- [ ] Export options plist prepared (`ios/ExportOptions.plist`)
- [ ] dSYM upload hook configured

---

## Privacy Policy URL

- [ ] Privacy policy URL live and accessible
- [ ] Same URL in:
  - Desktop "Privacy" screen
  - Google Play Store listing
  - Apple App Store listing
- [ ] URL returns 200 OK
- [ ] Content matches app behavior

---

## Crash Symbols

### iOS
- [ ] dSYM upload hook configured in Fastlane
- [ ] dSYM files archived with release

### Android
- [ ] ProGuard mapping upload configured
- [ ] Mapping file attached to release artifacts

### Desktop
- [ ] Symbol files archived (if applicable)
- [ ] Source maps included (if applicable)

---

## SBOM & License

- [ ] Generate SBOM for `gui/`:
  ```bash
  cd gui
  npx @cyclonedx/cyclonedx-npm --output-file sbom.json
  ```
- [ ] Generate SBOM for `mobile/`:
  ```bash
  cd mobile
  npx @cyclonedx/cyclonedx-npm --output-file sbom.json
  ```
- [ ] Add SBOM files to release assets
- [ ] Include OSS attributions (LICENSE files)
- [ ] Verify all dependencies have licenses

---

## Rollback Plan

### Desktop
- [ ] Keep v1.0.x artifacts handy
- [ ] GitHub Release marked as "Latest" only after smoke tests pass
- [ ] Previous version artifacts remain accessible
- [ ] Rollback procedure documented

### Mobile

#### Android
- [ ] Phased rollout configured:
  - 10% for 24h
  - 50% for 48h
  - 100% after health gates pass
- [ ] Health gates:
  - Crash-free sessions ≥ 99.5%
  - Connect failures ≤ 1%
- [ ] Rollback procedure documented

#### iOS
- [ ] TestFlight: 100 testers first (24-48h)
- [ ] Health gates same as Android
- [ ] Promote to App Review only after green
- [ ] Rollback via App Store Connect

---

## Quick Verification Commands

### Desktop
```bash
## macOS Gatekeeper
spctl --assess --type open --verbose dist-package/CrypRQ.dmg

## Windows signature
signtool verify /pa dist-package/CrypRQ.exe

## Linux checksums
sha256sum dist-package/*.{AppImage,deb}
```

### Mobile
```bash
## Android AAB verification
jarsigner -verify -verbose -certs android/app/build/outputs/bundle/release/app-release.aab

## iOS archive verification
codesign -dv --verbose=4 ios/build/Build/Products/Release-iphoneos/CrypRQ.app
```

---

## Pre-Release Smoke Tests

- [ ] Tray: connect → rotate → disconnect (icon/menu update ≤1s)
- [ ] Fault inject: `dev:session:simulateExit` → modal + structured logs v1
- [ ] Diagnostics export: zip <10MB, no bearer/privKey when grepping
- [ ] Kill-switch: quit while connected → disconnect happens
- [ ] Endpoint allowlist: REMOTE profile validation works

---

## CI/CD Verification

- [ ] All CI checks pass
- [ ] Release workflow runs successfully
- [ ] Artifacts uploaded to GitHub Release
- [ ] Release notes generated correctly
- [ ] Signing status indicated in release notes

---

## Post-Release

- [ ] Monitor crash reports (if enabled)
- [ ] Monitor user feedback
- [ ] Run post-release smoke tests
- [ ] Verify structured logs adoption
- [ ] Check diagnostics export usage

---

## Emergency Contacts

- **Email**: codethor@gmail.com
- **GitHub**: [Repository URL]
- **Support**: [Support URL]

---

**Last Updated**: 2025-01-15

