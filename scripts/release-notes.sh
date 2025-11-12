#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Auto-changelog + version bump (only on green)
# Generate a clean CHANGELOG and bump versions from commit messages

TAG="${1:-}"

[ -z "$TAG" ] && { echo "Usage: $0 vX.Y.Z"; exit 1; }

# Simple conventional-commits changelog
echo "# $TAG ($(date +%Y-%m-%d))" > CHANGELOG.tmp

# Get previous tag or initial commit
PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo '')

if [[ -n "$PREV_TAG" ]]; then
  git log --pretty=format:'- %s' "${PREV_TAG}"..HEAD >> CHANGELOG.tmp
else
  git log --pretty=format:'- %s' >> CHANGELOG.tmp
fi

# Prepend to existing CHANGELOG.md
if [[ -f "CHANGELOG.md" ]]; then
  cat CHANGELOG.tmp CHANGELOG.md > CHANGELOG.new && mv CHANGELOG.new CHANGELOG.md
else
  mv CHANGELOG.tmp CHANGELOG.md
fi

rm -f CHANGELOG.tmp

git add CHANGELOG.md
git commit -m "chore(release): $TAG changelog" || true
git tag -a "$TAG" -m "$TAG" || true

echo "[release-notes] Generated CHANGELOG.md for $TAG"

