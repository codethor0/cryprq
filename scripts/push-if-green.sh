#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Only push if CI-green gate (server-side safety)
# Checks that the previous commit by us has green CI before pushing

BRANCH="${1:-$(git rev-parse --abbrev-ref HEAD)}"

if ! command -v gh >/dev/null 2>&1; then
  echo "[push-if-green] gh CLI not installed; skipping" >&2
  exit 0
fi

SHA="$(git rev-parse HEAD)"

# Wait briefly for CI to start
sleep 5

# Poll last few runs for this SHA and branch
for i in {1..12}; do
  st="$(gh run list --limit 20 --branch "$BRANCH" --json headSha,conclusion 2>/dev/null | \
       python3 - <<'PY'
import sys, json, os
try:
    runs = json.load(sys.stdin)
    sha = os.environ.get("SHA")
    ok = [r for r in runs if r.get("headSha") == sha and r.get("conclusion") in ("success", "skipped")]
    print("OK" if ok else "WAIT")
except:
    print("WAIT")
PY
)"

  if [[ "$st" == "OK" ]]; then
    echo "[push-if-green] CI green"
    exit 0
  fi
  
  sleep 10
done

echo "[push-if-green] CI not green in time window; preventing push." >&2
exit 1

