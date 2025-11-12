#!/bin/bash
set -euo pipefail

# One-shot: quick-smoke → validate → push/PR → (optional) ship → cleanup
# Orchestrates the entire workflow end-to-end

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

# Preconditions check
section "Preconditions Check"

if ! command -v docker >/dev/null; then
  error "Docker not found - please install Docker"
fi

if ! docker ps >/dev/null 2>&1; then
  error "Docker daemon not running - please start Docker"
fi

if ! command -v node >/dev/null; then
  error "Node.js not found - please install Node.js 18+"
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
  error "Node.js 18+ required (found: $(node -v))"
fi

if ! command -v git >/dev/null; then
  error "Git not found - please install Git"
fi

success "Preconditions OK"

# Step 1: Quick-smoke
section "Step 1: Quick-Smoke (Fast Confidence)"

QUICK_SMOKE_FAILED=false
if ./scripts/quick-smoke.sh; then
  success "Quick-smoke passed"
else
  warning "Quick-smoke failed - escalating to full validation"
  QUICK_SMOKE_FAILED=true
fi

# Step 2: Full validation if quick-smoke failed
if [ "$QUICK_SMOKE_FAILED" = true ]; then
  section "Step 2: Full Local Validation"
  
  if [ -f "scripts/local-validate.sh" ]; then
    if ./scripts/local-validate.sh; then
      success "Full validation passed"
    else
      error "Full validation failed - please fix issues before proceeding"
    fi
  else
    warning "local-validate.sh not found - skipping full validation"
  fi
fi

# Step 3: Summarize local outputs
section "Step 3: Local Outputs Summary"

