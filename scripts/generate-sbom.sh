#!/bin/bash
set -euo pipefail

# Generate SBOM for Desktop and Mobile

echo "📦 Generating SBOM files..."

# Desktop GUI
if [ -d "gui" ]; then
  echo "Generating SBOM for gui/..."
  cd gui
  if command -v cyclonedx-npm &> /dev/null; then
    cyclonedx-npm --output-file ../docs/sbom-gui.json
  elif command -v npx &> /dev/null; then
    npx @cyclonedx/cyclonedx-npm --output-file ../docs/sbom-gui.json
  else
    echo "⚠️  cyclonedx-npm not found, skipping GUI SBOM"
  fi
  cd ..
fi

# Mobile
if [ -d "mobile" ]; then
  echo "Generating SBOM for mobile/..."
  cd mobile
  if command -v cyclonedx-npm &> /dev/null; then
    cyclonedx-npm --output-file ../docs/sbom-mobile.json
  elif command -v npx &> /dev/null; then
    npx @cyclonedx/cyclonedx-npm --output-file ../docs/sbom-mobile.json
  else
    echo "⚠️  cyclonedx-npm not found, skipping Mobile SBOM"
  fi
  cd ..
fi

echo "✅ SBOM generation complete"
echo "Files:"
ls -lh docs/sbom-*.json 2>/dev/null || echo "No SBOM files generated"

