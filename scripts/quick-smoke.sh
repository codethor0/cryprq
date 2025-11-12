#!/bin/bash
set -euo pipefail

# Quick-Smoke: Fast local sanity check
# Run Docker + GUI + builds and push a branch with summary

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

echo "🧪 Quick-Smoke: Fast Local Sanity Check"
echo "========================================"
echo ""

# 1) Try local-validate.sh first
if [ -f "scripts/local-validate.sh" ]; then
  info "Running full local validation..."
  if ./scripts/local-validate.sh; then
    success "Full validation passed!"
    exit 0
  else
    warning "Full validation failed, falling back to minimal checks..."
  fi
fi

# 2) Minimal fallback
echo ""
echo -e "${BLUE}▶${NC} Minimal Fallback: Docker + GUI + Builds"
echo "----------------------------------------"

cd gui

# Start fake backend
info "Starting fake backend..."
docker compose -f docker-compose.yml up -d fake-cryprq || error "Failed to start fake backend"

info "Waiting for fake backend..."
timeout 30 bash -c 'until curl -sf http://localhost:9464/metrics >/dev/null; do sleep 1; done' || error "Fake backend not ready"

success "Fake backend ready"

# Install & test
info "Installing dependencies..."
npm ci || error "npm ci failed"

info "Running lint..."
npm run lint || error "Lint failed"

info "Running typecheck..."
npm run typecheck || error "Typecheck failed"

info "Running unit tests..."
npm run test:unit || error "Unit tests failed"

info "Running E2E tests..."
npm run test:playwright || error "E2E tests failed"

success "All tests passed"

# Build artifacts
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
info "Building artifacts for platform: $PLATFORM"

if [[ "$PLATFORM" == "linux" ]]; then
  make build-linux || error "Linux build failed"
elif [[ "$PLATFORM" == "darwin" ]]; then
  npm run build:mac || error "macOS build failed"
elif [[ "$PLATFORM" == *"mingw"* ]] || [[ "$PLATFORM" == *"msys"* ]]; then
  npm run build:win || error "Windows build failed"
else
  error "Unknown platform: $PLATFORM"
fi

success "Artifacts built"

cd ..

# 3) Artifacts & reports snapshot
echo ""
echo -e "${BLUE}▶${NC} Artifacts & Reports Snapshot"
echo "----------------------------------------"

mkdir -p artifacts/desktop artifacts/reports

if [ -d "gui/dist-package" ]; then
  PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
  mkdir -p "artifacts/desktop/$PLATFORM"
  cp -r gui/dist-package/* "artifacts/desktop/$PLATFORM/" 2>/dev/null || true
fi

info "Desktop artifacts:"
ls -la artifacts/desktop/* 2>/dev/null || warning "No desktop artifacts found"

info "Reports:"
ls -la artifacts/reports 2>/dev/null || warning "No reports found"

# 4) Push branch & PR
echo ""
echo -e "${BLUE}▶${NC} Git: Push Branch & PR"
echo "----------------------------------------"

BRANCH=$(git rev-parse --abbrev-ref HEAD)

if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "master" ]; then
  BRANCH="chore/local-validate-$(date +%Y%m%d-%H%M)"
  info "Creating feature branch: $BRANCH"
  git checkout -b "$BRANCH" || error "Failed to create branch"
fi

# Stage changes (excluding artifacts)
git add -A
git reset -- artifacts/ 2>/dev/null || true

if [ -n "$(git status --porcelain)" ]; then
  git commit -m "chore: local validation + desktop artifacts" || warning "No changes to commit"
else
  warning "No changes to commit"
fi

info "Pushing branch..."
git push -u origin "$BRANCH" || error "Failed to push branch"

success "Branch pushed: $BRANCH"

# Create PR if gh available
if command -v gh >/dev/null; then
  info "Creating PR..."
  gh pr create --fill --base main --title "chore: local validation + desktop artifacts" --body "Quick-smoke validation run." 2>/dev/null || warning "PR creation failed or already exists"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 QUICK-SMOKE SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Docker fake backend: Reachable on :9464"
echo "✅ Tests: Lint, typecheck, unit, E2E - All PASS"
echo "✅ Artifacts: Built for $PLATFORM"
echo "✅ Branch: $BRANCH pushed"
echo ""
echo "📦 Artifacts: artifacts/desktop/$PLATFORM/"
echo "📋 Reports: artifacts/reports/"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

success "Quick-smoke complete!"

