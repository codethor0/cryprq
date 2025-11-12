#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# CI cleanup step for Linux runners with a 10 GB cap
set -euxo pipefail

echo "== Disk before =="
df -h

echo "== Workspace usage =="
du -sh . || true

# Trim common language caches
echo "== Trim language caches =="
rm -rf ~/.cache/pip/http 2>/dev/null || true
find ~/.cache/pip wheels -type f -mtime +7 -delete 2>/dev/null || true
npm cache verify || true
npm cache prune || true
pnpm store prune || true
rm -rf ~/.gradle/caches/*/file-* || true

# Docker cleanup if available
if command -v docker >/dev/null 2>&1; then
  docker system prune -af || true
  docker builder prune -af || true
fi

# Remove old artifacts and logs in workspace to keep under 10 GB
find . -type f -name "*.log" -size +5M -mtime +3 -delete 2>/dev/null || true
find . -type f -name "*.log" -exec bash -lc 'f="{}"; [ $(stat -c%s "$f") -gt 5242880 ] && tail -n 2000 "$f" > "$f.trim" && mv "$f.trim" "$f"' \;

echo "== Disk after =="
df -h || true

