#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Wrapper to run a step and generate verification card
# Usage: qa-wrap-step.sh <step-name> <command> [args...]

set -euo pipefail

STEP_NAME="${1:-unknown}"
shift
COMMAND="$*"
ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa}"
STEP_DIR="$ARTIFACT_DIR/${STEP_NAME}"

mkdir -p "$STEP_DIR"

STEP_START=$(date +%s)

# Run command
if eval "$COMMAND" 2>&1 | tee "$STEP_DIR/execution.log"; then
    EXIT_CODE=0
else
    EXIT_CODE=$?
fi

STEP_DURATION=$(($(date +%s) - STEP_START))

# Generate verification card
bash scripts/qa-verification-card.sh "$STEP_NAME" "$EXIT_CODE" "$STEP_DURATION" "$STEP_DIR"

exit $EXIT_CODE

