#!/bin/bash
# Delete all failed workflow runs with rate limit handling

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🗑️  Deleting all failed workflow runs"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Function to check and wait for rate limit
wait_for_rate_limit() {
  local remaining=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0")
  local reset_time=$(gh api rate_limit --jq '.rate.reset' 2>/dev/null || echo "0")
  
  if [ "$remaining" -lt 10 ]; then
    if [ "$reset_time" -gt 0 ]; then
      local current_time=$(date +%s)
      local wait_seconds=$((reset_time - current_time + 5))
      
      if [ "$wait_seconds" -gt 0 ]; then
        echo "⚠️  Rate limit exhausted. Waiting ${wait_seconds} seconds until reset..."
        sleep "$wait_seconds"
      fi
    else
      echo "⚠️  Rate limit low. Waiting 60 seconds..."
      sleep 60
    fi
  fi
}

# Function to delete failed runs
delete_failed_runs() {
  local batch_size=50
  local deleted_total=0
  
  while true; do
    wait_for_rate_limit
    
    # Get failed runs (failure, cancelled, or timed_out)
    local FAILED=$(gh run list --limit 1000 --json databaseId,conclusion,status \
      --jq '.[] | select(.conclusion == "failure" or .conclusion == "cancelled" or .conclusion == "timed_out" or (.status == "completed" and .conclusion == null)) | .databaseId' 2>/dev/null | head -$batch_size)
    
    if [ -z "$FAILED" ]; then
      echo ""
      echo "✅ All failed runs deleted! (Total: $deleted_total)"
      break
    fi
    
    local COUNT=$(echo "$FAILED" | grep -v '^$' | wc -l | tr -d ' ')
    
    if [ "$COUNT" -eq 0 ]; then
      echo ""
      echo "✅ All failed runs deleted! (Total: $deleted_total)"
      break
    fi
    
    echo "Deleting $COUNT failed runs..."
    local deleted_batch=0
    echo "$FAILED" | while read -r run_id; do
      if [ -n "$run_id" ]; then
        if gh run delete "$run_id" 2>/dev/null; then
          echo -n "."
          deleted_batch=$((deleted_batch + 1))
        else
          echo -n "x"
        fi
      fi
    done
    echo ""
    
    deleted_total=$((deleted_total + COUNT))
    sleep 2
  done
  
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "✅ Done! Deleted $deleted_total failed workflow runs."
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

delete_failed_runs
