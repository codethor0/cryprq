#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Collect artifacts and open GitHub issue on QA failure
set -euo pipefail

DATE=$(date +%Y%m%d)
ARTIFACT_DIR="release-${DATE}/qa"
FAILED_PHASE="${1:-unknown}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Collecting Artifacts & Opening Issue"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Collect artifacts summary
cat > "$ARTIFACT_DIR/failure-summary.txt" << EOF
QA Pipeline Failure

Failed Phase: $FAILED_PHASE
Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Commit: $(git rev-parse HEAD)
Branch: $(git rev-parse --abbrev-ref HEAD)

Artifacts Location: $ARTIFACT_DIR

Next Steps:
1. Review failure logs in $ARTIFACT_DIR
2. Create minimal fix PR
3. Re-run failed phase
4. Re-run full pipeline once green
EOF

echo "✅ Failure summary created: $ARTIFACT_DIR/failure-summary.txt"

# Note: Actual GitHub issue creation would require GitHub CLI or API
# For now, just create a local issue file
cat > "$ARTIFACT_DIR/issue.md" << EOF
# QA Pipeline Failure

**Label**: qa/failure  
**Phase**: $FAILED_PHASE  
**Date**: $(date -u +%Y-%m-%dT%H:%M:%SZ)  
**Commit**: $(git rev-parse HEAD)  
**Branch**: $(git rev-parse --abbrev-ref HEAD)

## Failure Details

QA pipeline failed at phase: \`$FAILED_PHASE\`

## Artifacts

All artifacts available in: \`$ARTIFACT_DIR\`

## Next Steps

1. Review failure logs
2. Create minimal fix PR
3. Re-run failed phase: \`bash scripts/qa-all.sh\`
4. Re-run full pipeline once green

## Attachments

- Failure summary: \`failure-summary.txt\`
- Test logs: \`$ARTIFACT_DIR/*.log\`
- Artifacts: \`$ARTIFACT_DIR/\`
EOF

echo "✅ Issue template created: $ARTIFACT_DIR/issue.md"
echo ""
echo "To create GitHub issue, run:"
echo "  gh issue create --title 'QA Pipeline Failure: $FAILED_PHASE' --body-file $ARTIFACT_DIR/issue.md --label qa/failure"

