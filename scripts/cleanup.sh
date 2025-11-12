#!/bin/bash
# Cleanup & Quick Ops
# Stop Docker services, kill stray processes, wipe transient artifacts

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
  echo -e "${GREEN}ℹ️${NC} $1"
}

warning() {
  echo -e "${YELLOW}⚠️${NC} $1"
}

echo "🧼 Cleanup & Quick Ops"
echo "======================"
echo ""

# Stop Docker services
info "Stopping Docker services..."
cd gui && docker compose down && cd .. || warning "GUI Docker compose down failed"
cd mobile && docker compose down && cd .. || warning "Mobile Docker compose down failed (may not exist)"

# Kill stray Electron dev processes
info "Killing stray Electron processes..."
pkill -f electron || warning "No Electron processes found"

# Wipe transient artifacts (keeps reports)
info "Cleaning transient artifacts..."
rm -rf gui/node_modules mobile/node_modules gui/.playwright artifacts/desktop/* 2>/dev/null || true

# Keep reports directory structure
mkdir -p artifacts/reports 2>/dev/null || true

info "Cleanup complete!"
echo ""
echo "Removed:"
echo "  • Docker containers (stopped)"
echo "  • Electron dev processes"
echo "  • node_modules directories"
echo "  • Playwright cache"
echo "  • Desktop artifacts"
echo ""
echo "Kept:"
echo "  • artifacts/reports/ (test reports preserved)"

