#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Cleanup script for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 Cleaning Up"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Clean Rust build artifacts
echo "🦀 Cleaning Rust build artifacts..."
cargo clean

# Clean Docker containers and images
echo "🐳 Cleaning Docker containers and images..."
docker compose down -v 2>/dev/null || true
docker rm -f cryprq-listener cryprq-dialer test-runner 2>/dev/null || true
docker rmi cryprq-node:latest cryprq-node:test 2>/dev/null || true

# Clean test logs
echo "📝 Cleaning test logs..."
rm -f test-*.log security-audit-*.log performance-*.log compliance-*.log

# Clean temporary files
echo "🗑️  Cleaning temporary files..."
find . -type f -name "*.tmp" -delete
find . -type f -name "*.bak" -delete
find . -type f -name "*~" -delete

# Clean coverage reports
echo "📊 Cleaning coverage reports..."
rm -rf coverage/ tarpaulin-report.html

# Clean mobile build artifacts (optional)
if [ "${CLEAN_MOBILE:-}" = "true" ]; then
    echo "📱 Cleaning mobile build artifacts..."
    rm -rf mobile/android/app/build
    rm -rf mobile/ios/build
    rm -rf mobile/node_modules
fi

# Clean GUI build artifacts (optional)
if [ "${CLEAN_GUI:-}" = "true" ]; then
    echo "🖥️  Cleaning GUI build artifacts..."
    rm -rf gui/dist gui/dist-electron gui/dist-package
    rm -rf gui/node_modules
fi

echo ""
echo "✅ Cleanup completed"
echo ""

exit 0
