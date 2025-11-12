#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_NAME="${APP_NAME:-CrypRQ}"
ID="${BUNDLE_ID:-io.cryrpq.app}"
ICON_ICNS="${ICON_ICNS:-branding/AppIcon.icns}"
APPDIR="${ROOT}/artifacts/macos-app/${APP_NAME}.app"
MACBIN="${ROOT}/target/aarch64-apple-darwin/release/cryprq"

have(){ command -v "$1" >/dev/null 2>&1; }
note(){ echo "[package-mac-app] $*"; }
fail(){ echo "[package-mac-app] ERROR: $*" >&2; exit 1; }

# Build binary if missing
if [[ ! -x "$MACBIN" ]]; then
  note "Building macOS binary..."
  rustup target add aarch64-apple-darwin >/dev/null 2>&1 || true
  cargo build --release -p cryprq --target aarch64-apple-darwin || fail "Build failed"
fi

rm -rf "$APPDIR"
mkdir -p "$APPDIR/Contents/MacOS" "$APPDIR/Contents/Resources"

# Info.plist
cat > "$APPDIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>${APP_NAME}</string>
  <key>CFBundleDisplayName</key><string>${APP_NAME}</string>
  <key>CFBundleIdentifier</key><string>${ID}</string>
  <key>CFBundleVersion</key><string>1.0.0</string>
  <key>CFBundleShortVersionString</key><string>1.0.0</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>CFBundleExecutable</key><string>cryprq</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
</dict></plist>
PLIST

# Icon
if [[ -f "$ICON_ICNS" ]]; then
  cp "$ICON_ICNS" "$APPDIR/Contents/Resources/AppIcon.icns"
  note "Icon copied: $ICON_ICNS"
else
  note "Icon not found: $ICON_ICNS (skipping)"
fi

# Binary
cp "$MACBIN" "$APPDIR/Contents/MacOS/cryprq"
chmod +x "$APPDIR/Contents/MacOS/cryprq"
note "Binary copied: $MACBIN"

# Optional signing
if [[ -n "${SIGNING_IDENTITY:-}" ]]; then
  note "Signing app with identity: $SIGNING_IDENTITY"
  codesign --force --options runtime --timestamp -s "$SIGNING_IDENTITY" "$APPDIR" || fail "Codesign failed"
  note "App signed successfully"
else
  note "No SIGNING_IDENTITY set; skipping codesign"
fi

# Optional notarization (requires env vars)
if [[ -n "${NOTARY_APPLE_ID:-}" && -n "${NOTARY_TEAM_ID:-}" && -n "${NOTARY_API_KEY_ID:-}" && -n "${NOTARY_API_KEY_ISSUER:-}" ]]; then
  note "Notarizing app..."
  ditto -c -k --keepParent "$APPDIR" "${APPDIR}.zip"
  
  # Store API key in keychain
  xcrun notarytool store-credentials "$NOTARY_PROFILE" \
    --apple-id "$NOTARY_APPLE_ID" \
    --team-id "$NOTARY_TEAM_ID" \
    --key-id "$NOTARY_API_KEY_ID" \
    --key "$NOTARY_API_KEY" \
    --keychain-profile "$NOTARY_PROFILE" || true
  
  xcrun notarytool submit "${APPDIR}.zip" \
    --apple-id "$NOTARY_APPLE_ID" \
    --team-id "$NOTARY_TEAM_ID" \
    --keychain-profile "$NOTARY_PROFILE" \
    --wait || fail "Notarization failed"
  
  xcrun stapler staple "$APPDIR" || fail "Stapling failed"
  note "App notarized and stapled successfully"
else
  note "Notarization credentials not set; skipping notarization"
fi

note "macOS app created at: $APPDIR"

# Smoke test: verify app launches
note "Running smoke test..."
"$APPDIR/Contents/MacOS/cryprq" --version >/dev/null 2>&1 || fail "App binary failed smoke test"

note "✅ macOS app packaging complete"

