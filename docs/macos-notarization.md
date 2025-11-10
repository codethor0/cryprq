# macOS Signed App & Notarized DMG Pipeline

This document describes the automation required to produce a hardened `CrypRQ.app`, wrap it in a notarized DMG, and verify Gatekeeper compliance.

## Prerequisites

- Apple Developer ID Application certificate (`Developer ID Application: Thor Thor (TEAMID)`).
- Apple Developer ID Installer certificate (for signing DMG / installer packages).
- `notarytool` credentials stored via `xcrun notarytool store-credentials`, referenced in CI.
- Entitlements plist for the CLI (`cryprq.entitlements`) and the app bundle (`CrypRQApp.entitlements`).
- Hardened runtime enabled on all binaries.

## Directory Layout

```
macos/
  entitlements/
    cryprq-cli.plist
    CrypRQApp.plist
  icons/CrypRQ.icns
  scripts/
    build-app.sh
    sign-app.sh
    notarize-dmg.sh
    staple.sh
    verify.sh
```

## Build Script (`build-app.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${ROOT}/dist/macos"
BIN="${ROOT}/target/release/cryprq"

cargo build --release -p cryprq

APP_DIR="${OUT}/CrypRQ.app"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "${BIN}" "${APP_DIR}/Contents/MacOS/cryprq"
cp "${ROOT}/macos/icons/CrypRQ.icns" "${APP_DIR}/Contents/Resources/"

cat > "${APP_DIR}/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>CrypRQ</string>
  <key>CFBundleIdentifier</key><string>com.cryprq.cli</string>
  <key>CFBundleExecutable</key><string>cryprq</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
EOF
```

`VERSION` exported by the calling workflow.

## Signing Script (`sign-app.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

APP_PATH="$1"
CERT_IDENTITY="Developer ID Application: Thor Thor (TEAMID)"
ENTITLEMENTS="macos/entitlements/CrypRQApp.plist"

codesign --force --options runtime --timestamp \
  --entitlements "${ENTITLEMENTS}" \
  --sign "${CERT_IDENTITY}" \
  "${APP_PATH}/Contents/MacOS/cryprq"

codesign --force --options runtime --timestamp \
  --entitlements "${ENTITLEMENTS}" \
  --sign "${CERT_IDENTITY}" \
  "${APP_PATH}"
```

- Ensure dependent frameworks (if any) are signed individually before the bundle.
- Validate with `codesign --verify --deep --strict ${APP_PATH}`.

## DMG Creation

```bash
hdiutil create -volname "CrypRQ" -srcfolder "${APP_PATH}" -ov -fs HFS+ "${OUT}/CrypRQ.dmg"
```

## Notarization (`notarize-dmg.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

DMG="$1"
PROFILE="cryprq-notary" # stored via `notarytool store-credentials`

xcrun notarytool submit "${DMG}" --keychain-profile "${PROFILE}" --wait
```

## Stapling

```bash
xcrun stapler staple "${DMG}"
```

## Verification (`verify.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

APP_PATH="$1"
DMG_PATH="$2"

codesign --verify --deep --strict --verbose=2 "${APP_PATH}"
spctl --assess --verbose=2 "${APP_PATH}"
spctl --assess --type open --verbose=2 "${DMG_PATH}"
```

## CI Workflow (`.github/workflows/macos-dmg.yml`)

```yaml
name: macOS DMG

on:
  workflow_dispatch:
  push:
    branches: [ main ]

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        run: rustup default 1.83.0
      - name: Build app bundle
        run: macos/scripts/build-app.sh
        env:
          VERSION: ${{ github.run_number }}
      - name: Sign app
        run: macos/scripts/sign-app.sh dist/macos/CrypRQ.app
        env:
          SIGNING_IDENTITY: ${{ secrets.MACOS_DEV_ID_APP }}
          KEYCHAIN_PASSWORD: ${{ secrets.MACOS_KEYCHAIN_PASSWORD }}
      - name: Create DMG
        run: hdiutil create -volname CrypRQ -srcfolder dist/macos/CrypRQ.app -ov -fs HFS+ dist/macos/CrypRQ.dmg
      - name: Notarize DMG
        run: macos/scripts/notarize-dmg.sh dist/macos/CrypRQ.dmg
        env:
          NOTARY_PROFILE: ${{ secrets.MACOS_NOTARY_PROFILE }}
      - name: Staple
        run: macos/scripts/staple.sh dist/macos/CrypRQ.dmg
      - name: Verify
        run: macos/scripts/verify.sh dist/macos/CrypRQ.app dist/macos/CrypRQ.dmg
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: cryprq-dmg
          path: dist/macos/CrypRQ.dmg
```

- CI needs access to signing certificates via encrypted secrets. Create a temporary keychain, import `.p12`, and set key partition list to allow `codesign`.
- Non-interactive `notarytool` requires a stored profile or an API key (App Store Connect API).

## Hardened Runtime Entitlements

`CrypRQApp.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.cs.allow-jit</key><false/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key><false/>
  <key>com.apple.security.cs.disable-library-validation</key><true/>
  <key>com.apple.security.cs.allow-dyld-environment-variables</key><false/>
  <key>com.apple.security.app-sandbox</key><false/>
</dict>
</plist>
```

Adjust entitlements if the CLI uses networking or needs additional capabilities (e.g., `com.apple.security.network.client`).

## Troubleshooting

- **Notarization reject: missing entitlements** — ensure `codesign` includes entitlements for both binary and app bundle, and that hardened runtime is enabled.
- **Third-party dylibs** — codesign them individually before signing the app bundle. Use `otool -L` to list dependencies.
- **Timeouts** — `notarytool` with `--wait` reports progress; for large DMGs consider using `altool` fallback or manage asynchronous submission.
- **Stapler fails** — ensure notarization succeeded (`notarytool history --keychain-profile ...`).

## Gatekeeper Validation

On a clean macOS machine:

```bash
hdiutil attach CrypRQ.dmg
spctl --assess --type execute CrypRQ.app
./CrypRQ.app/Contents/MacOS/cryprq --help
```

Document the verification steps in release notes and QA artifacts.

