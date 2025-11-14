#!/bin/bash
# Clean up releases after rate limit resets

set -e

REPO="codethor0/cryprq"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🗑️  DELETING RELEASES (Rate Limit Aware)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

wait_for_rate_limit() {
  while true; do
    REMAINING=$(gh api rate_limit --jq '.rate.remaining' 2>/dev/null || echo "0")
    if [ "$REMAINING" -gt 5 ]; then
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

RELEASES_TO_DELETE=(
  "v0.1.0-alpha.1-245-gcd946df-dirty"
  "v1.0.1"
  "v0.1.0-alpha.1"
)

for release in "${RELEASES_TO_DELETE[@]}"; do
  wait_for_rate_limit
  echo "   Deleting release $release..."
  gh release delete "$release" --yes 2>&1 && echo "   ✅ Deleted $release" || echo "   ⚠️  Failed or already deleted"
  sleep 2
done

echo ""
echo "✅ Release cleanup complete"
