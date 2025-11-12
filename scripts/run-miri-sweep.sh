#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Full Miri sweep - detects undefined behavior
# Usage: ./scripts/run-miri-sweep.sh [package]

set -euo pipefail

PACKAGE="${1:-all}"
LOG_DIR="${LOG_DIR:-release-$(date +%Y%m%d)/qa/miri}"

mkdir -p "$LOG_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Full Miri Sweep - Undefined Behavior Detection"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

rustup +nightly component add miri 2>/dev/null || true

if [ "$PACKAGE" = "all" ]; then
    echo "Running Miri on all packages..."
    cargo +nightly miri test --all 2>&1 | tee "$LOG_DIR/miri-all.log" || {
        echo "⚠️ Miri found issues - check $LOG_DIR/miri-all.log"
        exit 1
    }
else
    echo "Running Miri on package: $PACKAGE"
    cargo +nightly miri test --package "$PACKAGE" 2>&1 | tee "$LOG_DIR/miri-${PACKAGE}.log" || {
        echo "⚠️ Miri found issues in $PACKAGE - check $LOG_DIR/miri-${PACKAGE}.log"
        exit 1
    }
fi

echo ""
echo "✅ Miri sweep complete - no undefined behavior detected"

