#!/bin/bash
# Helper script to update WEB_VALIDATION_RUN.md after web smoke test
# Usage: ./scripts/update-web-validation.sh WEB-1 PASS "2025-11-14" "testfile.bin" "abc123..." "matches CLI minimal sanity"

set -euo pipefail

TEST_ID="${1:-}"
STATUS="${2:-}"
DATE="${3:-}"
FILE_NAME="${4:-}"
SHA256="${5:-}"
NOTE="${6:-}"

if [ -z "$TEST_ID" ] || [ -z "$STATUS" ]; then
    echo "Usage: $0 TEST_ID STATUS [DATE] [FILE_NAME] [SHA256] [NOTE]"
    echo "Example: $0 WEB-1 PASS \"2025-11-14\" \"testfile.bin\" \"abc123...\" \"matches CLI minimal sanity\""
    exit 1
fi

VALIDATION_FILE="docs/WEB_VALIDATION_RUN.md"

if [ ! -f "$VALIDATION_FILE" ]; then
    echo "Error: $VALIDATION_FILE not found"
    exit 1
fi

# Update status in table
if [ "$TEST_ID" = "WEB-1" ]; then
    sed -i.bak "s/| WEB-1  | Minimal Web Loopback File Transfer   | ☐.*|/| WEB-1  | Minimal Web Loopback File Transfer   | ✅ PASS | $DATE: $NOTE |/" "$VALIDATION_FILE"
    echo "✅ Updated WEB-1 status in table"
elif [ "$TEST_ID" = "WEB-2" ]; then
    sed -i.bak "s/| WEB-2  | Medium File Web Transfer             | ☐.*|/| WEB-2  | Medium File Web Transfer             | ✅ PASS | $DATE: $NOTE |/" "$VALIDATION_FILE"
    echo "✅ Updated WEB-2 status in table"
fi

# Update detailed status section
if [ "$TEST_ID" = "WEB-1" ]; then
    # Find WEB-1 status line and update
    sed -i.bak "s/\*\*Status:\*\* ☐ TODO \/ ✅ PASS \/ ⚠️ PARTIAL \/ ❌ FAIL/\*\*Status:\*\* ✅ PASS/" "$VALIDATION_FILE"
    # Update notes section
    if [ -n "$NOTE" ]; then
        sed -i.bak "/^### WEB-1/,/^---/ s/^\*\*Notes:\*\*$/\*\*Notes:\*\*\n$DATE: $NOTE\n- File: $FILE_NAME\n- SHA-256: $SHA256/" "$VALIDATION_FILE"
    fi
    echo "✅ Updated WEB-1 detailed status"
fi

# Clean up backup
rm -f "${VALIDATION_FILE}.bak"

echo ""
echo "✅ Updated $VALIDATION_FILE for $TEST_ID"
echo "Review the file and commit when ready."

