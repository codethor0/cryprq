#!/bin/bash
# Preflight and Tag Script for v1.0.1-web-preview Release
# Usage: ./scripts/preflight-and-tag.sh [--dry-run]

set -euo pipefail

TAG="v1.0.1-web-preview"
DRY_RUN="${1:-}"

echo "=== CrypRQ v1.0.1-web-preview Preflight & Tag Script ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if dry-run
if [ "$DRY_RUN" = "--dry-run" ]; then
    echo -e "${YELLOW}DRY RUN MODE - No changes will be made${NC}"
    echo ""
fi

# Step 1: CLI Validation Check
echo "=== Step 1: CLI Validation Check ==="
if grep -q "✅ PASS\|Status.*PASS" docs/VALIDATION_RUN.md 2>/dev/null; then
    echo -e "${GREEN}✅ CLI validation recorded as PASS${NC}"
    grep -m 1 "✅ PASS\|Status.*PASS" docs/VALIDATION_RUN.md | head -1
else
    echo -e "${RED}❌ CLI validation not found as PASS in docs/VALIDATION_RUN.md${NC}"
    echo "   Please verify CLI file transfer works before proceeding."
    exit 1
fi
echo ""

# Step 2: Web Validation Check (informational)
echo "=== Step 2: Web Validation Status ==="
WEB_STATUS=$(grep -E "WEB-1.*\|" docs/WEB_VALIDATION_RUN.md | head -1 | grep -oE "☐|✅|⚠️|❌" || echo "☐")
if [ "$WEB_STATUS" = "✅" ] || [ "$WEB_STATUS" = "⚠️" ]; then
    echo -e "${GREEN}✅ Web validation has been executed${NC}"
elif [ "$WEB_STATUS" = "☐" ]; then
    echo -e "${YELLOW}⚠️  Web validation not yet executed (WEB-1/WEB-2 marked as TODO)${NC}"
    echo "   Consider running web smoke test before tagging:"
    echo "   1. docker compose -f docker-compose.web.yml up --build"
    echo "   2. Hit web UI and do a file transfer"
    echo "   3. Verify SHA-256 matches"
    echo "   4. Update docs/WEB_VALIDATION_RUN.md"
    echo ""
    read -p "   Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
else
    echo -e "${RED}❌ Web validation shows FAIL${NC}"
    echo "   Please fix web issues before proceeding."
    exit 1
fi
echo ""

# Step 3: Security Disclaimers Check
echo "=== Step 3: Security Disclaimers Check ==="
DISCLAIMER_FILES=(
    "README.md"
    "docs/WEB_STACK_QUICK_START.md"
    "docs/SECURITY_NOTES.md"
    "docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md"
)

ALL_GOOD=true
for file in "${DISCLAIMER_FILES[@]}"; do
    if [ -f "$file" ]; then
        if grep -qiE "test.*mode|static.*keys|MUST NOT.*production|NOT.*FOR.*PRODUCTION" "$file"; then
            echo -e "${GREEN}✅${NC} $file has disclaimers"
        else
            echo -e "${RED}❌${NC} $file missing test-mode disclaimers"
            ALL_GOOD=false
        fi
    else
        echo -e "${RED}❌${NC} $file not found"
        ALL_GOOD=false
    fi
done

if [ "$ALL_GOOD" = false ]; then
    echo ""
    echo -e "${RED}❌ Security disclaimers check failed${NC}"
    echo "   Please add test-mode warnings to all files before proceeding."
    exit 1
fi
echo ""

# Step 4: Git Status Check
echo "=== Step 4: Git Status Check ==="
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}❌ Not in a git repository${NC}"
    exit 1
fi

CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: $CURRENT_BRANCH"

if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}⚠️  Working tree has uncommitted changes${NC}"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
else
    echo -e "${GREEN}✅ Working tree is clean${NC}"
fi

LATEST_COMMIT=$(git log -1 --oneline)
echo "Latest commit: $LATEST_COMMIT"
echo ""

# Step 5: Tag Creation
echo "=== Step 5: Tag Creation ==="
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}❌ Tag $TAG already exists${NC}"
    echo "   Existing tag points to: $(git rev-parse "$TAG")"
    echo ""
    read -p "Delete and recreate? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ "$DRY_RUN" != "--dry-run" ]; then
            git tag -d "$TAG" 2>/dev/null || true
            git push origin ":refs/tags/$TAG" 2>/dev/null || true
        else
            echo -e "${YELLOW}[DRY RUN] Would delete tag $TAG${NC}"
        fi
    else
        echo "Aborted."
        exit 1
    fi
fi

if [ "$DRY_RUN" = "--dry-run" ]; then
    echo -e "${YELLOW}[DRY RUN] Would create tag:${NC}"
    echo "  git tag -a $TAG -m \"CrypRQ v1.0.1 web-only preview (test mode)\""
    echo "  git push origin $TAG"
else
    echo "Creating annotated tag: $TAG"
    git tag -a "$TAG" -m "CrypRQ v1.0.1 web-only preview (test mode)"
    
    echo "Pushing tag to origin..."
    git push origin "$TAG"
    
    echo -e "${GREEN}✅ Tag created and pushed${NC}"
    
    # Verify tag exists
    if git rev-parse "$TAG" >/dev/null 2>&1; then
        echo "Tag verification: $(git rev-parse "$TAG")"
    else
        echo -e "${RED}❌ Tag verification failed${NC}"
        exit 1
    fi
fi
echo ""

# Step 6: GitHub Release Instructions
echo "=== Step 6: GitHub Release Instructions ==="
echo ""
echo -e "${GREEN}Next steps to create GitHub Release:${NC}"
echo ""
echo "1. Go to GitHub → Releases → Draft new release"
echo ""
echo "2. Choose tag: $TAG"
echo ""
echo "3. Title:"
echo "   CrypRQ v1.0.1 — Web-Only Preview (Test Mode, Non-Production)"
echo ""
echo "4. Description: Copy entire contents of:"
echo "   docs/GITHUB_RELEASE_BODY_v1.0.1-web-preview.md"
echo ""
echo "5. Double-check the top of the body clearly warns:"
echo "   - static keys"
echo "   - no handshake"
echo "   - no peer auth"
echo "   - NOT FOR PRODUCTION"
echo ""
echo "6. Click 'Publish release'"
echo ""
echo -e "${GREEN}✅ Preflight complete!${NC}"
echo ""
echo "Tag URL (after release is created):"
echo "  https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/releases/tag/$TAG"
echo ""

# Step 7: Next Phase Reminder
echo "=== Step 7: Next Phase Reminder ==="
echo ""
echo "After the release is published, start the handshake/identity phase:"
echo ""
echo "  git checkout -b feature/handshake-and-identity"
echo ""
echo "Then use: docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md"
echo ""