echo ""
info "Desktop artifacts:"
if [ -d "artifacts/desktop" ]; then
  ls -1 artifacts/desktop/* 2>/dev/null | while read -r artifact; do
    echo "  • $artifact"
  done || echo "  (none)"
else
  echo "  (none)"
fi

echo ""
info "Reports:"
if [ -d "artifacts/reports" ]; then
  ls -1 artifacts/reports 2>/dev/null | while read -r report; do
    echo "  • $report"
  done || echo "  (none)"
else
  echo "  (none)"
fi

# Step 4: Ensure branch + PR (idempotent)
section "Step 4: Branch + PR (Idempotent)"

BRANCH=$(git rev-parse --abbrev-ref HEAD)

if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "master" ]; then
  BRANCH="chore/local-validate-$(date +%Y%m%d-%H%M)"
  info "Creating feature branch: $BRANCH"
  git checkout -b "$BRANCH" || warning "Branch may already exist"
fi

info "Current branch: $BRANCH"

# Stage changes (excluding artifacts)
git add -A
git reset -- artifacts/ 2>/dev/null || true

if [ -n "$(git status --porcelain)" ]; then
  git commit -m "chore: local validation + artifacts + reports" || warning "No changes to commit"
else
  warning "No changes to commit"
fi

info "Pushing branch..."
git push -u origin "$BRANCH" || error "Failed to push branch"

success "Branch pushed: $BRANCH"

# Create PR if gh available
PR_URL=""
if command -v gh >/dev/null; then
  info "Creating PR..."
  PR_URL=$(gh pr create --fill --base main --title "chore: local validation + artifacts + reports" --body "Automated PR from one-shot validation workflow." 2>&1) || {
    # Check if PR already exists
    EXISTING_PR=$(gh pr list --head "$BRANCH" --json url --jq '.[0].url' 2>/dev/null || echo "")
    if [ -n "$EXISTING_PR" ]; then
      PR_URL="$EXISTING_PR"
      info "PR already exists: $PR_URL"
    else
      warning "PR creation failed or already exists"
    fi
  }
  
  if [ -n "$PR_URL" ] && [[ "$PR_URL" == http* ]]; then
    success "PR created: $PR_URL"
  fi
else
  warning "GitHub CLI (gh) not available - skipping PR creation"
fi

# Step 5: CI mirror (optional)
section "Step 5: CI Mirror (Optional)"

if command -v gh >/dev/null; then
  info "Triggering CI mirror workflow..."
  if gh workflow run local-validate-mirror.yml 2>&1 | grep -q "Created workflow_dispatch event"; then
    success "CI mirror workflow triggered"
  else
    warning "CI mirror workflow trigger failed (may already be running)"
  fi
else
  warning "GitHub CLI (gh) not available - skipping CI mirror"
fi

# Step 6: Optional ship
section "Step 6: Optional Ship (Guarded)"

SHIP="${SHIP:-false}"
if [ "$SHIP" = "true" ] || [ "${1:-}" = "--ship" ]; then
  if [ -z "$PR_URL" ]; then
    warning "No PR URL found - skipping ship (wait for PR review first)"
  else
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
      else
        error "Go-live failed - please review errors"
      fi
    else
      warning "go-live.sh not found - skipping ship"
    fi
  fi
else
  info "Ship step skipped (set SHIP=true or use --ship flag)"
fi

# Step 7: Post-actions (optional)
section "Step 7: Post-Actions (Optional)"

RUN_POST="${RUN_POST:-false}"
if [ "$RUN_POST" = "true" ] || [ "${1:-}" = "--post" ] || [ "${2:-}" = "--post" ]; then
  if [ -f "scripts/observability-checks.sh" ]; then
    info "Running observability checks..."
    ./scripts/observability-checks.sh || warning "Observability checks had issues"
  fi
  
  if [ -f "scripts/sanity-checks.sh" ]; then
    info "Running sanity checks..."
    ./scripts/sanity-checks.sh || warning "Sanity checks had issues"
  fi
else
  info "Post-actions skipped (set RUN_POST=true or use --post flag)"
fi

# Step 8: Cleanup
section "Step 8: Cleanup"

if [ -f "scripts/cleanup.sh" ]; then
  info "Running cleanup..."
  ./scripts/cleanup.sh || warning "Cleanup had issues"
else
  warning "cleanup.sh not found - skipping cleanup"
fi

# Final summary
section "Summary"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}📊 ONE-SHOT SUMMARY${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${CYAN}Branch:${NC} $BRANCH"
if [ -n "$PR_URL" ] && [[ "$PR_URL" == http* ]]; then
  echo -e "${CYAN}PR:${NC} $PR_URL"
else
  echo -e "${CYAN}PR:${NC} (not created - gh CLI may not be available)"
fi
echo ""

# Artifact paths
info "Artifacts:"
if [ -d "artifacts/desktop" ]; then
  find artifacts/desktop -type f 2>/dev/null | head -5 | while read -r artifact; do
    echo "  • $artifact"
  done || echo "  (none)"
else
  echo "  (none)"
fi

echo ""
info "Reports:"
if [ -d "artifacts/reports" ]; then
  find artifacts/reports -type f 2>/dev/null | head -5 | while read -r report; do
    echo "  • $report"
  done || echo "  (none)"
else
  echo "  (none)"
fi

echo ""
echo -e "${CYAN}Next Steps:${NC}"
if [ -n "$PR_URL" ] && [[ "$PR_URL" == http* ]]; then
  echo "  1. Review PR: $PR_URL"
  echo "  2. Wait for CI to pass"
  echo "  3. Merge PR when ready"
  echo "  4. To ship: SHIP=true ./scripts/one-shot.sh --ship"
else
  echo "  1. Create PR manually: gh pr create --fill --base main"
  echo "  2. Wait for CI to pass"
  echo "  3. Merge PR when ready"
fi

echo ""
echo -e "${CYAN}Golden Path (for PR testing):${NC}"
echo "  Connect → charts ≤3–5s → Rotate (toast ≤2s) → Disconnect"
echo ""
echo -e "${CYAN}Every 2h (first 24h):${NC}"
echo "  ./scripts/observability-checks.sh"
echo "  ./scripts/sanity-checks.sh"
echo ""
echo -e "${CYAN}Rollback:${NC}"
echo "  Unmark desktop 'Latest' release / pause mobile rollout"
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

success "One-shot complete!"

