#!/usr/bin/env bash
set -euo pipefail

NAME="Thor Thor"
EMAIL="codethor@gmail.com"
LINKEDIN="https://www.linkedin.com/in/thor-thor0"
SPDX_ID="MIT"
YEAR="$(date +%Y)"

EXTENSIONS=(go rs py sh bash zsh rb yml yaml toml ini cfg nix ps1 js ts jsx tsx c cc cpp h hh hpp java kt kts scala cs php css scss html htm xml sql hs lua dart)
SPECIAL_NAMES=(Dockerfile Dockerfile.* Makefile makefile)
SKIP_DIRS=(.git node_modules vendor target build dist .next .venv venv __pycache__ .tox .mypy_cache coverage .idea .vscode)

is_text() {
  LC_ALL=C grep -qI '' "$1"
}

skip_file() {
  local file="$1"
  case "$file" in
    *.min.*|*.map|*.json|*.ipynb) return 0 ;;
    *) return 1 ;;
  esac
}

has_header() {
  local file="$1"
  grep -qE 'SPDX-License-Identifier:|codethor@gmail.com' "$file"
}

comment_style() {
  local ext="$1"
  case "$ext" in
    py|sh|bash|zsh|rb|yml|yaml|toml|ini|cfg|nix|ps1) echo "hash" ;;
    css|scss) echo "block" ;;
    html|htm|xml) echo "html" ;;
    sql|hs|lua) echo "dash" ;;
    php) echo "php" ;;
    *) echo "slash" ;;
  esac
}

make_header() {
  local style="$1"
  case "$style" in
    hash)
      cat <<EOF
# © ${YEAR} ${NAME}
# Contact: ${EMAIL}
# LinkedIn: ${LINKEDIN}
# SPDX-License-Identifier: ${SPDX_ID}
EOF
      ;;
    slash)
      cat <<EOF
// © ${YEAR} ${NAME}
// Contact: ${EMAIL}
// LinkedIn: ${LINKEDIN}
// SPDX-License-Identifier: ${SPDX_ID}
EOF
      ;;
    block)
      cat <<EOF
/*
 * © ${YEAR} ${NAME}
 * Contact: ${EMAIL}
 * LinkedIn: ${LINKEDIN}
 * SPDX-License-Identifier: ${SPDX_ID}
 */
EOF
      ;;
    dash)
      cat <<EOF
-- © ${YEAR} ${NAME}
-- Contact: ${EMAIL}
-- LinkedIn: ${LINKEDIN}
-- SPDX-License-Identifier: ${SPDX_ID}
EOF
      ;;
    html)
      cat <<EOF
<!--
  © ${YEAR} ${NAME}
  Contact: ${EMAIL}
  LinkedIn: ${LINKEDIN}
  SPDX-License-Identifier: ${SPDX_ID}
-->
EOF
      ;;
    php)
      cat <<EOF
/*
 * © ${YEAR} ${NAME}
 * Contact: ${EMAIL}
 * LinkedIn: ${LINKEDIN}
 * SPDX-License-Identifier: ${SPDX_ID}
 */
EOF
      ;;
  esac
}

insert_header() {
  local file="$1"
  local style="$2"
  local header
  header="$(make_header "$style")"

  local tmp
  tmp="$(mktemp)"
  trap 'rm -f "$tmp"' RETURN

  case "$style" in
    hash)
      if head -n1 "$file" | grep -q '^#!'; then
        {
          head -n1 "$file"
          echo ""
          echo "$header"
          tail -n +2 "$file"
        } >"$tmp"
      else
        {
          echo "$header"
          echo ""
          cat "$file"
        } >"$tmp"
      fi
      ;;
    html)
      if head -n1 "$file" | grep -Eqi '^<\?xml|^<!DOCTYPE'; then
        {
          head -n1 "$file"
          echo ""
          echo "$header"
          tail -n +2 "$file"
        } >"$tmp"
      else
        {
          echo "$header"
          echo ""
          cat "$file"
        } >"$tmp"
      fi
      ;;
    php)
      if head -n1 "$file" | grep -q '^<\?php'; then
        {
          head -n1 "$file"
          echo "$header"
          tail -n +2 "$file"
        } >"$tmp"
      else
        {
          echo "$header"
          echo ""
          cat "$file"
        } >"$tmp"
      fi
      ;;
    *)
      {
        echo "$header"
        echo ""
        cat "$file"
      } >"$tmp"
      ;;
  esac

  mv "$tmp" "$file"
}

ext_in_list() {
  local ext="$1"
  for e in "${EXTENSIONS[@]}"; do
    [[ "$ext" == "$e" ]] && return 0
  done
  return 1
}

is_special_name() {
  local base="$1"
  for s in "${SPECIAL_NAMES[@]}"; do
    if [[ "$base" == $s ]]; then
      return 0
    fi
  done
  return 1
}

build_prune_args() {
  local args=()
  for d in "${SKIP_DIRS[@]}"; do
    args+=(-path "*/${d}")
    args+=(-prune)
    args+=(-o)
  done
  echo "${args[@]}"
}

main() {
  local prune_args
  prune_args=($(build_prune_args))

  # shellcheck disable=SC2206
  find . "${prune_args[@]}" -type f -print0 |
    while IFS= read -r -d '' file; do
      skip_file "$file" && continue
      is_text "$file" || continue

      local base="${file##*/}"
      local ext="${file##*.}"

      if is_special_name "$base"; then
        ext="__special_hash__"
      elif [[ "$file" == *.* ]]; then
        :
      else
        continue
      fi

      if [[ "$ext" == "__special_hash__" ]]; then
        style="hash"
      else
        ext_in_list "$ext" || continue
        style="$(comment_style "$ext")"
      fi

      has_header "$file" && continue

      insert_header "$file" "$style"
      echo "Added header to: $file"
    done
}

main "$@"

