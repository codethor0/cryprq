#!/usr/bin/env bash
set -euo pipefail

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${ROOT}/dist/macos"
BIN="${ROOT}/target/release/cryprq"
VERSION="${VERSION:-1.0.1}"

if [[ ! -f "$BIN" ]]; then
    echo "ERROR: Binary not found at $BIN"
    echo "Run: cargo build --release -p cryprq"
    exit 1
fi

APP_DIR="${OUT}/CrypRQ.app"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "${BIN}" "${APP_DIR}/Contents/MacOS/cryprq"
chmod +x "${APP_DIR}/Contents/MacOS/cryprq"

# Copy icon if available
if [[ -f "${ROOT}/dragon_icon_512x512.png" ]]; then
    # Convert PNG to ICNS if sips is available
    if command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
        ICONSET="${OUT}/CrypRQ.iconset"
        rm -rf "${ICONSET}"
        mkdir -p "${ICONSET}"
        
        # Generate various icon sizes
        for size in 16 32 64 128 256 512 1024; do
            sips -z $size $size "${ROOT}/dragon_icon_512x512.png" --out "${ICONSET}/icon_${size}x${size}.png" 2>/dev/null || true
            if [[ $size -ne 1024 ]]; then
                sips -z $((size*2)) $((size*2)) "${ROOT}/dragon_icon_512x512.png" --out "${ICONSET}/icon_${size}x${size}@2x.png" 2>/dev/null || true
            fi
        done
        
        iconutil -c icns "${ICONSET}" -o "${APP_DIR}/Contents/Resources/CrypRQ.icns" 2>/dev/null || true
        rm -rf "${ICONSET}"
    else
        cp "${ROOT}/dragon_icon_512x512.png" "${APP_DIR}/Contents/Resources/CrypRQ.png"
    fi
fi

cat > "${APP_DIR}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>CrypRQ</string>
  <key>CFBundleIdentifier</key>
  <string>com.cryprq.cli</string>
  <key>CFBundleExecutable</key>
  <string>cryprq</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>NSHumanReadableCopyright</key>
  <string>Copyright © 2025 Thor Thor. MIT License.</string>
</dict>
</plist>
EOF

# Code sign the app bundle
echo ""
echo "=== Code Signing App Bundle ==="
if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
    echo "Signing with identity: ${APPLE_SIGNING_IDENTITY}"
    codesign --force --deep --sign "${APPLE_SIGNING_IDENTITY}" "${APP_DIR}" || {
        echo "⚠️  Warning: Code signing failed, trying ad-hoc signature..."
        codesign --force --deep --sign - "${APP_DIR}"
    }
else
    echo "No signing identity found, using ad-hoc signature..."
    codesign --force --deep --sign - "${APP_DIR}"
fi

# Verify signature
if codesign -vvv --deep --strict "${APP_DIR}" 2>&1 | grep -q "valid on disk"; then
    echo "✅ App bundle signed and verified"
else
    echo "⚠️  Warning: Signature verification had issues, but app should still work"
fi

# Remove quarantine attribute if present (from downloads)
xattr -d com.apple.quarantine "${APP_DIR}" 2>/dev/null || true

echo ""
echo "=== macOS App Bundle Created ==="
echo "App: ${APP_DIR}"
ls -lh "${APP_DIR}/Contents/MacOS/cryprq"
echo ""
echo "✅ App is ready to use!"

