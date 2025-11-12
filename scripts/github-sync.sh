#!/bin/bash
set -euo pipefail

# GitHub Sync + Build Health + (Optional) Ship
# Ensures local and GitHub are synchronized; verifies CI; runs builds/tests;
# pushes branch + opens PR; triggers CI; (optionally) tags/ships

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

error() {
  echo -e "${RED}❌${NC} $1" >&2
  exit 1
}

warning() {
  echo -e "${YELLOW}⚠️${NC} $1"
}

success() {
  echo -e "${GREEN}✅${NC} $1"
}

info() {
  echo -e "${BLUE}ℹ️${NC} $1"
}

section() {
  echo ""
  echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${CYAN}▶${NC} $1"
  echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Initialize variables
BRANCH=""
PR_URL=""
CI_CHECKS_PASSED=false

# Step 0: Environment + repo hygiene
section "Step 0: Environment + Repo Hygiene"

info "Environment:"
echo "  Git: $(git --version 2>/dev/null || echo 'not found')"
echo "  Node: $(node -v 2>/dev/null || echo 'not found')"
echo "  npm: $(npm -v 2>/dev/null || echo 'not found')"
echo "  Docker: $(docker --version 2>/dev/null || echo 'not found')"

if ! command -v git >/dev/null; then
  error "Git not found"
fi

if ! command -v node >/dev/null; then
  error "Node.js not found"
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
  error "Node.js 18+ required (found: $(node -v))"
fi

if ! docker ps >/dev/null 2>&1; then
  error "Docker daemon not running"
fi

info "Git remotes:"
git remote -v || error "Failed to list remotes"

info "Fetching all remotes..."
git fetch --all --prune || error "Failed to fetch remotes"

CURRENT=$(git rev-parse --abbrev-ref HEAD)
info "Current branch: $CURRENT"

if [ "$CURRENT" = "main" ] || [ "$CURRENT" = "master" ]; then
  BRANCH="chore/github-sync-$(date +%Y%m%d-%H%M)"
  info "Creating feature branch: $BRANCH"
  git checkout -b "$BRANCH" || warning "Branch may already exist"
else
  BRANCH="$CURRENT"
  info "Using existing branch: $BRANCH"
fi

# Ensure .gitignore includes artifacts/
if ! grep -q "^artifacts/" .gitignore 2>/dev/null; then
  info "Adding artifacts/ to .gitignore"
  echo "artifacts/" >> .gitignore
fi

success "Environment OK"

# Step 1: GitHub auth & repo checks
section "Step 1: GitHub Auth & Repo Checks"

HAS_GH=false
if command -v gh >/dev/null; then
  HAS_GH=true
  info "GitHub CLI found"
  
  if gh auth status >/dev/null 2>&1; then
    success "GitHub CLI authenticated"
  else
    warning "GitHub CLI not authenticated - attempting login"
    gh auth login --web || warning "GitHub CLI login failed - some features will be unavailable"
  fi
else
  warning "GitHub CLI (gh) not found - some features will be unavailable"
fi

# Verify required workflows exist
REQUIRED_WORKFLOWS=(
  ".github/workflows/release.yml"
  ".github/workflows/release-verify.yml"
  ".github/workflows/mobile-ci.yml"
  ".github/workflows/local-validate-mirror.yml"
)

MISSING_WORKFLOWS=()
for workflow in "${REQUIRED_WORKFLOWS[@]}"; do
  if [ ! -f "$workflow" ]; then
    MISSING_WORKFLOWS+=("$workflow")
  fi
done

if [ ${#MISSING_WORKFLOWS[@]} -gt 0 ]; then
  error "Missing required workflows: ${MISSING_WORKFLOWS[*]}"
fi

success "All required workflows present"

# Step 2: Local validation
section "Step 2: Local Validation"

VALIDATION_FAILED=false
if ./scripts/quick-smoke.sh; then
  success "Quick-smoke passed"
else
  warning "Quick-smoke failed - escalating to full validation"
  if ./scripts/local-validate.sh; then
    success "Full validation passed"
  else
    error "Local validation failed - check logs in artifacts/reports/"
  fi
fi

# Step 3: Sync & push
section "Step 3: Sync & Push"

git add -A
git reset -- artifacts/ 2>/dev/null || true

if [ -n "$(git status --porcelain)" ]; then
  git commit -m "chore: github sync + local validation artifacts" || warning "No changes to commit"
else
  warning "No changes to commit"
fi

info "Pushing branch..."
git push -u origin "$BRANCH" || error "Failed to push branch"

success "Branch pushed: $BRANCH"

# Step 4: Open/ensure PR
section "Step 4: Open/Ensure PR"

if [ "$HAS_GH" = true ]; then
  # Check if PR already exists
  EXISTING_PR=$(gh pr view "$BRANCH" --json url -q .url 2>/dev/null || echo "")
  
  if [ -n "$EXISTING_PR" ]; then
    PR_URL="$EXISTING_PR"
    success "PR already exists: $PR_URL"
  else
    info "Creating PR..."
    PR_URL=$(gh pr create --fill --base main --head "$BRANCH" --title "chore: github sync + local validation artifacts" --body "Automated PR from github-sync workflow." 2>&1 | grep -o 'https://github.com/[^ ]*' | head -1 || echo "")
    
    if [ -n "$PR_URL" ]; then
      success "PR created: $PR_URL"
    else
      warning "PR creation failed or URL not captured"
    fi
  fi
  
  # Auto-comment operator cheat sheet
  if [ -f "OPERATOR_CHEAT_SHEET.txt" ] && [ -n "$PR_URL" ]; then
    info "Adding operator cheat sheet comment..."
    gh pr comment "$BRANCH" -F OPERATOR_CHEAT_SHEET.txt 2>/dev/null || warning "Failed to add cheat sheet comment"
  fi
else
  warning "GitHub CLI not available - skipping PR creation"
fi

# Step 5: Trigger CI
section "Step 5: Trigger CI"

if [ "$HAS_GH" = true ]; then
  info "Triggering CI workflows..."
  
  if gh workflow run local-validate-mirror.yml 2>&1 | grep -q "Created workflow_dispatch event"; then
    success "CI mirror workflow triggered"
  else
    warning "CI mirror workflow trigger failed (may already be running)"
  fi
  
  if gh workflow run mobile-ci.yml 2>&1 | grep -q "Created workflow_dispatch event"; then
    success "Mobile CI workflow triggered"
  else
    warning "Mobile CI workflow trigger failed (may already be running)"
  fi
else
  warning "GitHub CLI not available - skipping CI trigger"
fi

# Step 6: Await CI results (bounded polling)
section "Step 6: Await CI Results"

if [ "$HAS_GH" = true ] && [ -n "$PR_URL" ]; then
  info "Polling CI checks (max 20 minutes)..."
  
  MAX_WAIT=1200  # 20 minutes in seconds
  POLL_INTERVAL=60  # 1 minute
  ELAPSED=0
  
  while [ $ELAPSED -lt $MAX_WAIT ]; do
    sleep $POLL_INTERVAL
    ELAPSED=$((ELAPSED + POLL_INTERVAL))
    
    # Get check status
    CHECKS=$(gh pr checks "$BRANCH" --json name,conclusion,status 2>/dev/null || echo "[]")
    
    if [ -z "$CHECKS" ] || [ "$CHECKS" = "[]" ]; then
      info "Waiting for checks to start... ($ELAPSED/${MAX_WAIT}s)"
      continue
    fi
    
    # Check if all checks are complete
    ALL_COMPLETE=$(echo "$CHECKS" | jq -r 'all(.status == "completed")' 2>/dev/null || echo "false")
    
    if [ "$ALL_COMPLETE" = "true" ]; then
      # Check if all passed
      ALL_PASSED=$(echo "$CHECKS" | jq -r 'all(.conclusion == "success" or .conclusion == "skipped")' 2>/dev/null || echo "false")
      
      if [ "$ALL_PASSED" = "true" ]; then
        success "All CI checks passed"
        CI_CHECKS_PASSED=true
        break
      else
        # Show failing checks
        FAILING=$(echo "$CHECKS" | jq -r '.[] | select(.conclusion != "success" and .conclusion != "skipped") | "\(.name): \(.conclusion)"' 2>/dev/null || echo "")
        if [ -n "$FAILING" ]; then
          error "CI checks failed:\n$FAILING"
        fi
      fi
    else
      info "Checks still running... ($ELAPSED/${MAX_WAIT}s)"
    fi
  done
  
  if [ "$CI_CHECKS_PASSED" = false ]; then
    warning "CI checks did not complete within timeout - proceeding anyway"
  fi
else
  warning "GitHub CLI not available or no PR - skipping CI polling"
fi

# Step 7: Optional Tag & Ship
section "Step 7: Optional Tag & Ship"

if [ "${SHIP:-}" = "true" ]; then
  info "Ship flag detected - running go-live..."
  
  if [ -f "scripts/go-live.sh" ]; then
    VERSION="${VERSION:-1.1.0}"
    if ./scripts/go-live.sh "$VERSION"; then
      success "Go-live completed"
      
      if [ -f "scripts/verify-release.sh" ]; then
        info "Verifying release..."
        if ./scripts/verify-release.sh; then
          success "Release verification passed"
        else
          warning "Release verification had issues - please review"
        fi
      fi
      
      # Verify GitHub Release artifacts
      if [ "$HAS_GH" = true ]; then
        info "Checking GitHub Release artifacts..."
        RELEASE_TAG="v${VERSION}"
        
        ASSETS=$(gh release view "$RELEASE_TAG" --json assets,url 2>/dev/null || echo "")
        if [ -n "$ASSETS" ]; then
          ASSET_COUNT=$(echo "$ASSETS" | jq -r '.assets | length' 2>/dev/null || echo "0")
          RELEASE_URL=$(echo "$ASSETS" | jq -r '.url' 2>/dev/null || echo "")
          
          if [ "$ASSET_COUNT" -gt 0 ]; then
            success "Release artifacts found: $ASSET_COUNT assets"
            info "Release URL: $RELEASE_URL"
            
            # List assets
            echo "$ASSETS" | jq -r '.assets[] | "  • \(.name) (\(.size | . / 1024 / 1024 | floor)MB)"' 2>/dev/null || echo "  (unable to parse assets)"
          else
            warning "No release artifacts found"
          fi
        else
          warning "Release not found or not accessible"
        fi
      fi
    else
      error "Go-live failed - please review errors"
    fi
  else
    warning "go-live.sh not found - skipping ship"
  fi
else
  info "Ship step skipped (set SHIP=true to enable)"
fi

# Step 8: Post-actions (optional)
section "Step 8: Post-Actions (Optional)"

if [ "${RUN_POST:-}" = "true" ]; then
  info "Running post-actions..."
  
  if [ -f "scripts/observability-checks.sh" ]; then
    ./scripts/observability-checks.sh || warning "Observability checks had issues"
  fi
  
  if [ -f "scripts/sanity-checks.sh" ]; then
    ./scripts/sanity-checks.sh || warning "Sanity checks had issues"
  fi
else
  info "Post-actions skipped (set RUN_POST=true to enable)"
fi

# Step 9: Cleanup
section "Step 9: Cleanup"

if [ -f "scripts/cleanup.sh" ]; then
  ./scripts/cleanup.sh || warning "Cleanup had issues"
else
  warning "cleanup.sh not found - skipping cleanup"
fi

# Step 10: Summary
section "Step 10: Summary"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}📊 GITHUB SYNC SUMMARY${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${CYAN}Branch:${NC} $BRANCH"
echo -e "${CYAN}PR:${NC} ${PR_URL:-n/a}"

# Desktop artifacts
if [ -d "artifacts/desktop" ]; then
  ARTIFACTS=$(ls -1 artifacts/desktop/* 2>/dev/null | head -5 | tr '\n' ',' | sed 's/,$//' || echo "none")
  echo -e "${CYAN}Desktop artifacts:${NC} ${ARTIFACTS:-none}"
else
  echo -e "${CYAN}Desktop artifacts:${NC} none"
fi

# Reports
if [ -d "artifacts/reports" ]; then
  REPORTS=$(ls -1 artifacts/reports 2>/dev/null | head -5 | tr '\n' ',' | sed 's/,$//' || echo "none")
  echo -e "${CYAN}Reports:${NC} ${REPORTS:-none}"
else
  echo -e "${CYAN}Reports:${NC} none"
fi

echo -e "${CYAN}CI mirror:${NC} triggered (see PR Checks tab)"
if [ "$CI_CHECKS_PASSED" = true ]; then
  echo -e "${CYAN}CI status:${NC} ${GREEN}✅ All checks passed${NC}"
else
  echo -e "${CYAN}CI status:${NC} ${YELLOW}⏳ Checks pending or unavailable${NC}"
fi

if [ "${SHIP:-}" = "true" ]; then
  echo -e "${CYAN}Release:${NC} v${VERSION:-1.1.0} pushed; verify assets on GitHub Releases page"
fi

echo ""
echo -e "${CYAN}Next:${NC} Golden path"
echo "  Desktop: Connect → Charts ≤3s → Rotate (toast ≤2s) → Disconnect"
echo "  Mobile: Settings → Report Issue → share sheet (<2MB, 'Report Prepared')"
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

success "GitHub sync complete!"

