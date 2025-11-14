#!/bin/bash

# Copyright (c) 2025 Thor Thor
# Author: Thor Thor (GitHub: https://github.com/codethor0)
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# License: MIT (see LICENSE file for details)

# Check for emojis in code files
# Exit code 0 if no emojis found, 1 if emojis found

set -e

# Unicode emoji ranges (common emojis that can break builds)
EMOJI_PATTERN='[\U0001F300-\U0001F9FF\U00002600-\U000027BF\U0001F600-\U0001F64F\U0001F680-\U0001F6FF\U0001F1E0-\U0001F1FF\U00002700-\U000027BF]'
FOUND=0

echo "Checking for emojis in code files..."

# Check source code files
while IFS= read -r file; do
    if grep -qP "$EMOJI_PATTERN" "$file" 2>/dev/null; then
        echo "ERROR: Emojis found in $file"
        grep -nP "$EMOJI_PATTERN" "$file" 2>/dev/null | head -5 || true
        FOUND=1
    fi
done < <(find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.mjs" -o -name "*.sh" \) \
    ! -path "*/node_modules/*" \
    ! -path "*/target/*" \
    ! -path "*/dist/*" \
    ! -path "*/.git/*" \
    ! -path "*/scripts/check-no-emojis.sh")

if [ $FOUND -eq 1 ]; then
    echo ""
    echo "ERROR: Emojis found in code files. Please remove them."
    exit 1
fi

echo "No emojis found in code files."
exit 0

