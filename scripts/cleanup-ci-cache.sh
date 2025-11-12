#!/usr/bin/env bash
# Clean up old CI runs and cache to free space
set -euo pipefail

echo "Cleaning up old failed/cancelled runs..."
FAILED_RUNS=$(gh run list --limit 200 --json databaseId,conclusion --jq '.[] | select(.conclusion == "failure" or .conclusion == "cancelled") | .databaseId' | head -100)

count=0
for run_id in $FAILED_RUNS; do
  gh run delete "$run_id" --confirm 2>/dev/null && ((count++)) || true
done
echo "Deleted $count old runs"

echo ""
echo "Cleaning up old cache entries..."
OLD_CACHES=$(gh cache list --limit 200 | grep -E "MiB|GiB" | awk '{print $1}' | head -50)

cache_count=0
for cache_id in $OLD_CACHES; do
  gh cache delete "$cache_id" 2>/dev/null && ((cache_count++)) || true
done
echo "Deleted $cache_count cache entries"
