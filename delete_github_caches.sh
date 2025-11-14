#!/bin/bash
# Delete all GitHub Actions artifacts and caches

set -e

REPO="codethor0/cryprq"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🗑️  DELETING GITHUB CACHES & ARTIFACTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

wait_for_rate_limit() {
  while true; do
    REMAINING=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0")
    if [ "$REMAINING" -gt 10 ]; then
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

# Delete artifacts
echo "1. Deleting GitHub Actions artifacts..."
wait_for_rate_limit

while true; do
  ARTIFACTS=$(gh api repos/$REPO/actions/artifacts --jq '.artifacts[].id' 2>/dev/null || echo "")
  if [ -z "$ARTIFACTS" ]; then
    break
  fi
  
  COUNT=$(echo "$ARTIFACTS" | wc -l | tr -d ' ')
  if [ "$COUNT" -eq 0 ]; then
    break
  fi
  
  echo "   Found $COUNT artifacts. Deleting..."
  echo "$ARTIFACTS" | head -10 | while read -r artifact_id; do
    gh api -X DELETE repos/$REPO/actions/artifacts/$artifact_id 2>&1 > /dev/null && echo -n "." || echo -n "x"
  done
  echo ""
  sleep 2
done

echo "✅ All artifacts deleted!"
echo ""

# Delete caches
echo "2. Deleting GitHub Actions caches..."
wait_for_rate_limit

while true; do
  CACHES=$(gh api repos/$REPO/actions/caches --jq '.actions_caches[].id' 2>/dev/null || echo "")
  if [ -z "$CACHES" ]; then
    break
  fi
  
  COUNT=$(echo "$CACHES" | wc -l | tr -d ' ')
  if [ "$COUNT" -eq 0 ]; then
    break
  fi
  
  echo "   Found $COUNT caches. Deleting..."
  echo "$CACHES" | head -10 | while read -r cache_id; do
    gh api -X DELETE repos/$REPO/actions/caches/$cache_id 2>&1 > /dev/null && echo -n "." || echo -n "x"
  done
  echo ""
  sleep 2
done

echo "✅ All caches deleted!"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ COMPLETE: All GitHub caches and artifacts deleted!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
