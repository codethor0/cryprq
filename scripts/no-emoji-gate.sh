#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Fail if any Markdown file contains Unicode emoji or :shortcodes:
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

shopt -s globstar nullglob

files=("$ROOT"/**/*.md)

pat_emoji='[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]'
pat_shortcode=':[a-z0-9_+-]+:'

fail=0

for f in "${files[@]}"; do
  [[ -f "$f" ]] || continue
  
  # Skip excluded directories
  [[ "$f" == *"/node_modules/"* ]] && continue
  [[ "$f" == *"/target/"* ]] && continue
  [[ "$f" == *"/dist/"* ]] && continue
  [[ "$f" == *"/build/"* ]] && continue
  [[ "$f" == *"/vendor/"* ]] && continue
  
  # Unicode emojis
  if grep -Pq "$pat_emoji" "$f" 2>/dev/null || grep -qE "$pat_emoji" "$f" 2>/dev/null; then
    echo "NO-EMOJI: $f contains Unicode emoji" >&2
    fail=1
  fi
  
  # :shortcodes:
  if grep -Eq "$pat_shortcode" "$f"; then
    echo "NO-EMOJI: $f contains emoji shortcode" >&2
    fail=1
  fi
done

if [[ $fail -eq 1 ]]; then
  echo "NO-EMOJI: FAILED - Remove emojis/shortcodes from Markdown files" >&2
  exit 1
fi

echo "NO-EMOJI: OK - No emojis or shortcodes found"

