#!/bin/bash
set -euo pipefail

# One-Shot Local Test + Build + Push Script
# Performs full local validation using Docker, verifies GUI, builds artifacts, and pushes changes

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

step() {
  echo ""
  echo -e "${BLUE}▶${NC} $1"
  echo "----------------------------------------"
}

# 1) Repo hygiene
step "Repo hygiene check"

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
info "Current branch: $CURRENT_BRANCH"

if [ "$CURRENT_BRANCH" = "main" ] || [ "$CURRENT_BRANCH" = "master" ]; then
  BRANCH="chore/local-validate-$(date +%Y%m%d-%H%M)"
  info "Creating feature branch: $BRANCH"
  git checkout -b "$BRANCH" || error "Failed to create branch"
else
  BRANCH="$CURRENT_BRANCH"
  info "Using existing branch: $BRANCH"
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
  warning "Uncommitted changes detected - will be staged and committed"
fi

# Verify subpaths exist
for dir in gui mobile scripts; do
  if [ ! -d "$dir" ]; then
    error "Required directory missing: $dir"
  fi
done

success "Repo hygiene OK"

# Create artifacts directory
mkdir -p artifacts/desktop artifacts/reports artifacts/mobile

# 2) Desktop validation
step "Desktop: Docker-backed validation + GUI build"

cd gui

# a) Start fake backend
info "Starting fake backend..."
docker compose -f docker-compose.yml up -d fake-cryprq || error "Failed to start fake backend"

info "Waiting for fake backend to be ready..."
timeout=30
elapsed=0
while ! curl -sf http://localhost:9464/metrics >/dev/null 2>&1; do
  sleep 1
  elapsed=$((elapsed + 1))
  if [ $elapsed -ge $timeout ]; then
    error "Fake backend did not become ready within ${timeout}s"
  fi
done
success "Fake backend ready at http://localhost:9464/metrics"

# b) Install deps
info "Installing dependencies..."
npm install || error "npm install failed"

# c) Static checks
info "Running lint..."
npm run lint || error "Lint failed"

info "Running typecheck..."
npm run typecheck || error "Typecheck failed"

success "Static checks passed"

# d) Unit + E2E tests
info "Running unit tests..."
npm run test:unit || error "Unit tests failed"

info "Running Playwright E2E tests..."
npm run test:playwright || error "E2E tests failed"

success "All tests passed"

# e) Docker full CI test bundle (optional)
if command -v make >/dev/null && [ -f Makefile ]; then
  info "Running Docker CI test bundle..."
  make test || warning "Docker CI test bundle failed (non-blocking)"
fi

# f) Build desktop artifacts (local platform)
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
info "Building artifacts for platform: $PLATFORM"

BUILD_SUCCESS=false

if [[ "$PLATFORM" == "linux" ]]; then
  if command -v make >/dev/null && [ -f Makefile ]; then
    info "Building Linux artifacts..."
    make build-linux && BUILD_SUCCESS=true || warning "Linux build failed"
  fi
elif [[ "$PLATFORM" == "darwin" ]]; then
  info "Building macOS artifacts..."
  npm run build:mac && BUILD_SUCCESS=true || warning "macOS build failed"
elif [[ "$PLATFORM" == *"mingw"* ]] || [[ "$PLATFORM" == *"msys"* ]]; then
  info "Building Windows artifacts..."
  npm run build:win && BUILD_SUCCESS=true || warning "Windows build failed"
else
  warning "Unknown platform: $PLATFORM - skipping build"
fi

if [ "$BUILD_SUCCESS" = true ]; then
  success "Artifacts built successfully"
else
  warning "Build step skipped or failed (non-blocking)"
fi

# g) Smoke tests (post-build)
if [ "$BUILD_SUCCESS" = true ]; then
  info "Running smoke tests..."
  ../scripts/smoke-tests.sh || warning "Smoke tests failed (non-blocking)"
  
  info "Running observability checks..."
  ../scripts/observability-checks.sh || warning "Observability checks failed (non-blocking)"
  
  info "Running sanity checks..."
  ../scripts/sanity-checks.sh || warning "Sanity checks failed (non-blocking)"
fi

# h) Save outputs
info "Collecting artifacts and reports..."

