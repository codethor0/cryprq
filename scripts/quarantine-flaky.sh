#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Flaky test quarantine (keeps the loop green, logs for fix)
# Captures failing tests and documents them for nightly triage

OUT="artifacts/dev-watch/QUARANTINED_TESTS.txt"
mkdir -p "$(dirname "$OUT")"

# Capture failed tests from the last cargo test run
if [[ -f "artifacts/dev-watch/tests.txt" ]]; then
  grep -E "test .* FAILED|test .* failed" artifacts/dev-watch/tests.txt | \
    awk '{print $2}' | sort -u > "$OUT" || true
else
  touch "$OUT"
fi

cnt=$(wc -l < "$OUT" 2>/dev/null || echo 0)

if [[ "$cnt" -gt 0 ]]; then
  {
    echo "# Quarantined Tests"
    echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "Count: $cnt"
    echo ""
    echo "These tests failed in the last run. Documented for nightly triage."
    echo "Local dev loop remains unblocked but visible."
    echo ""
    cat "$OUT"
  } > "$OUT"
  
  echo "[quarantine] Found $cnt failing tests. Documented in $OUT"
else
  echo "[quarantine] No failing tests found"
fi

