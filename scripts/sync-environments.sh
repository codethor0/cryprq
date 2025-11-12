#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Synchronize files across different environments
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔄 Synchronizing Environments"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for rsync
if ! command -v rsync &> /dev/null; then
    echo "❌ ERROR: rsync not found. Please install rsync."
    exit 1
fi

# Configuration
REMOTE_HOST="${REMOTE_HOST:-}"
REMOTE_PATH="${REMOTE_PATH:-/path/to/remote}"
EXCLUDE_PATTERNS=(
    "target/"
    "node_modules/"
    ".git/"
    "dist/"
    "*.log"
    ".DS_Store"
    "build/"
)

if [ -z "$REMOTE_HOST" ]; then
    echo "⚠️  REMOTE_HOST not set. Skipping remote sync."
    echo ""
    echo "Usage:"
    echo "  REMOTE_HOST=user@host REMOTE_PATH=/path/to/remote bash scripts/sync-environments.sh"
    exit 0
fi

echo "📡 Syncing to $REMOTE_HOST:$REMOTE_PATH"
echo ""

# Build exclude string
EXCLUDE_ARGS=()
for pattern in "${EXCLUDE_PATTERNS[@]}"; do
    EXCLUDE_ARGS+=(--exclude="$pattern")
done

# Sync files
rsync -avz \
    "${EXCLUDE_ARGS[@]}" \
    --delete \
    ./ "$REMOTE_HOST:$REMOTE_PATH/"

echo ""
echo "✅ Synchronization completed"
echo ""

exit 0

