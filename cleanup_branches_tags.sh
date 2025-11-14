#!/bin/bash
# Clean up stale branches, tags, and releases

set -e

REPO="codethor0/cryprq"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 CLEANING BRANCHES, TAGS & RELEASES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Delete stale branches (keep main only)
echo "1️⃣  Deleting stale branches..."
BRANCHES_TO_DELETE=(
  "web-only-refactor"
  "qa/vnext-20251112"
  "docs/cleanup_20251111_231638"
  "chore/add-funding-links"
  "ci/docker-smoke"
)

for branch in "${BRANCHES_TO_DELETE[@]}"; do
  echo "   Deleting origin/$branch..."
  git push origin --delete "$branch" 2>&1 | grep -v "remote:" || echo "   ✅ Deleted or already gone"
done

echo "✅ Branch cleanup complete"
echo ""

# Delete old tags
echo "2️⃣  Deleting old tags..."
TAGS_TO_DELETE=(
  "pre-web-split-20251113"
  "v0.1.0-alpha.1"
  "v0.0.1-alpha.0"
  "v0.1.0"
)

for tag in "${TAGS_TO_DELETE[@]}"; do
  echo "   Deleting tag $tag..."
  git tag -d "$tag" 2>&1 || true
  git push origin ":refs/tags/$tag" 2>&1 | grep -v "remote:" || echo "   ✅ Deleted or already gone"
done

echo "✅ Tag cleanup complete"
echo ""

# Delete releases
echo "3️⃣  Deleting releases..."
RELEASES_TO_DELETE=(
  "v0.1.0-alpha.1-245-gcd946df-dirty"
  "v1.0.1"
  "v0.1.0-alpha.1"
)

for release in "${RELEASES_TO_DELETE[@]}"; do
  echo "   Deleting release $release..."
  gh release delete "$release" --yes 2>&1 | grep -v "remote:" || echo "   ✅ Deleted or already gone"
done

echo "✅ Release cleanup complete"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ BRANCHES, TAGS & RELEASES CLEANED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
