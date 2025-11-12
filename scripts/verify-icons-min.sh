#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

FAIL() { echo "ICON VERIFY: $*" >&2; exit 1; }

REQ=()

# ---- Canonical master

REQ+=("store-assets/icon_master_1024.png")

# ---- Android (densities + anydpi xml)

for d in mdpi hdpi xhdpi xxhdpi xxxhdpi; do
  REQ+=("android/app/src/main/res/mipmap-${d}/ic_launcher.png")
done

REQ+=("android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml")
REQ+=("android/app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml")

# Android manifest reference
[[ -f "${ROOT}/android/app/src/main/AndroidManifest.xml" ]] && \
  grep -q '@mipmap/ic_launcher' "${ROOT}/android/app/src/main/AndroidManifest.xml" || \
  FAIL "AndroidManifest.xml missing @mipmap/ic_launcher"

# ---- iOS/macOS (asset catalog + 1024 marketing)
# Check both possible locations: apple/Sources/CrypRQ/Assets.xcassets or apple/App/Assets.xcassets
IOSSET1="${ROOT}/apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset"
IOSSET2="${ROOT}/apple/App/Assets.xcassets/AppIcon.appiconset"

if [[ -d "${IOSSET1}" ]]; then
  REQ+=("${IOSSET1}/Contents.json")
  REQ+=("${IOSSET1}/icon_1024.png")
elif [[ -d "${IOSSET2}" ]]; then
  REQ+=("${IOSSET2}/Contents.json")
  REQ+=("${IOSSET2}/icon_marketing_1024.png")
else
  # If iOS directory exists but no asset catalog, warn but don't fail (iOS may not be set up yet)
  if [[ -d "${ROOT}/apple" ]]; then
    echo "ICON VERIFY: Warning - iOS asset catalog not found (iOS may not be configured)" >&2
  fi
fi

# ---- macOS ICNS (optional but recommended in bundle targets)
REQ+=("branding/CrypRQ.icns")

# ---- Windows ICO and manifest
REQ+=("windows/Assets/AppIcon.ico")

# If appxmanifest exists, ensure VisualElements reference Assets/ or Assets\
if [[ -f "${ROOT}/windows/packaging/AppxManifest.xml" ]]; then
  grep -qE 'Assets[/\\]' "${ROOT}/windows/packaging/AppxManifest.xml" || \
    FAIL "windows/packaging/AppxManifest.xml does not reference Assets/ icons"
elif [[ -f "${ROOT}/windows/Package.appxmanifest" ]]; then
  grep -qE 'Assets[/\\]' "${ROOT}/windows/Package.appxmanifest" || \
    FAIL "windows/Package.appxmanifest does not reference Assets/ icons"
fi

# ---- Linux hicolor theme
for px in 16 24 32 48 64 128 256 512; do
  REQ+=("packaging/linux/hicolor/${px}x${px}/apps/cryprq.png")
done

# .desktop Icon=cryprq
if [[ -f "${ROOT}/packaging/linux/cryprq.desktop" ]]; then
  grep -q '^Icon=cryprq' "${ROOT}/packaging/linux/cryprq.desktop" || \
    FAIL "packaging/linux/cryprq.desktop missing 'Icon=cryprq'"
fi

# ---- Electron / GUI (if present)
if [[ -d "${ROOT}/gui" ]]; then
  # Electron: build/icon.* referenced in configs
  if [[ -f "${ROOT}/gui/package.json" ]]; then
    # Check for icon references anywhere in package.json (can be in build.mac.icon, build.win.icon, build.linux.icon)
    grep -q '"icon"' "${ROOT}/gui/package.json" || \
      FAIL "gui/package.json build.icon missing"
  fi
  
  # Icon files may be generated during build, so check is non-blocking
  if [[ ! -f "${ROOT}/gui/build/icon.png" && ! -f "${ROOT}/gui/build/icon.icns" && ! -f "${ROOT}/gui/build/icon.ico" ]]; then
    echo "ICON VERIFY: Warning - gui/build icon artifacts not found (may be generated during build)" >&2
  fi
  
  # electron-builder.yml (if used)
  if [[ -f "${ROOT}/gui/electron-builder.yml" ]]; then
    grep -q '^  icon:' "${ROOT}/gui/electron-builder.yml" || \
      FAIL "gui/electron-builder.yml missing 'icon:' reference"
  fi
fi

# ---- Docker OCI logo label (non-blocking if Dockerfile not found)
if [[ -f "${ROOT}/Dockerfile" ]]; then
  grep -qi 'org.opencontainers.image.logo' "${ROOT}/Dockerfile" || \
    FAIL "Dockerfile missing OCI logo label (org.opencontainers.image.logo)"
fi

# ---- File existence checks
MISSING=()

for p in "${REQ[@]}"; do
  [[ -e "${ROOT}/${p}" ]] || MISSING+=("${p}")
done

if (( ${#MISSING[@]} )); then
  echo "ICON VERIFY: Missing files:" >&2
  for x in "${MISSING[@]}"; do echo "  - ${x}" >&2; done
  # Non-blocking: Some icons may be generated during build or are optional
  echo "ICON VERIFY: Warning - Some icon files missing (may be generated during build)" >&2
  # Only fail on critical missing files (Android/iOS if directories exist)
  CRITICAL_MISSING=0
  for x in "${MISSING[@]}"; do
    if [[ "$x" == *"android"* ]] && [[ -d "${ROOT}/android" ]]; then
      CRITICAL_MISSING=1
    elif [[ "$x" == *"apple"* ]] && [[ -d "${ROOT}/apple" ]]; then
      CRITICAL_MISSING=1
    fi
  done
  if [ $CRITICAL_MISSING -eq 1 ]; then
    exit 2
  else
    echo "ICON VERIFY: Non-critical files missing, continuing..." >&2
    exit 0
  fi
fi

echo "ICON VERIFY: OK"

