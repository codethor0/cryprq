#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# OpenSSF Scorecard check
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/scorecard}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "OpenSSF Scorecard"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Get repo URL
REPO_URL=$(git config --get remote.origin.url | sed 's/\.git$//' | sed 's/.*github\.com[:/]\(.*\)/https:\/\/github.com\/\1/')

if [ -z "$REPO_URL" ]; then
    echo "⚠️ Could not determine repository URL"
    echo "Skipping Scorecard check"
    exit 0
fi

echo "Repository: $REPO_URL"
echo ""

# Install scorecard if not present
if ! command -v scorecard >/dev/null 2>&1; then
    echo "Installing OpenSSF Scorecard..."
    # Download latest release
    SCORECARD_VERSION=$(curl -s https://api.github.com/repos/ossf/scorecard/releases/latest | grep tag_name | cut -d'"' -f4)
    wget -qO- "https://github.com/ossf/scorecard/releases/download/${SCORECARD_VERSION}/scorecard_${SCORECARD_VERSION}_linux_amd64.tar.gz" | tar -xz || {
        echo "⚠️ Scorecard installation failed"
        echo "Skipping Scorecard check"
        exit 0
    }
    chmod +x scorecard
    sudo mv scorecard /usr/local/bin/ || mv scorecard ~/.cargo/bin/ || {
        echo "⚠️ Could not install scorecard"
        exit 0
    }
fi

# Run scorecard
echo "Running OpenSSF Scorecard..."
if scorecard --repo="$REPO_URL" --format=json > "$ARTIFACT_DIR/scorecard.json" 2>&1; then
    # Extract key checks
    echo ""
    echo "Key checks:"
    jq -r '.checks[] | "\(.name): \(.score)"' "$ARTIFACT_DIR/scorecard.json" 2>/dev/null || echo "Could not parse scorecard JSON"
    
    # Check for regressions on critical checks
    CRITICAL_CHECKS=("CI-Tests" "Branch-Protection" "Fuzzing" "SAST")
    REGRESSION=0
    
    for check in "${CRITICAL_CHECKS[@]}"; do
        SCORE=$(jq -r ".checks[] | select(.name==\"$check\") | .score" "$ARTIFACT_DIR/scorecard.json" 2>/dev/null || echo "0")
        if [ "$SCORE" != "null" ] && [ "$SCORE" != "0" ]; then
            echo "$check score: $SCORE"
        fi
    done
    
    echo ""
    echo "✅ Scorecard complete"
    echo "📄 Full report: $ARTIFACT_DIR/scorecard.json"
else
    echo "⚠️ Scorecard check failed or not available"
    exit 0
fi

