#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Local Rebuild With Icons and Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check ImageMagick
if ! command -v magick &> /dev/null && ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick not found. Install with:"
    echo "   macOS: brew install imagemagick"
    echo "   Linux: sudo apt-get install imagemagick"
    exit 1
fi

echo "✅ ImageMagick found"
echo ""

# Step 1: Generate icons
echo "[1/4] Generating icons..."
bash scripts/generate-icons.sh
if [ $? -ne 0 ]; then
    echo "❌ Icon generation failed"
    exit 1
fi

# Step 2: Fast verify gate
echo ""
echo "[2/5] Fast icon verification gate..."
if ! bash scripts/verify-icons-min.sh; then
    echo ""
    echo "❌ Fast icon verification FAILED"
    echo "Fix by: bash scripts/generate-icons.sh"
    exit 1
fi
echo "✅ Fast verification passed"

# Step 3: Comprehensive verification
echo ""
echo "[3/5] Comprehensive icon verification..."
bash scripts/verify-icons.sh > artifacts/icons/verify_report.txt 2>&1
VERIFY_EXIT=$?

if [ $VERIFY_EXIT -ne 0 ]; then
    echo ""
    echo "❌ Comprehensive icon verification FAILED"
    echo ""
    echo "Missing or incorrect icons:"
    cat artifacts/icons/verify_report.txt | grep "❌" || true
    echo ""
    echo "Fix by:"
    echo "  1. Re-running: bash scripts/generate-icons.sh"
    echo "  2. Updating manifest references if needed"
    echo "  3. Re-verifying: bash scripts/verify-icons.sh"
    exit 1
fi

echo "✅ Comprehensive verification passed"
cat artifacts/icons/verify_report.txt

# Step 4: Rebuild packages
echo ""
echo "[4/5] Rebuilding packages with icons..."
echo ""

# Android
if [ -d "android" ] && [ -f "android/gradlew" ]; then
    echo "Rebuilding Android..."
    cd android
    chmod +x gradlew
    ./gradlew assembleRelease || echo "⚠️  Android build failed or skipped"
    cd ..
else
    echo "⚠️  Android directory not found, skipping"
fi

# Electron GUI
if [ -d "gui" ] && [ -f "gui/package.json" ]; then
    echo "Rebuilding Electron GUI..."
    cd gui
    npm run build || echo "⚠️  GUI build failed or skipped"
    cd ..
else
    echo "⚠️  GUI directory not found, skipping"
fi

# Step 5: Final verification
echo ""
echo "[5/5] Final verification..."
bash scripts/verify-icons.sh > artifacts/icons/verify_report_final.txt 2>&1
if [ $? -ne 0 ]; then
    echo "❌ Final verification failed"
    cat artifacts/icons/verify_report_final.txt
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Rebuild Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Reports:"
echo "  • artifacts/icons/icon_report.txt - Icon generation report"
echo "  • artifacts/icons/verify_report.txt - Initial verification"
echo "  • artifacts/icons/verify_report_final.txt - Final verification"
echo ""
echo "Next: Verify icons are embedded in rebuilt packages"
echo "  • Android: Check APK/AAB with 'aapt dump badging'"
echo "  • Electron: Check DMG/EXE/AppImage icons"
echo "  • Other platforms: Follow platform-specific verification"

