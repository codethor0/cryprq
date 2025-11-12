#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "ANDROID ICON VALIDATE: $*" >&2; exit 1; }

# Skip if Android directory doesn't exist
if [[ ! -d "${ROOT}/android" ]]; then
    echo "ANDROID ICON VALIDATE: Skipping - Android directory not found"
    exit 0
fi

declare -a DENS=(mdpi hdpi xhdpi xxhdpi xxxhdpi)
for d in "${DENS[@]}"; do
  f="${ROOT}/android/app/src/main/res/mipmap-${d}/ic_launcher.png"
  [[ -s "$f" ]] || fail "Missing or empty: $f"
done

# anydpi v26 XMLs are recommended
for x in ic_launcher.xml ic_launcher_round.xml; do
  [[ -s "${ROOT}/android/app/src/main/res/mipmap-anydpi-v26/${x}" ]] || \
    fail "Missing anydpi XML: ${x}"
done

AM="${ROOT}/android/app/src/main/AndroidManifest.xml"
[[ -f "$AM" ]] || fail "Missing AndroidManifest.xml"
grep -q '@mipmap/ic_launcher' "$AM" || fail "AndroidManifest.xml missing @mipmap/ic_launcher reference"

echo "ANDROID ICON VALIDATE: OK"

