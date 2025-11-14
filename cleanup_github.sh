#!/bin/bash
# Fast GitHub cleanup script for codethor0/cryprq

set -e

REPO="codethor0/cryprq"
REPO_PATH=$(echo $REPO | cut -d'/' -f2)

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 FAST GITHUB CLEANUP"
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
    if [ "$WAIT" -gt 0 ] && [ "$WAIT" -lt 3600 ]; then
      echo "⏳ Rate limit low. Waiting $WAIT seconds..."
      sleep "$WAIT"
    else
      sleep 5
    fi
  done
}

# 1. Delete ALL workflow runs
echo "1️⃣  Deleting ALL workflow runs..."
wait_for_rate_limit

DELETED_RUNS=0
while true; do
  RUNS=$(gh api repos/$REPO/actions/runs --paginate -q '.workflow_runs[].id' 2>/dev/null || echo "")
  if [ -z "$RUNS" ]; then
    break
  fi
  
  COUNT=$(echo "$RUNS" | grep -v '^$' | wc -l | tr -d ' ')
  if [ "$COUNT" -eq 0 ]; then
    break
  fi
  
  echo "   Deleting $COUNT runs..."
  echo "$RUNS" | head -20 | while read -r run_id; do
    if [ -n "$run_id" ]; then
      gh api -X DELETE repos/$REPO/actions/runs/$run_id 2>&1 > /dev/null && echo -n "." || echo -n "x"
      DELETED_RUNS=$((DELETED_RUNS + 1))
    fi
  done
  echo ""
  sleep 1
done

echo "✅ Deleted $DELETED_RUNS workflow runs"
echo ""

# 2. Delete ALL artifacts
echo "2️⃣  Deleting ALL artifacts..."
wait_for_rate_limit

DELETED_ARTIFACTS=0
while true; do
  ARTIFACTS=$(gh api repos/$REPO/actions/artifacts --paginate -q '.artifacts[].id' 2>/dev/null || echo "")
  if [ -z "$ARTIFACTS" ]; then
    break
  fi
  
  COUNT=$(echo "$ARTIFACTS" | grep -v '^$' | wc -l | tr -d ' ')
  if [ "$COUNT" -eq 0 ]; then
    break
  fi
  
  echo "   Deleting $COUNT artifacts..."
  echo "$ARTIFACTS" | head -20 | while read -r artifact_id; do
    if [ -n "$artifact_id" ]; then
      gh api -X DELETE repos/$REPO/actions/artifacts/$artifact_id 2>&1 > /dev/null && echo -n "." || echo -n "x"
      DELETED_ARTIFACTS=$((DELETED_ARTIFACTS + 1))
    fi
  done
  echo ""
  sleep 1
done

echo "✅ Deleted $DELETED_ARTIFACTS artifacts"
echo ""

# 3. Delete ALL caches
echo "3️⃣  Deleting ALL caches..."
wait_for_rate_limit

DELETED_CACHES=0
while true; do
  CACHES=$(gh api repos/$REPO/actions/caches --paginate -q '.actions_caches[].id' 2>/dev/null || echo "")
  if [ -z "$CACHES" ]; then
    break
  fi
  
  COUNT=$(echo "$CACHES" | grep -v '^$' | wc -l | tr -d ' ')
  if [ "$COUNT" -eq 0 ]; then
    break
  fi
  
  echo "   Deleting $COUNT caches..."
  echo "$CACHES" | head -20 | while read -r cache_id; do
    if [ -n "$cache_id" ]; then
      gh api -X DELETE repos/$REPO/actions/caches/$cache_id 2>&1 > /dev/null && echo -n "." || echo -n "x"
      DELETED_CACHES=$((DELETED_CACHES + 1))
    fi
  done
  echo ""
  sleep 1
done

echo "✅ Deleted $DELETED_CACHES caches"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ GITHUB CLEANUP COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
