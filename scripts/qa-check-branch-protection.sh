#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Check branch protection status
set -euo pipefail

BRANCH="${1:-main}"
REPO="${2:-codethor0/cryprq}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Branch Protection Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if GitHub CLI is available
if ! command -v gh >/dev/null 2>&1; then
    echo "⚠️ GitHub CLI not available - cannot verify branch protection"
    echo "Please verify manually: https://github.com/${REPO}/settings/branches"
    exit 0
fi

# Check branch protection
echo "Checking branch protection for: ${BRANCH}"
if gh api "repos/${REPO}/branches/${BRANCH}/protection" 2>&1 | tee /tmp/branch-protection.json; then
    echo "✅ Branch protection configured"
    
    # Check required checks
    REQUIRED_CHECKS=$(jq -r '.required_status_checks.contexts[]' /tmp/branch-protection.json 2>/dev/null || echo "")
    if [ -n "$REQUIRED_CHECKS" ]; then
        echo ""
        echo "Required checks:"
        echo "$REQUIRED_CHECKS" | while read check; do
            echo "  - $check"
        done
    else
        echo "⚠️ No required checks configured"
    fi
else
    echo "❌ Branch protection not configured or branch not found"
    echo ""
    echo "Please configure branch protection:"
    echo "  https://github.com/${REPO}/settings/branches"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Branch protection check complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

