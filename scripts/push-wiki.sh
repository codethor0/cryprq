#!/bin/bash
# Push all wiki pages to GitHub wiki repository
# Run this AFTER creating the first page manually on GitHub

set -e

REPO="codethor0/cryprq"
WIKI_DIR=".github/wiki"
TEMP_WIKI="/tmp/cryprq-wiki"

echo "GitHub Wiki Push Script"
echo "======================"
echo ""

# Check if wiki directory exists
if [ ! -d "$WIKI_DIR" ]; then
    echo "Error: $WIKI_DIR directory not found!"
    exit 1
fi

# Clone or update wiki repository
if [ -d "$TEMP_WIKI" ]; then
    echo "Updating existing wiki repository..."
    cd "$TEMP_WIKI"
    git pull origin main 2>/dev/null || git pull origin master 2>/dev/null || true
else
    echo "Cloning wiki repository..."
    git clone "https://github.com/${REPO}.wiki.git" "$TEMP_WIKI" || {
        echo "Error: Could not clone wiki repository."
        echo "Make sure you've created at least one page on GitHub wiki first."
        echo "Go to: https://github.com/${REPO}/wiki"
        exit 1
    }
    cd "$TEMP_WIKI"
fi

# Copy all wiki files
echo ""
echo "Copying wiki files..."
for file in "$WIKI_DIR"/*.md; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "  Copying $filename..."
        cp "$file" "$filename"
    fi
done

# Add and commit
echo ""
echo "Committing changes..."
git add *.md
git commit -m "docs: add wiki pages from repository

- Home page with overview
- Getting Started guide
- VPN Setup instructions
- Testing documentation
- Troubleshooting guide

All documentation is emoji-free and production-ready." || echo "No changes to commit"

# Push to GitHub
echo ""
echo "Pushing to GitHub..."
git push origin main 2>/dev/null || git push origin master 2>/dev/null || {
    echo "Error: Could not push to GitHub."
    echo "Make sure you have push access to the wiki repository."
    exit 1
}

echo ""
echo "✅ Wiki pages pushed successfully!"
echo ""
echo "View your wiki at: https://github.com/${REPO}/wiki"

