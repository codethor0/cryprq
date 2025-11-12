#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
have(){ command -v "$1" >/dev/null 2>&1; }

# Prefer markdown-link-check (node). Fallback to lychee (Rust) if installed.
if have npx; then
  shopt -s globstar nullglob
  status=0
  
  # Exclude directories
  for f in "$ROOT"/**/*.md; do
    [[ -f "$f" ]] || continue
    [[ "$f" == *"/node_modules/"* ]] && continue
    [[ "$f" == *"/target/"* ]] && continue
    [[ "$f" == *"/dist/"* ]] && continue
    [[ "$f" == *"/build/"* ]] && continue
    [[ "$f" == *"/vendor/"* ]] && continue
    
    echo "[link-check] $f"
    if [[ -f "${ROOT}/.mlc.json" ]]; then
      npx --yes markdown-link-check -q -c "${ROOT}/.mlc.json" "$f" || status=$?
    else
      npx --yes markdown-link-check -q "$f" || status=$?
    fi
  done
  exit $status
elif have lychee; then
  # lychee checks all links in one shot
  lychee --no-progress --max-concurrency 8 --accept 200,204,301,302 "$ROOT" || exit $?
else
  echo "No link checker found (install Node for markdown-link-check or lychee)." >&2
  exit 2
fi

