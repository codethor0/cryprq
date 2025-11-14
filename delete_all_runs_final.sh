#!/bin/bash
# Final script to delete ALL workflow runs
# Waits for rate limit and deletes everything

REPO="codethor0/cryprq"
BATCH=30
DELAY=5

echo "Deleting ALL workflow runs..."
echo ""

while true; do
  # Check rate limit
  REMAINING=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0")
  RESET=$(gh api rate_limit --jq '.rate.reset' 2>/dev/null || echo "0")
  
  if [ "$REMAINING" -lt 50 ]; then
    NOW=$(date +%s)
    WAIT=$((RESET - NOW + 10))
    if [ $WAIT -gt 0 ]; then
      echo "Rate limit low. Waiting $WAIT seconds..."
      sleep $WAIT
    fi
  fi
  
  # Get count
  COUNT=$(gh run list --limit 1 --json databaseId --jq '. | length' 2>/dev/null || echo "0")
  
  if [ "$COUNT" -eq 0 ]; then
    echo "✅ All runs deleted!"
    break
  fi
  
  TOTAL=$(gh run list --limit 1000 --json databaseId --jq '. | length' 2>/dev/null || echo "?")
  echo "Deleting batch... (~$TOTAL remaining)"
  
  gh run list --limit $BATCH --json databaseId --jq '.[].databaseId' 2>/dev/null | while read -r id; do
    gh run delete "$id" 2>&1 > /dev/null && echo -n "." || echo -n "x"
  done
  
  echo ""
  sleep $DELAY
done
