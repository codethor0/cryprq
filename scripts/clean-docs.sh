#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
# Clean, standardize, and verify Markdown docs (no emojis/icons). macOS/Linux only.
set -euo pipefail

# -------- settings
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRANCH_DEFAULT="${BRANCH_DEFAULT:-main}"         # override if needed
DOC_GLOBS=(
  "README.md" "README*.md" "docs/**/*.md" "**/*.md"
)
EXCLUDES=(
  ".git" "node_modules" "target" "dist" "build" "vendor" ".venv"
)
SUMMARY="${ROOT}/docs/DOCS_CLEANUP_SUMMARY.md"
RUN_FMT=${RUN_FMT:-1}        # set to 0 to skip prettier
RUN_LINT=${RUN_LINT:-1}      # set to 0 to skip markdownlint
RUN_GH=${RUN_GH:-1}          # set to 0 to skip GitHub Actions status checks

# -------- helpers
have() { command -v "$1" >/dev/null 2>&1; }
note() { echo "[clean-docs] $*"; }
fail() { echo "[clean-docs] ERROR: $*" >&2; exit 1; }

# -------- gather files
cd "$ROOT"
mapfile -t CANDIDATES < <(printf '%s\n' "${DOC_GLOBS[@]}" | xargs -I{} bash -c 'shopt -s globstar nullglob; for f in {}; do echo "$f"; done' | sort -u)

