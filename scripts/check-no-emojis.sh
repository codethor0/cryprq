#!/bin/bash
# Check for emojis in code files
# Exit code 0 if no emojis found, 1 if emojis found

set -e

EMOJI_PATTERN="[🎉🔥✅📊📁📦🔄🚀🧹⚙️🔧🌐📄📝💡❌⚠️🎯🔐🔓🐳🔒ℹ️⏱️]"
FOUND=0

echo "Checking for emojis in code files..."

# Check source code files
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.mjs" -o -name "*.sh" \) \
    ! -path "*/node_modules/*" \
    ! -path "*/target/*" \
    ! -path "*/dist/*" \
    ! -path "*/.git/*" \
    -exec grep -l "$EMOJI_PATTERN" {} \; | while read file; do
    echo "ERROR: Emojis found in $file"
    grep -n "$EMOJI_PATTERN" "$file" | head -5
    FOUND=1
done

if [ $FOUND -eq 1 ]; then
    echo ""
    echo "ERROR: Emojis found in code files. Please remove them."
    exit 1
fi

echo "No emojis found in code files."
exit 0

