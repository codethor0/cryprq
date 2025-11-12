#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Icon Verification - All Platforms"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

MISSING=0
TOTAL=0
MANIFEST_ERRORS=0

check_file() {
    local file=$1
    local platform=$2
    TOTAL=$((TOTAL + 1))
    if [ -f "$file" ]; then
        echo "✅ $platform: $file"
        return 0
    else
        echo "❌ $platform: MISSING $file"
        MISSING=$((MISSING + 1))
        return 1
    fi
}

echo "Android Icons:"
check_file "android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png" "Android (xxxhdpi)"
check_file "android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png" "Android (xxhdpi)"
check_file "android/app/src/main/res/mipmap-xhdpi/ic_launcher.png" "Android (xhdpi)"
check_file "android/app/src/main/res/mipmap-hdpi/ic_launcher.png" "Android (hdpi)"
check_file "android/app/src/main/res/mipmap-mdpi/ic_launcher.png" "Android (mdpi)"
check_file "store-assets/android_play_512.png" "Android Play Store"

echo ""
echo "iOS Icons:"
check_file "apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset/Contents.json" "iOS (Contents.json)"
check_file "apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset/icon_1024.png" "iOS (App Store)"

echo ""
echo "macOS Icons:"
check_file "branding/CrypRQ.icns" "macOS (.icns)"

echo ""
echo "Windows Icons:"
check_file "windows/Assets/AppIcon.ico" "Windows (.ico)"

echo ""
echo "Linux Desktop Icons:"
for size in 16 24 32 48 64 128 256 512; do
    check_file "packaging/linux/hicolor/${size}x${size}/apps/cryprq.png" "Linux (${size}x${size})"
done

echo ""
echo "Electron GUI Icons:"
check_file "gui/build/icon.icns" "Electron (macOS)"
check_file "gui/build/icon.ico" "Electron (Windows)"
check_file "gui/build/icon.png" "Electron (Linux)"

echo ""
echo "Docker Logo:"
check_file "branding/docker-logo.png" "Docker"

echo ""
echo "Manifest Reference Checks:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check Android manifest
if [ -f "android/app/src/main/AndroidManifest.xml" ]; then
    if grep -q "@mipmap/ic_launcher" android/app/src/main/AndroidManifest.xml; then
        echo "✅ Android manifest references @mipmap/ic_launcher"
    else
        echo "❌ Android manifest missing @mipmap/ic_launcher reference"
        MANIFEST_ERRORS=$((MANIFEST_ERRORS + 1))
    fi
fi

# Check Electron package.json
if [ -f "gui/package.json" ]; then
    if grep -q "build/icon" gui/package.json; then
        echo "✅ Electron package.json references build/icon"
    else
        echo "❌ Electron package.json missing build/icon reference"
        MANIFEST_ERRORS=$((MANIFEST_ERRORS + 1))
    fi
fi

# Check Electron builder config
if [ -f "gui/electron-builder.yml" ]; then
    if grep -q "build/icon" gui/electron-builder.yml; then
        echo "✅ Electron builder config references build/icon"
    else
        echo "❌ Electron builder config missing build/icon reference"
        MANIFEST_ERRORS=$((MANIFEST_ERRORS + 1))
    fi
fi

# Check Linux desktop entry
if [ -f "packaging/linux/cryprq.desktop" ]; then
    if grep -q "Icon=cryprq" packaging/linux/cryprq.desktop; then
        echo "✅ Linux desktop entry references Icon=cryprq"
    else
        echo "❌ Linux desktop entry missing Icon=cryprq"
        MANIFEST_ERRORS=$((MANIFEST_ERRORS + 1))
    fi
fi

# Check Dockerfile logo label
if [ -f "Dockerfile" ]; then
    if grep -q "org.opencontainers.image.logo" Dockerfile; then
        echo "✅ Dockerfile includes logo label"
    else
        echo "❌ Dockerfile missing logo label"
        MANIFEST_ERRORS=$((MANIFEST_ERRORS + 1))
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Verification Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Total icon checks: $TOTAL"
echo "Missing icons: $MISSING"
echo "Manifest errors: $MANIFEST_ERRORS"
echo ""

if [ $MISSING -eq 0 ] && [ $MANIFEST_ERRORS -eq 0 ]; then
    echo "✅ All icons present and manifests correct!"
    exit 0
else
    echo "❌ Verification FAILED"
    if [ $MISSING -gt 0 ]; then
        echo "  • $MISSING icon(s) missing. Run 'bash scripts/generate-icons.sh' to generate."
    fi
    if [ $MANIFEST_ERRORS -gt 0 ]; then
        echo "  • $MANIFEST_ERRORS manifest reference(s) incorrect. Update configs to reference correct icon paths."
    fi
    exit 1
fi

