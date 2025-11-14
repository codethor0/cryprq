#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Helper script to find and display available code signing identities

set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Available Code Signing Identities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Find all code signing identities
IDENTITIES=$(security find-identity -v -p codesigning 2>&1 | grep -E "^\s+[0-9]+\)" || true)

if [[ -z "$IDENTITIES" ]]; then
    echo "❌ No code signing identities found."
    echo ""
    echo "To create a Developer ID:"
    echo "  1. Visit: https://developer.apple.com/account/resources/certificates/list"
    echo "  2. Click '+' to create a new certificate"
    echo "  3. Select 'Developer ID Application'"
    echo "  4. Follow the wizard and install the certificate"
    echo ""
    exit 1
fi

echo "$IDENTITIES" | while IFS= read -r line; do
    # Extract the identity name
    IDENTITY=$(echo "$line" | sed -E 's/^[[:space:]]+[0-9]+\)[[:space:]]+"([^"]+)"[[:space:]]+.*/\1/')
    
    # Check if it's a Developer ID (recommended for distribution)
    if echo "$IDENTITY" | grep -q "Developer ID Application"; then
        echo "✅ RECOMMENDED: $IDENTITY"
    elif echo "$IDENTITY" | grep -q "Apple Development"; then
        echo "⚠️  Development only: $IDENTITY"
    else
        echo "   $IDENTITY"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💡 Usage"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "To use a Developer ID for signing, set:"
echo ""
echo "  export APPLE_SIGNING_IDENTITY='Developer ID Application: Your Name (TEAM_ID)'"
echo ""
echo "Then rebuild:"
echo "  ./macos/scripts/build-app.sh"
echo ""

