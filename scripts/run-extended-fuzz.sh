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
MINS="${3:-30}"  # Minutes for --mins flag
ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/fuzz}"

mkdir -p "$ARTIFACT_DIR"
mkdir -p "$ARTIFACT_DIR/corpus"
mkdir -p "$ARTIFACT_DIR/crashers"

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
        
        FUZZ_START=$(date +%s)
        cargo +nightly fuzz run "$target" \
            -- -max_total_time="$DURATION" \
            -artifact_prefix="$ARTIFACT_DIR/crashers/" \
            -print_final_stats=1 \
            2>&1 | tee "$ARTIFACT_DIR/fuzz-${target}-${DURATION}s.log" || {
            echo "⚠️ Fuzz target $target encountered issues"
            continue
        }
        FUZZ_DURATION=$(($(date +%s) - FUZZ_START))
        
        # Record CPU time and corpus growth
        CORPUS_SIZE=$(find "fuzz/corpus/$target" -type f 2>/dev/null | wc -l | tr -d ' ')
        CRASHES=$(find "$ARTIFACT_DIR/crashers" -name "crash-*" 2>/dev/null | wc -l | tr -d ' ')
        
        cat >> "$ARTIFACT_DIR/fuzz-metrics.json" << EOF
{
  "target": "$target",
  "duration_seconds": $FUZZ_DURATION,
  "corpus_size": $CORPUS_SIZE,
  "crashes": $CRASHES,
  "status": "complete"
}
EOF
        
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

