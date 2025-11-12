#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Safe rollback: Undo last green auto-commit (keeps changes staged)
# Usage: bash scripts/rollback-last-commit.sh

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Check if we're in a git repo
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Error: Not in a git repository" >&2
  exit 1
fi

# Check if there's a commit to rollback
if ! git rev-parse HEAD >/dev/null 2>&1; then
  echo "Error: No commits to rollback" >&2
  exit 1
fi

# Get last commit message
LAST_MSG=$(git log -1 --pretty=format:"%s" HEAD)

# Check if it's a green gate auto-commit
if [[ "$LAST_MSG" != *"green gate auto-commit"* ]]; then
  echo "Warning: Last commit doesn't appear to be a green gate auto-commit"
  echo "Last commit: $LAST_MSG"
  read -p "Continue anyway? (y/N) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
fi

# Show what will be rolled back
echo "Rolling back last commit:"
git log -1 --oneline HEAD
echo ""

# Soft reset (keeps changes staged)
git reset --soft HEAD~1

echo "✅ Rollback complete"
echo "Changes are staged and ready to edit/commit"
echo ""
echo "To unstage: git reset HEAD"
echo "To discard changes: git reset --hard HEAD"

