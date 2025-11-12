#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Extended fuzz runner - runs each target for 30+ minutes
# Usage: ./scripts/run-extended-fuzz.sh [target] [duration_seconds]

set -euo pipefail

TARGET="${1:-all}"
DURATION="${2:-1800}"  # Default 30 minutes
ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/fuzz}"

mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Extended Fuzz Runner - ${DURATION}s per target"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    echo "Installing cargo-fuzz..."
    cargo install cargo-fuzz --force
fi

TARGETS=("hybrid_handshake" "protocol_parse" "key_rotation" "ppk_derivation")

if [ "$TARGET" = "all" ]; then
    for target in "${TARGETS[@]}"; do
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "Fuzzing: $target (${DURATION}s)"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        cargo +nightly fuzz run "$target" \
            -- -max_total_time="$DURATION" \
            -artifact_prefix="$ARTIFACT_DIR/" \
            -print_final_stats=1 \
            2>&1 | tee "$ARTIFACT_DIR/fuzz-${target}-${DURATION}s.log" || {
            echo "⚠️ Fuzz target $target encountered issues"
            continue
        }
        
        echo ""
        echo "✅ $target fuzzing complete"
        echo ""
    done
else
    echo "Fuzzing: $TARGET (${DURATION}s)"
    cargo +nightly fuzz run "$TARGET" \
        -- -max_total_time="$DURATION" \
        -artifact_prefix="$ARTIFACT_DIR/" \
        -print_final_stats=1 \
        2>&1 | tee "$ARTIFACT_DIR/fuzz-${TARGET}-${DURATION}s.log"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Fuzz Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Artifacts: $ARTIFACT_DIR"
echo "Crashes: $(find "$ARTIFACT_DIR" -name "crash-*" 2>/dev/null | wc -l | tr -d ' ')"
echo "Corpus size: $(find "$ARTIFACT_DIR" -name "corpus" -type d 2>/dev/null | head -1 | xargs -I {} find {} -type f 2>/dev/null | wc -l | tr -d ' ')"
echo ""

