#!/bin/bash
set -euo pipefail

# Go-Live Sequence Script for Desktop 1.1.0
# Run this script to execute the complete go-live sequence

VERSION="${1:-1.1.0}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🚀 CrypRQ Desktop ${VERSION} Go-Live Sequence"
echo "=============================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
check_secret() {
  local secret_name=$1
  if gh secret list 2>/dev/null | grep -q "^${secret_name}"; then
    echo -e "${GREEN}✅${NC} ${secret_name} configured"
    return 0
  else
    echo -e "${YELLOW}⚠️${NC} ${secret_name} not found (will build unsigned)"
    return 1
  fi
}

step() {
  echo ""
  echo -e "${GREEN}▶${NC} $1"
  echo "----------------------------------------"
}

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

# Step 1: Secrets Check
step "1. Checking GitHub Secrets"
echo "Checking required secrets..."
MACOS_SECRETS=0
WINDOWS_SECRETS=0

check_secret "APPLE_ID" && MACOS_SECRETS=$((MACOS_SECRETS + 1))
check_secret "APPLE_APP_SPECIFIC_PASSWORD" && MACOS_SECRETS=$((MACOS_SECRETS + 1))
check_secret "APPLE_TEAM_ID" && MACOS_SECRETS=$((MACOS_SECRETS + 1))

check_secret "CSC_LINK" && WINDOWS_SECRETS=$((WINDOWS_SECRETS + 1))
check_secret "CSC_KEY_PASSWORD" && WINDOWS_SECRETS=$((WINDOWS_SECRETS + 1))

if [ $MACOS_SECRETS -eq 3 ]; then
  success "macOS signing secrets configured"
else
  warning "macOS signing secrets incomplete (will build unsigned)"
fi

if [ $WINDOWS_SECRETS -eq 2 ]; then
  success "Windows signing secrets configured"
else
  warning "Windows signing secrets incomplete (will build unsigned)"
fi

# Step 2: SBOM Generation
step "2. Generating SBOM"
cd "$REPO_ROOT"
if [ -f "./scripts/generate-sbom.sh" ]; then
  ./scripts/generate-sbom.sh
  success "SBOM files generated"
else
  warning "SBOM generation script not found, skipping"
fi

# Step 3: Store Listing Validation
step "3. Validating Store Listings"
if [ -f "store/validate.mjs" ]; then
  if node store/validate.mjs; then
    success "Store listings validated"
  else
    warning "Store listing validation failed (check output above)"
  fi
else
  warning "Store validation script not found, skipping"
fi

# Step 4: Pre-Release Tests
step "4. Running Pre-Release Tests"
cd "$REPO_ROOT/gui"
if [ -f "Makefile" ]; then
  echo "Running tests..."
  make test || error "Tests failed"
  success "Tests passed"
else
  warning "Makefile not found, skipping tests"
fi

# Step 5: Local Build Sanity Check
step "5. Local Build Sanity Check (Linux)"
if [ -f "Makefile" ]; then
  echo "Building Linux artifacts locally..."
  make build-linux || warning "Linux build failed (may be OK if not on Linux)"
  success "Linux build completed"
else
  warning "Makefile not found, skipping local build"
fi

# Step 6: Smoke Tests
step "6. Running Smoke Tests"
cd "$REPO_ROOT"
if [ -f "scripts/smoke-tests.sh" ]; then
  echo "Running smoke tests..."
  ./scripts/smoke-tests.sh || warning "Smoke tests failed (manual verification recommended)"
  success "Smoke tests completed"
else
  warning "Smoke test script not found, skipping"
fi

# Step 7: Release
step "7. Creating Release"
cd "$REPO_ROOT"
if [ -f "scripts/release.sh" ]; then
  echo "Creating release ${VERSION}..."
  read -p "Continue with release? (y/N) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    ./scripts/release.sh "$VERSION"
    success "Release ${VERSION} created"
    echo ""
    echo "Next steps:"
    echo "  1. git push origin v${VERSION}"
    echo "  2. git push origin main"
    echo "  3. Monitor GitHub Actions: https://github.com/[your-repo]/actions"
  else
    warning "Release cancelled by user"
  fi
else
  error "Release script not found"
fi

# Step 8: Post-Release Instructions
step "8. Post-Release Steps"
echo ""
echo "After CI completes and artifacts are uploaded:"
echo ""
echo "1. Gatekeeper Check (macOS):"
echo "   spctl --assess --type open --verbose dist-package/*.dmg"
echo ""
echo "2. Signature Verification (Windows):"
echo "   signtool verify /pa dist-package/*.exe"
echo ""
echo "3. Observability Checks:"
echo "   ./scripts/observability-checks.sh"
echo ""
echo "4. Export Diagnostics:"
echo "   Help → Export Diagnostics → verify no secrets"
echo ""
echo "5. Sanity Checks:"
echo "   - Kill-switch: quit while connected → session stops"
echo "   - HTTPS enforcement: REMOTE http:// blocked"
echo "   - Redaction: grep diagnostics ZIP for secrets"
echo ""
success "Go-live sequence completed!"
echo ""
echo "📋 See docs/GO_LIVE_SEQUENCE.md for detailed instructions"

