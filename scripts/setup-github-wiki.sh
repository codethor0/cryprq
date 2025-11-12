#!/bin/bash
# Setup GitHub Wiki from .github/wiki/ directory
# This script helps copy wiki files to the GitHub wiki repository

set -e

REPO_NAME="codethor0/cryprq"
WIKI_DIR=".github/wiki"
WIKI_REPO="${REPO_NAME}.wiki.git"

echo "GitHub Wiki Setup Script"
echo "========================"
echo ""
echo "This script will help you set up the GitHub wiki."
echo ""
echo "Prerequisites:"
echo "  1. GitHub wiki must be enabled in repository settings"
echo "  2. You must have push access to the repository"
echo ""
read -p "Press Enter to continue or Ctrl+C to cancel..."

# Check if wiki directory exists
if [ ! -d "$WIKI_DIR" ]; then
    echo "Error: $WIKI_DIR directory not found!"
    exit 1
fi

# Check if wiki files exist
WIKI_FILES=$(find "$WIKI_DIR" -name "*.md" -type f)
if [ -z "$WIKI_FILES" ]; then
    echo "Error: No wiki files found in $WIKI_DIR!"
    exit 1
fi

echo ""
echo "Found wiki files:"
echo "$WIKI_FILES" | while read -r file; do
    echo "  - $(basename "$file")"
done

echo ""
echo "Step 1: Clone the wiki repository"
echo "---------------------------------"
echo "Run this command to clone the wiki repository:"
echo ""
echo "  git clone https://github.com/${WIKI_REPO} wiki-temp"
echo ""
read -p "Have you cloned the wiki repository? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Please clone the wiki repository first, then run this script again."
    exit 0
fi

# Check if wiki-temp directory exists
if [ ! -d "wiki-temp" ]; then
    echo "Error: wiki-temp directory not found!"
    echo "Please clone the wiki repository first:"
    echo "  git clone https://github.com/${WIKI_REPO} wiki-temp"
    exit 1
fi

echo ""
echo "Step 2: Copy wiki files"
echo "----------------------"
echo "Copying files from $WIKI_DIR to wiki-temp/..."

# Copy all markdown files
for file in "$WIKI_DIR"/*.md; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "  Copying $filename..."
        cp "$file" "wiki-temp/$filename"
    fi
done

echo ""
echo "Step 3: Commit and push"
echo "----------------------"
echo "Files have been copied to wiki-temp/"
echo ""
echo "Next steps:"
echo "  1. cd wiki-temp"
echo "  2. git add *.md"
echo "  3. git commit -m 'docs: add wiki pages from repository'"
echo "  4. git push origin main"
echo ""
echo "Or run these commands:"
echo ""
echo "  cd wiki-temp && \\"
echo "  git add *.md && \\"
echo "  git commit -m 'docs: add wiki pages from repository' && \\"
echo "  git push origin main"
echo ""

read -p "Do you want to commit and push now? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    cd wiki-temp
    git add *.md
    git commit -m "docs: add wiki pages from repository" || echo "No changes to commit"
    git push origin main
    cd ..
    echo ""
    echo "Wiki has been updated!"
    echo "You can now view it at: https://github.com/${REPO_NAME}/wiki"
else
    echo "Wiki files are ready in wiki-temp/"
    echo "You can commit and push them manually when ready."
fi

echo ""
echo "Done!"