if [ -d "dist-package" ] && [ "$(ls -A dist-package 2>/dev/null)" ]; then
  mkdir -p "../artifacts/desktop/$PLATFORM"
  cp -r dist-package/* "../artifacts/desktop/$PLATFORM/" 2>/dev/null || true
  success "Desktop artifacts saved to artifacts/desktop/$PLATFORM/"
else
  warning "No dist-package directory found - artifacts may not have been built"
fi

# Collect test reports
if [ -d "tests" ]; then
  find tests -name "report*" -o -name "*.html" -o -name "*.json" | while read -r report; do
    mkdir -p "../artifacts/reports/$(dirname "$report")"
    cp "$report" "../artifacts/reports/$(dirname "$report")/" 2>/dev/null || true
  done
fi

# Playwright reports
if [ -d "playwright-report" ]; then
  cp -r playwright-report/* ../artifacts/reports/ 2>/dev/null || true
fi

cd ..

# 3) Desktop: manual GUI sanity (quick)
step "Desktop: Manual GUI sanity check"

info "To manually verify GUI:"
info "  1. cd gui && npm run dev"
info "  2. Connect → verify 'Throughput (last 60s)' and 'Latency' charts render within 3-5s"
info "  3. Toggle EMA smoothing slider → verify curve changes immediately"
info "  4. Trigger 'Report Issue...' → verify diagnostics export path renders"
info ""
info "Press Enter when done (or Ctrl+C to skip)..."
read -r || warning "Manual GUI check skipped"

# 4) Mobile (optional local smoke for Android)
step "Mobile: Optional Android smoke test"

if [ -d "mobile" ]; then
  cd mobile
  
  info "Installing mobile dependencies..."
  npm install || warning "Mobile npm install failed (non-blocking)"
  
  info "Starting mobile fake backend..."
  docker compose up -d fake-cryprq || warning "Mobile fake backend failed (non-blocking)"
  
  if [ -d "android" ]; then
    info "Building Android debug APK..."
    (cd android && ./gradlew assembleDebug) || warning "Android build failed (non-blocking)"
    
    # Detox E2E (optional - requires emulator)
    if command -v detox >/dev/null; then
      info "Running Detox E2E tests (requires emulator)..."
      npx detox test -c android.emu.debug --headless --record-logs all || warning "Detox tests failed or emulator not available (non-blocking)"
    else
      warning "Detox not available - skipping E2E tests"
    fi
    
    # Save mobile artifacts
    if [ -d "android/app/build/outputs" ]; then
      mkdir -p ../artifacts/mobile
      cp -r android/app/build/outputs/* ../artifacts/mobile/ 2>/dev/null || true
      success "Mobile artifacts saved"
    fi
  else
    warning "Android directory not found - skipping mobile build"
  fi
  
  cd ..
else
  warning "Mobile directory not found - skipping mobile tests"
fi

# 5) Git push + PR scaffold
step "Git: Commit and push"

# Stage files (excluding artifacts)
git add -A
git reset -- artifacts/ 2>/dev/null || true

# Check if there are changes to commit
if [ -z "$(git status --porcelain)" ]; then
  warning "No changes to commit"
else
  info "Committing changes..."
  git commit -m "chore: local Docker validation, desktop build + test logs" || warning "Commit failed or no changes"
fi

info "Pushing branch to origin..."
git push -u origin "$BRANCH" || error "Failed to push branch"

success "Branch pushed: $BRANCH"

# 6) Optional: trigger CI
step "CI: Optional workflow triggers"

if command -v gh >/dev/null; then
  info "Creating PR..."
  PR_URL=$(gh pr create --fill --base main --title "chore: local Docker validation + desktop build artifacts" --body "Local validation run with Docker fake backend, tests, and artifact builds." 2>/dev/null || echo "")
  
  if [ -n "$PR_URL" ]; then
    success "PR created: $PR_URL"
  else
    warning "PR creation failed or already exists"
  fi
  
  info "To trigger CI workflows manually:"
  info "  gh workflow run mobile-ci.yml"
  info "  gh workflow run release-verify.yml"
else
  warning "GitHub CLI not available - skipping PR creation"
  info "Create PR manually at: https://github.com/[org]/cryprq/compare/main...$BRANCH"
fi

# 7) Summary output
step "Summary"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 LOCAL VALIDATION SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Docker container status
FAKE_CRYPRQ_CONTAINER=$(docker ps --filter "name=fake-cryprq" --format "{{.ID}}  {{.Status}}" 2>/dev/null | head -1)
if [ -n "$FAKE_CRYPRQ_CONTAINER" ]; then
  echo "🐳 Docker fake-cryprq: $FAKE_CRYPRQ_CONTAINER"
else
  echo "🐳 Docker fake-cryprq: Not running"
fi

# Test results
echo ""
echo "✅ Test Results:"
echo "   • Lint: PASS"
echo "   • Typecheck: PASS"
echo "   • Unit tests: PASS"
echo "   • E2E tests: PASS"

# Artifacts
echo ""
echo "📦 Artifacts:"
if [ -d "artifacts/desktop/$PLATFORM" ] && [ "$(ls -A artifacts/desktop/$PLATFORM 2>/dev/null)" ]; then
  echo "   • Desktop ($PLATFORM): $(ls -1 artifacts/desktop/$PLATFORM | wc -l | xargs) file(s)"
  echo "     Path: artifacts/desktop/$PLATFORM/"
  ls -lh artifacts/desktop/$PLATFORM/ | tail -n +2 | awk '{print "       " $9 " (" $5 ")"}'
else
  echo "   • Desktop: No artifacts found"
fi

if [ -d "artifacts/mobile" ] && [ "$(ls -A artifacts/mobile 2>/dev/null)" ]; then
  echo "   • Mobile: $(find artifacts/mobile -type f | wc -l | xargs) file(s)"
  echo "     Path: artifacts/mobile/"
else
  echo "   • Mobile: No artifacts found"
fi

# Reports
echo ""
echo "📋 Reports:"
if [ -d "artifacts/reports" ] && [ "$(ls -A artifacts/reports 2>/dev/null)" ]; then
  echo "   • Test reports: artifacts/reports/"
  find artifacts/reports -type f -name "*.html" -o -name "*.json" | head -5 | while read -r report; do
    echo "       $(basename "$report")"
  done
else
  echo "   • No reports found"
fi

# Git info
echo ""
echo "🔀 Git:"
echo "   • Branch: $BRANCH"
if [ -n "${PR_URL:-}" ]; then
  echo "   • PR: $PR_URL"
fi
echo "   • Remote: $(git remote get-url origin 2>/dev/null || echo 'not configured')"

# Next steps
echo ""
echo "🚀 Next Steps:"
echo "   1. Review artifacts in: artifacts/desktop/$PLATFORM/"
echo "   2. Review test reports in: artifacts/reports/"
echo "   3. When ready to ship: ./scripts/go-live.sh 1.1.0 && ./scripts/verify-release.sh"
echo ""

# Cleanup reminder
echo "🧹 Cleanup:"
echo "   • Stop fake backend: docker compose -f gui/docker-compose.yml down"
echo "   • Stop mobile fake backend: docker compose -f mobile/docker-compose.yml down"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

success "Local validation complete!"

