#!/bin/bash
# Delete ALL workflow runs (including failed ones)

set -e

REPO="codethor0/cryprq"
BATCH_SIZE=50
DELAY=2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🗑️  DELETING ALL WORKFLOW RUNS (including failed)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

wait_for_rate_limit() {
  while true; do
    REMAINING=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0")
    if [ "$REMAINING" -gt 10 ]; then
      echo "✅ Rate limit OK ($REMAINING remaining)"
      break
    fi
    RESET=$(gh api rate_limit --jq '.rate.reset' 2>/dev/null || echo "0")
    NOW=$(date +%s)
    WAIT=$((RESET - NOW + 5))
    if [ "$WAIT" -gt 0 ]; then
      echo "⏳ Rate limit exhausted. Waiting $WAIT seconds..."
      sleep "$WAIT"
    else
      sleep 5
    fi
  done
}

delete_batch() {
  local batch_num=$1
  local deleted=0
  local failed=0
  
  echo "📦 Batch $batch_num: Fetching runs..."
  
  # Get all runs (prioritize failed/cancelled)
  RUNS=$(gh run list --limit 100 --json databaseId,conclusion --jq '.[] | select(.conclusion == "failure" or .conclusion == "cancelled" or .conclusion == null) | .databaseId' 2>/dev/null || echo "")
  
  if [ -z "$RUNS" ]; then
    # If no failed runs, get all runs
    RUNS=$(gh run list --limit 100 --json databaseId --jq '.[].databaseId' 2>/dev/null || echo "")
  fi
  
  if [ -z "$RUNS" ]; then
    return 1  # No more runs
  fi
  
  echo "$RUNS" | head -$BATCH_SIZE | while read -r run_id; do
    if gh run delete "$run_id" 2>&1 | grep -q "submitted\|deleted"; then
      deleted=$((deleted + 1))
      echo -n "."
    else
      failed=$((failed + 1))
      echo -n "x"
    fi
  done
  
  echo ""
  return 0
}

# Main deletion loop
batch=1
while true; do
  wait_for_rate_limit
  
  TOTAL=$(gh run list --limit 1000 --json databaseId --jq '. | length' 2>/dev/null || echo "0")
  
  if [ "$TOTAL" -eq 0 ]; then
    echo ""
    echo "✅ ALL WORKFLOW RUNS DELETED!"
    break
  fi
  
  echo ""
  echo "📊 Remaining runs: $TOTAL"
  
  if ! delete_batch $batch; then
    echo "✅ No more runs to delete!"
    break
  fi
  
  batch=$((batch + 1))
  sleep $DELAY
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ COMPLETE: All workflow runs deleted!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
