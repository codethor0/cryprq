#!/bin/bash
set -euo pipefail

# Quick validation smoke tests for charts, toast limiter, and allowlist UI
# Run this after starting the app with fake backend

echo "🧪 Charts & UX Validation Smoke Tests"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

echo "Manual validation checklist:"
echo ""
echo "1. Charts Sanity:"
echo "   • Start fake backend: cd gui && npm run e2e:serve-fake"
echo "   • Connect in app → charts should render within ≤3s"
echo "   • Charts should update ~1 Hz (once per second)"
echo "   • Slide smoothing 0 → 0.4 → verify immediate visual damping"
echo ""
success "Charts validation: Manual check required"

echo ""
echo "2. Toast Limiter:"
echo "   • Trigger 5 errors in 2s (e.g., invalid peer connections)"
echo "   • Only 1 toast should show (dev bypass OFF)"
echo "   • Verify rate limit: max 1 error toast per 10s"
echo ""
success "Toast limiter: Manual check required"

echo ""
echo "3. Allowlist UI:"
echo "   • Settings → Security → Manage Allowlist"
echo "   • Add 'api.good.example' → confirm stored + counted"
echo "   • When REMOTE save validation is wired: disallow host until added"
echo ""
success "Allowlist UI: Manual check required"

echo ""
echo "4. Performance Guard:"
echo "   • Metrics ingestion throttled to 1 Hz max"
echo "   • metricsSeries60s should stay at ~60-90 points"
echo "   • No UI jank during updates"
echo ""
success "Performance: Check metrics update rate in DevTools"

echo ""
warning "Note: These are manual checks. Run with app + fake backend running."
echo ""
success "Validation script complete!"

