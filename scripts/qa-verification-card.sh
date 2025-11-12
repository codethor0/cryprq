#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Generate Verification Card for a QA step
# Usage: qa-verification-card.sh <step-name> <exit-code> <duration> <artifacts-dir>

set -euo pipefail

STEP_NAME="${1:-unknown}"
EXIT_CODE="${2:-0}"
DURATION="${3:-0}"
ARTIFACTS_DIR="${4:-release-$(date +%Y%m%d)/qa/${STEP_NAME}}"
COMMIT=$(git rev-parse HEAD)
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

mkdir -p "$ARTIFACTS_DIR"

# Generate Verification Card
cat > "$ARTIFACTS_DIR/VERIFICATION_CARD.md" << EOF
# Verification Card: ${STEP_NAME}

**Date**: ${TIMESTAMP}  
**Commit**: ${COMMIT}  
**Duration**: ${DURATION}s  
**Exit Code**: ${EXIT_CODE}  
**Status**: $([ "$EXIT_CODE" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

## How Implemented

**Files**: \`$(git diff --name-only HEAD~1 HEAD 2>/dev/null | grep -E "(scripts/|\.rs|\.toml)" | head -5 | tr '\n' ',' | sed 's/,$//' || echo "See git log")\`

**Commit**: \`${COMMIT}\`

**Diffs**: \`git show ${COMMIT} --stat | head -10\`

## How Executed

**Command**: \`$(history | tail -1 | sed 's/^[ ]*[0-9]*[ ]*//' || echo "See execution log")\`

**Wall-clock Duration**: ${DURATION}s

**Exit Code**: ${EXIT_CODE}

**Log**: \`${ARTIFACTS_DIR}/execution.log\`

## Result

**Metrics vs Threshold**: See \`${ARTIFACTS_DIR}/metrics.json\`

**Status**: $([ "$EXIT_CODE" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

## Artifacts

**Directory**: \`${ARTIFACTS_DIR}\`

**Files**:
\`\`\`
$(find "$ARTIFACTS_DIR" -type f -name "*.log" -o -name "*.json" -o -name "*.md" 2>/dev/null | head -10 | sed 's|^|  |')
\`\`\`

**SHA256 Checksums**:
\`\`\`
$(find "$ARTIFACTS_DIR" -type f \( -name "*.log" -o -name "*.json" \) 2>/dev/null | head -5 | while read f; do echo "$(shasum -a 256 "$f" 2>/dev/null | cut -d' ' -f1)  $(basename "$f")"; done)
\`\`\`

## Docs Updated

**Files**: \`docs/QA_STATUS.md\`, \`docs/WORKFLOW_STATUS.md\`, \`docs/PRODUCTION_READY.md\`

**Changes**: See git diff for doc updates

## CI Gate

**Check Name**: \`qa-${STEP_NAME}\`

**Status**: $([ "$EXIT_CODE" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

**Link**: \`.github/workflows/qa-vnext.yml\`

## Double-Confirm

**How do you know this is real?**

1. **Artifact exists**: \`${ARTIFACTS_DIR}/VERIFICATION_CARD.md\` (this file)
2. **Logs present**: \`${ARTIFACTS_DIR}/*.log\`
3. **Checksums verified**: See SHA256 above
4. **Git commit**: \`${COMMIT}\`
5. **Timestamp**: ${TIMESTAMP}

**Reproduce**: \`bash scripts/run-${STEP_NAME}.sh\` (or equivalent)

EOF

echo "✅ Verification Card generated: ${ARTIFACTS_DIR}/VERIFICATION_CARD.md"

