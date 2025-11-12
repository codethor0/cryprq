#!/bin/bash
set -e

# Release script for CrypRQ Desktop
# Usage: ./scripts/release.sh [version]

VERSION=${1:-$(node -p "require('./gui/package.json').version")}
TAG="v${VERSION}"

echo "🚀 Preparing release ${TAG}"

# Verify we're on main/master branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "main" && "$BRANCH" != "master" ]]; then
  echo "⚠️  Warning: Not on main/master branch (currently on ${BRANCH})"
  read -p "Continue anyway? (y/N) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
fi

# Verify working directory is clean
if [[ -n $(git status --porcelain) ]]; then
  echo "❌ Working directory is not clean. Commit or stash changes first."
  exit 1
fi

# Update version in package.json
echo "📝 Updating version to ${VERSION}..."
cd gui
# Use node to update version (more reliable than npm version)
node -e "const fs = require('fs'); const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8')); pkg.version = '${VERSION}'; fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');"
cd ..

# Generate CHANGELOG section (if not already updated)
if ! grep -q "## \[${VERSION}\]" gui/CHANGELOG.md; then
  echo "⚠️  CHANGELOG.md doesn't have entry for ${VERSION}. Please update it manually."
  exit 1
fi

# Commit version bump
git add gui/package.json gui/CHANGELOG.md
git commit -m "chore: bump version to ${VERSION}"

# Create tag
echo "🏷️  Creating tag ${TAG}..."
git tag -a "${TAG}" -m "Release ${TAG}

See CHANGELOG.md for details."

echo "✅ Release ${TAG} prepared!"
echo ""
echo "Next steps:"
echo "  1. Push tag: git push origin ${TAG}"
echo "  2. Push commits: git push origin ${BRANCH}"
echo "  3. CI will build artifacts and create GitHub Release"
echo "  4. Run smoke tests on each platform"

