#!/bin/bash
set -euo pipefail

# Post-Release Verification Script
# Run this after CI completes and artifacts are downloaded

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-dist-package}"

echo "🔍 Post-Release Verification"
echo "============================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

success() {
  echo -e "${GREEN}✅${NC} $1"
}

warning() {
  echo -e "${YELLOW}⚠️${NC} $1"
}

error() {
  echo -e "${RED}❌${NC} $1"
}

# Check if artifacts directory exists
if [ ! -d "$REPO_ROOT/gui/$ARTIFACTS_DIR" ]; then
  error "Artifacts directory not found: $REPO_ROOT/gui/$ARTIFACTS_DIR"
  echo "Download artifacts from GitHub Release first"
  exit 1
fi

cd "$REPO_ROOT/gui/$ARTIFACTS_DIR"

# macOS Gatekeeper Check
echo "Checking macOS DMG..."
DMG_FILE=$(find . -name "*.dmg" -type f | head -1)
if [ -n "$DMG_FILE" ]; then
  if command -v spctl &> /dev/null; then
    if spctl --assess --type open --verbose "$DMG_FILE" 2>&1 | grep -q "accepted"; then
      success "macOS DMG passes Gatekeeper"
    else
      warning "macOS DMG not notarized (expected for dev builds)"
    fi
  else
    warning "spctl not available (not on macOS)"
  fi
else
  warning "No DMG file found"
fi

# Windows Signature Check
echo ""
echo "Checking Windows EXE..."
EXE_FILE=$(find . -name "*.exe" -type f | head -1)
if [ -n "$EXE_FILE" ]; then
  if command -v signtool &> /dev/null; then
    if signtool verify /pa "$EXE_FILE" 2>&1 | grep -q "Successfully verified"; then
      success "Windows EXE is signed"
    else
      warning "Windows EXE not signed (expected for dev builds)"
    fi
  else
    warning "signtool not available (install Windows SDK)"
  fi
else
  warning "No EXE file found"
fi

# Linux Checksums
echo ""
echo "Checking Linux artifacts..."
if find . -name "*.AppImage" -o -name "*.deb" | grep -q .; then
  echo "Generating checksums..."
  sha256sum *.AppImage *.deb 2>/dev/null || true
  success "Linux artifacts found"
else
  warning "No Linux artifacts found"
fi

# Observability Checks
echo ""
echo "Running observability checks..."
cd "$REPO_ROOT"
if [ -f "scripts/observability-checks.sh" ]; then
  ./scripts/observability-checks.sh
else
  warning "Observability check script not found"
fi

# Chart bundle check
echo ""
echo "Checking chart components..."
if grep -R "Throughput (last 60s)" "$REPO_ROOT/gui/dist" 2>/dev/null | head -1 > /dev/null; then
  success "Chart components bundled"
else
  error "Chart bundle missing - charts may not render"
  exit 1
fi

# Security audit (fail only on high/critical)
echo ""
echo "Running npm audit..."
cd "$REPO_ROOT/gui"
if npm audit --omit=dev --audit-level=high 2>&1 | grep -q "found [1-9]"; then
  error "High or critical vulnerabilities found"
  npm audit --omit=dev --audit-level=high
  exit 1
else
  success "No high/critical vulnerabilities in production dependencies"
fi

# License checker (non-blocking)
echo ""
echo "Checking production licenses..."
if command -v npx &> /dev/null; then
  # Try to run license-checker, but don't fail if it's not installed or errors
  if npx license-checker --production --onlyAllow "MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC" 2>&1 | grep -q "unacceptable licenses found"; then
    warning "Some licenses may need review (non-blocking)"
    npx license-checker --production --summary 2>/dev/null || true
  else
    # Check if license-checker ran successfully
    if npx license-checker --production --summary 2>&1 | head -1 | grep -q "license-checker"; then
      success "Production licenses OK"
    else
      warning "license-checker not available - install with: npm install -g license-checker"
    fi
  fi
else
  warning "npx not available - skipping license check"
fi

# Redaction Check (if diagnostics ZIP exists)
echo ""
echo "Checking redaction in diagnostics..."
DIAG_ZIP=$(find ~ -name "cryprq-diagnostics-*.zip" -type f -mtime -1 | head -1)
if [ -n "$DIAG_ZIP" ]; then
  TEMP_DIR=$(mktemp -d)
  unzip -q "$DIAG_ZIP" -d "$TEMP_DIR" 2>/dev/null || true
  if grep -r -E "bearer |privKey=|authorization:" "$TEMP_DIR" 2>/dev/null; then
    error "Secrets found in diagnostics ZIP!"
    rm -rf "$TEMP_DIR"
    exit 1
  else
    success "No secrets found in diagnostics ZIP"
  fi
  rm -rf "$TEMP_DIR"
else
  warning "No recent diagnostics ZIP found (export one to verify)"
fi

echo ""
success "Verification complete!"
echo ""
echo "Next steps:"
echo "  1. Test kill-switch: quit while connected → session stops"
echo "  2. Test HTTPS enforcement: REMOTE http:// blocked"
echo "  3. Export diagnostics: verify no secrets"
echo "  4. Monitor crash reports (if enabled)"

