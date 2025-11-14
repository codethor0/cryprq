#!/bin/bash
# Rate-limit-safe script to delete GitHub Actions artifacts in batches
# Usage: ./cleanup_artifacts_batch.sh

set -e

REPO="codethor0/cryprq"
BATCH_SIZE=50
MIN_RATE_LIMIT=10

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🗑️  CLEANING GITHUB ACTIONS ARTIFACTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

check_rate_limit() {
    REMAINING=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0")
    if [ "$REMAINING" -lt "$MIN_RATE_LIMIT" ]; then
        RESET=$(gh api rate_limit --jq '.rate.reset' 2>/dev/null || echo "0")
        NOW=$(date +%s)
        WAIT=$((RESET - NOW + 5))
        if [ "$WAIT" -gt 0 ] && [ "$WAIT" -lt 3600 ]; then
            echo "⚠️  Rate limit low ($REMAINING remaining). Waiting $WAIT seconds..."
            echo "   Run this script again later, or wait for the rate limit to reset."
            exit 0
        fi
        return 1
    fi
    return 0
}

TOTAL_DELETED=0
ITERATION=0

while true; do
    ITERATION=$((ITERATION + 1))
    
    # Check rate limit before each batch
    if ! check_rate_limit; then
        echo "⚠️  Rate limit exhausted. Stopping."
        echo "   Deleted $TOTAL_DELETED artifacts so far."
        echo "   Run this script again later to continue."
        exit 0
    fi
    
    echo "[Batch $ITERATION] Fetching up to $BATCH_SIZE artifacts..."
    
    # Fetch artifact IDs (limit to BATCH_SIZE)
    ARTIFACT_IDS=$(gh api repos/$REPO/actions/artifacts --paginate -q ".artifacts[:$BATCH_SIZE] | .[].id" 2>/dev/null || echo "")
    
    if [ -z "$ARTIFACT_IDS" ] || [ "$(echo "$ARTIFACT_IDS" | grep -v '^$' | wc -l | tr -d ' ')" -eq 0 ]; then
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "✅ ALL ARTIFACTS DELETED"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "   Total deleted: $TOTAL_DELETED artifacts"
        exit 0
    fi
    
    COUNT=$(echo "$ARTIFACT_IDS" | grep -v '^$' | wc -l | tr -d ' ')
    echo "   Found $COUNT artifacts. Deleting..."
    
    DELETED_IN_BATCH=0
    echo "$ARTIFACT_IDS" | while read -r artifact_id; do
        if [ -n "$artifact_id" ]; then
            if gh api -X DELETE repos/$REPO/actions/artifacts/$artifact_id 2>&1 > /dev/null; then
                echo -n "."
                DELETED_IN_BATCH=$((DELETED_IN_BATCH + 1))
            else
                echo -n "x"
            fi
            sleep 1
        fi
    done
    
    echo ""
    TOTAL_DELETED=$((TOTAL_DELETED + COUNT))
    echo "   ✅ Deleted $COUNT artifacts (Total: $TOTAL_DELETED)"
    echo ""
    
    # Small delay between batches
    sleep 2
done