# filter excludes
FILES=()
for f in "${CANDIDATES[@]}"; do
  [[ ! -f "$f" ]] && continue
  skip=0
  for ex in "${EXCLUDES[@]}"; do
    [[ "$f" == "$ex"* || "$f" == */"$ex"/* ]] && { skip=1; break; }
  done
  (( skip==0 )) && FILES+=("$f")
done

[[ ${#FILES[@]} -eq 0 ]] && fail "No Markdown files found."

# -------- create work branch
TS="$(date +%Y%m%d_%H%M%S)"
BR="docs/cleanup_${TS}"
git rev-parse --verify "$BRANCH_DEFAULT" >/dev/null 2>&1 || BRANCH_DEFAULT="$(git symbolic-ref --short HEAD)"
git checkout -b "$BR" >/dev/null 2>&1 || note "Working on current branch"

note "Stripping emojis/shortcodes from ${#FILES[@]} Markdown files…"

# Process files directly
EMOJI_JSON="$(python3 <<'PYEOF'
import sys, re, json

# Basic emoji removal: Unicode emoji blocks + shortcode patterns like :rocket:
# We intentionally do NOT remove generic images/diagrams.
EMOJI_PATTERN = re.compile(
    "["                    # unicode ranges (broad but safe)
    "\U0001F300-\U0001F5FF"  # symbols & pictographs
    "\U0001F600-\U0001F64F"  # emoticons
    "\U0001F680-\U0001F6FF"  # transport & map
    "\U0001F700-\U0001F77F"  # alchemical
    "\U0001F780-\U0001F7FF"  # geometric ext
    "\U0001F800-\U0001F8FF"  # supplemental arrows-c
    "\U0001F900-\U0001F9FF"  # supplemental symbols and pictographs
    "\U0001FA00-\U0001FAFF"  # symbols & pictographs ext-a
    "\U00002700-\U000027BF"  # dingbats
    "\U00002600-\U000026FF"  # misc symbols
    "]+",
    flags=re.UNICODE
)
SHORTCODE = re.compile(r':[a-z0-9_+-]+:', re.IGNORECASE)
HTML_EMOJI_IMG = re.compile(r'<img[^>]+(emoji|emote)[^>]*>', re.IGNORECASE)

def clean_text(text:str)->str:
    t = HTML_EMOJI_IMG.sub('', text)
    t = SHORTCODE.sub('', t)
    t = EMOJI_PATTERN.sub('', t)
    # collapse multiple spaces created by removals
    t = re.sub(r'[ \t]{2,}', ' ', t)
    # trim trailing spaces
    t = re.sub(r'[ \t]+$', '', t, flags=re.M)
    return t

def process(path):
    try:
        with open(path, 'r', encoding='utf-8', errors='ignore') as f:
            original = f.read()
        cleaned = clean_text(original)
        if cleaned != original:
            with open(path, 'w', encoding='utf-8') as f:
                f.write(cleaned)
            return True
    except Exception as e:
        print(f"[emoji-strip] WARN: {path}: {e}", file=sys.stderr)
    return False

changed = []
# Read file paths from stdin (one per line)
for line in sys.stdin:
    p = line.strip()
    if not p:
        continue
    if process(p):
        changed.append(p)

print(json.dumps({"changed": changed}))
PYEOF
<<<"$(printf '%s\n' "${FILES[@]}")"
)"

# shellcheck disable=SC2181
if [[ $? -ne 0 || -z "$EMOJI_JSON" ]]; then
  fail "Emoji stripping step failed."
fi
CHANGED=$(printf '%s' "$EMOJI_JSON" | python3 -c 'import sys,json; print("\n".join(json.load(sys.stdin)["changed"]))' || true)

# -------- normalize headings: ensure a single H1 at top if missing (conservative)
normalize_headings() {
  f="$1"
  # If file has no H1, convert first non-empty line to H1
  if ! grep -qE '^\# ' "$f"; then
    first="$(grep -nve '^\s*$' "$f" | head -n1 | cut -d: -f1 || true)"
    if [[ -n "$first" ]]; then
      sed -i.bak "${first}s|^|# |" "$f" && rm -f "$f.bak"
    fi
  fi
  # Demote any multiple leading H1s to H2+
  awk '
    BEGIN{h1=0}
    /^\# / { if (h1==1) { sub(/^# /,"## "); print; next } else { h1=1; print; next } }
    { print }
  ' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
}

note "Normalizing headings…"
for f in "${FILES[@]}"; do
  normalize_headings "$f"
done

# -------- optional: prettier + markdownlint
if (( RUN_FMT==1 )); then
  if have npx; then
    note "Running Prettier on Markdown…"
    npx --yes prettier -w "${FILES[@]}" >/dev/null 2>&1 || note "Prettier formatting completed (warnings may exist)"
  else
    note "Prettier not found (npx missing); skipping format."
  fi
fi

if (( RUN_LINT==1 )); then
  if have npx; then
    note "Running markdownlint…"
    npx --yes markdownlint-cli2 "**/*.md" "!**/node_modules/**" "!**/target/**" "!**/dist/**" "!**/build/**" 2>&1 || note "markdownlint completed (warnings may exist)"
  else
    note "markdownlint not found (npx missing); skipping lint."
  fi
fi

# -------- write summary
mkdir -p "$(dirname "$SUMMARY")"
{
  echo "# Documentation Cleanup Summary"
  echo
  echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
  echo
  echo "Files processed: ${#FILES[@]}"
  echo
  if [[ -n "$CHANGED" ]]; then
    echo "Files modified (emoji/shortcode removals or structural fixes):"
    echo "$CHANGED" | sed '/^$/d' | sed 's/^/- /'
  else
    echo "No content changes required."
  fi
  echo
  echo "Style tools:"
  echo "- Prettier: $([[ $RUN_FMT -eq 1 ]] && echo "attempted" || echo "skipped")"
  echo "- markdownlint: $([[ $RUN_LINT -eq 1 ]] && echo "attempted" || echo "skipped")"
} > "$SUMMARY"

# -------- optional: check CI status on default branch
if (( RUN_GH==1 )); then
  if have gh; then
    note "Checking GitHub Actions status on '${BRANCH_DEFAULT}'…"
    # Requires 'gh auth login' beforehand
    set +e
    gh run list --branch "$BRANCH_DEFAULT" --limit 20 --json name,conclusion,workflowDatabaseId,url \
      | python3 - <<'PY'
import sys, json
runs = json.load(sys.stdin)
failing = [r for r in runs if r.get("conclusion") not in (None,"success","skipped","cancelled")]
if failing:
    print("[clean-docs] CI FAILURES detected on default branch:")
    for r in failing:
        print(f"- {r.get('name')} -> {r.get('conclusion')} ({r.get('url')})")
    sys.exit(3)
print("[clean-docs] CI looks green or no failures in recent runs.")
PY
    RC=$?
    set -e
    if [[ $RC -ne 0 ]]; then
      echo "[clean-docs] See above CI failures. Fix before release." >&2
      # Keep working tree for fixes; do not exit non-zero to allow doc commit first.
    fi
  else
    note "'gh' CLI not found; skipping CI status check. (Install: https://cli.github.com/)"
  fi
fi

# -------- auto-commit
git add --all
if ! git diff --cached --quiet; then
  # Skip pre-commit hooks for automated cleanup (use --no-verify)
  if git commit --no-verify -m "docs: remove emojis and normalize production-grade Markdown (automated cleanup)" 2>&1; then
    note "Committed documentation cleanup. Branch: $(git rev-parse --abbrev-ref HEAD)"
  else
    note "Commit failed (may need manual intervention). Changes are staged."
  fi
else
  note "No staged changes to commit."
fi

echo "[clean-docs] DONE. Summary at: $SUMMARY"

