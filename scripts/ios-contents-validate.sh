#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Check both possible locations
IOSSET1="${ROOT}/apple/App/Assets.xcassets/AppIcon.appiconset"
IOSSET2="${ROOT}/apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset"

if [[ -d "${IOSSET1}" ]]; then
    IOSSET="${IOSSET1}"
elif [[ -d "${IOSSET2}" ]]; then
    IOSSET="${IOSSET2}"
else
    # If iOS directory exists but no asset catalog, skip validation (iOS may not be set up yet)
    if [[ -d "${ROOT}/apple" ]]; then
        echo "iOS ICON VALIDATE: Skipping - iOS asset catalog not found (iOS may not be configured)"
        exit 0
    else
        echo "iOS ICON VALIDATE: Skipping - iOS directory not found"
        exit 0
    fi
fi

JSON="${IOSSET}/Contents.json"

fail(){ echo "iOS ICON VALIDATE: $*" >&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || fail "Missing tool: $1"; }

need jq
[[ -f "$JSON" ]] || fail "Missing $JSON"

# Required entries (common minimal set). Add/remove to match your project's target matrix.
declare -a REQ=(
  '1024x1024'           # marketing icon
  '120x120' '180x180'   # iPhone app icons (60pt@2x, 60pt@3x)
  '80x80' '167x167'     # iPad (40pt@2x, 83.5pt@2x)
  '58x58' '87x87'       # spotlight/settings (29pt@2x/@3x)
  '80x80@ipad'          # iPad spotlight 40pt@2x (redundant check for some templates)
)

# Extract all declared pixel sizes from Contents.json
sizes=($(jq -r '.images[]?|select(.size and .scale and .filename) | 
  (.size|split("x")[0]|tonumber) as $pt |
  (.scale|sub("x$";"")|tonumber) as $sc |
  "\((($pt)*$sc)|tostring)x\((($pt)*$sc)|tostring)"' "$JSON" 2>/dev/null | sort -u))

# Helper to test presence of a size token, allowing "@ipad" hints
has_size(){
  local token="$1"
  local sz="${token%@ipad}"
  for s in "${sizes[@]}"; do [[ "$s" == "$sz" ]] && return 0; done
  return 1
}

missing=()
for r in "${REQ[@]}"; do has_size "$r" || missing+=("$r"); done

if (( ${#missing[@]} )); then
  echo "iOS ICON VALIDATE: Missing pixel sizes (derived from Contents.json):" >&2
  for m in "${missing[@]}"; do echo "  - $m" >&2; done
  exit 2
fi

# Confirm the 1024 marketing icon exists on disk (check both possible filenames)
if [[ ! -f "${IOSSET}/icon_marketing_1024.png" ]] && [[ ! -f "${IOSSET}/icon_1024.png" ]]; then
    fail "Missing icon_marketing_1024.png or icon_1024.png (App Store marketing icon)"
fi

echo "iOS ICON VALIDATE: OK"

