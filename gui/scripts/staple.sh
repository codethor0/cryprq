#!/bin/bash
set -euo pipefail

# Staple notarization ticket to macOS DMG
# Run after notarization completes

DMG_PATH="${1:-dist-package/*.dmg}"

if [ ! -f "$DMG_PATH" ]; then
  echo "❌ DMG not found: $DMG_PATH"
  exit 1
fi

echo "📎 Stapling notarization ticket to DMG..."
xcrun stapler staple "$DMG_PATH"

echo "✅ Stapling complete"
echo "Verifying..."
spctl --assess --type open --verbose "$DMG_PATH" || {
  echo "⚠️  Gatekeeper assessment failed (may need manual check)"
  exit 1
}

echo "✅ Gatekeeper assessment passed"

