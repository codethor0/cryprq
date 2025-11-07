#!/bin/bash
set -e

echo "Force resetting to clean commit state..."

# Stash any local changes
git stash push -m "pre-cleanup-stash" || true

# Get the latest commit hash from origin
git fetch origin feat/batch-merge
clean_commit=$(git rev-parse origin/feat/batch-merge)

# Reset hard to the clean remote state
git reset --hard $clean_commit

# Verify no conflict markers remain
remaining_conflicts=$(find . -type f \( -name "*.toml" -o -name "*.rs" \) -exec grep -l "<<<<<<<" {} \; 2>/dev/null || true)

if [ -n "$remaining_conflicts" ]; then
    echo "ERROR: Remote branch still contains conflicts. Manual intervention required."
    echo "Conflicted files: $remaining_conflicts"
    exit 1
fi

echo "Clean state verified. Building Docker image..."
docker build -t cryprq-dev -f Dockerfile.reproducible .

echo "Build completed successfully."
