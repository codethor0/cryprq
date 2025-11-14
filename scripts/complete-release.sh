#!/bin/bash

# Copyright (c) 2025 Thor Thor
# Author: Thor Thor (GitHub: https://github.com/codethor0)
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# License: MIT (see LICENSE file for details)

# Complete Release Script for v1.0.1-web-preview
# This script automates Steps 2-4 after web smoke test is complete
# Usage: ./scripts/complete-release.sh

set -euo pipefail

echo "=== CrypRQ v1.0.1-web-preview Complete Release Script ==="
echo ""

# Step 2: Preflight + Tag
echo "=== Step 2: Running Preflight Script ==="
echo ""

# Dry run first
echo "Running dry-run..."
./scripts/preflight-and-tag.sh --dry-run

echo ""
read -p "Dry-run looks good? Continue with actual tag creation? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "Creating tag..."
./scripts/preflight-and-tag.sh

echo ""
echo "=== Step 2 Complete: Tag Created ==="
echo ""

# Step 3: GitHub Release Instructions
echo "=== Step 3: GitHub Release (Manual) ==="
echo ""
echo "Next steps:"
echo "1. Go to GitHub → Releases → Draft new release"
echo "2. Select tag: v1.0.1-web-preview"
echo "3. Title: CrypRQ v1.0.1 — Web-Only Preview (Test Mode)"
echo "4. Copy contents of: docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md"
echo "5. Paste into release description"
echo "6. Verify test-mode warnings are visible"
echo "7. Click 'Publish release'"
echo ""

read -p "Press Enter when GitHub release is published, or Ctrl+C to exit..."
echo ""

# Step 4: Switch to Next Phase
echo "=== Step 4: Switching to Next Phase ==="
echo ""

CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "⚠️  Warning: Not on main branch (currently on $CURRENT_BRANCH)"
    read -p "Switch to main first? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git checkout main
    fi
fi

echo "Switching to feature/handshake-and-identity..."
git checkout feature/handshake-and-identity

echo "Merging main into feature branch..."
git merge main -m "chore: merge main into handshake-and-identity branch"

echo ""
echo "=== Step 4 Complete: Ready for Next Phase ==="
echo ""
echo "Next steps:"
echo "1. Open: docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md"
echo "2. Use that prompt to guide:"
echo "   - Real handshake (CRYPRQ_CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH)"
echo "   - Peer identity & auth"
echo "   - Remove static keys"
echo "   - Remove 'both sides are initiator' test hack"
echo "   - Harden SECURITY_NOTES.md for production"
echo ""
echo "✅ Release process complete!"
echo ""

