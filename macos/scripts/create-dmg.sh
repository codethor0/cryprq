#!/usr/bin/env bash
set -euo pipefail

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${ROOT}/dist/macos"
APP_DIR="${OUT}/CrypRQ.app"
VERSION="${VERSION:-1.0.1}"

if [[ ! -d "$APP_DIR" ]]; then
    echo "ERROR: App bundle not found at $APP_DIR"
    echo "Run: macos/scripts/build-app.sh first"
    exit 1
fi

DMG_NAME="CrypRQ-${VERSION}-macOS.dmg"
DMG_PATH="${OUT}/${DMG_NAME}"

# Remove existing DMG
rm -f "${DMG_PATH}"

# Create temporary directory for DMG contents
DMG_TEMP="${OUT}/dmg_temp"
rm -rf "${DMG_TEMP}"
mkdir -p "${DMG_TEMP}"

# Copy app to temp directory
cp -R "${APP_DIR}" "${DMG_TEMP}/"

# Create DMG directly as compressed
hdiutil create -volname "CrypRQ" \
    -srcfolder "${DMG_TEMP}" \
    -ov \
    -fs HFS+ \
    -format UDZO \
    -imagekey zlib-level=9 \
    "${DMG_PATH}"
rm -rf "${DMG_TEMP}"

echo ""
echo "=== DMG Created ==="
echo "DMG: ${DMG_PATH}"
ls -lh "${DMG_PATH}"

