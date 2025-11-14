#!/bin/bash
# Delete all GitHub Actions workflow runs
# Handles rate limits automatically with delays

REPO="codethor0/cryprq"
BATCH_SIZE=30
DELAY=3
MAX_RETRIES=3

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🗑️  Deleting all workflow runs for $REPO"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Function to check rate limit
check_rate_limit() {
  gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0"
}

# Function to delete batch with retry
delete_batch() {
  local batch_num=$1
  local retries=0
  
  while [ $retries -lt $MAX_RETRIES ]; do
    local remaining=$(check_rate_limit)
    
    if [ "$remaining" -lt 100 ]; then
      echo "⚠️  Rate limit low ($remaining remaining). Waiting 60 seconds..."
      sleep 60
      continue
    fi
    
    local count=$(gh run list --limit 1 --json databaseId --jq '. | length' 2>/dev/null || echo "0")
    
    if [ "$count" -eq 0 ]; then
      return 0
    fi
    
    echo -n "Batch $batch_num: "
    local deleted=0
    gh run list --limit $BATCH_SIZE --json databaseId --jq '.[].databaseId' 2>/dev/null | while read -r run_id; do
      if gh run delete "$run_id" 2>&1 | grep -q "submitted\|deleted"; then
        echo -n "."
        deleted=$((deleted + 1))
      else
        echo -n "x"
      fi
    done
    
    echo " ($deleted deleted)"
    sleep $DELAY
    return 0
  done
  
  echo "⚠️  Max retries reached. Rate limit may be exceeded."
  return 1
}

# Main deletion loop
batch=1
while true; do
  COUNT=$(gh run list --limit 1 --json databaseId --jq '. | length' 2>/dev/null || echo "0")
  
  if [ "$COUNT" -eq 0 ]; then
    echo ""
    echo "✅ All workflow runs deleted!"
    break
  fi
  
  TOTAL=$(gh run list --limit 1000 --json databaseId --jq '. | length' 2>/dev/null || echo "?")
  echo "Remaining runs: ~$TOTAL"
  
  if ! delete_batch $batch; then
    echo ""
    echo "⚠️  Rate limit exceeded. Please wait and run this script again later."
    echo "   Or continue manually via: https://github.com/$REPO/actions"
    exit 1
  fi
  
  batch=$((batch + 1))
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Cleanup complete! All workflow runs deleted."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
