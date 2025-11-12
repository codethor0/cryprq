#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# CI cleanup step for Linux runners with strict caps:
# DISK_CAP_GB=10 (total disk usage)
# MAX_LOG_MB=5 (per log file)
set -euxo pipefail

DISK_CAP_GB=${DISK_CAP_GB:-10}
MAX_LOG_MB=${MAX_LOG_MB:-5}

echo "== Disk Usage Before Cleanup =="
df -h
DISK_USAGE_GB=$(df -BG . | tail -1 | awk '{print $3}' | sed 's/G//' || echo "0")
echo "Current disk usage: ${DISK_USAGE_GB}GB (cap: ${DISK_CAP_GB}GB)"

if [ "$DISK_USAGE_GB" -gt "$DISK_CAP_GB" ]; then
  echo "⚠️ WARNING: Disk usage (${DISK_USAGE_GB}GB) exceeds cap (${DISK_CAP_GB}GB)"
fi

echo ""
echo "== Workspace Usage =="
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

# Trim logs larger than MAX_LOG_MB (enforce strict cap)
echo "== Trimming logs larger than ${MAX_LOG_MB}MB =="
find . -type f -name "*.log" -size +${MAX_LOG_MB}M -exec bash -c '
  file="$1"
  size_mb=$(stat -c%s "$file" 2>/dev/null | awk "{print \$1/1024/1024}" || echo "0")
  if [ "$(echo "$size_mb > $MAX_LOG_MB" | bc 2>/dev/null || echo "0")" = "1" ]; then
    echo "Trimming log: $file (${size_mb}MB)"
    tail -n 10000 "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
  fi
' _ {} \; 2>/dev/null || true

# Remove old log files (older than 3 days)
find . -type f -name "*.log" -mtime +3 -delete 2>/dev/null || true

# Remove old artifacts to keep under DISK_CAP_GB
echo "== Removing old artifacts =="
find . -type d -name "target" -prune -o -type f -name "*.rlib" -mtime +7 -delete 2>/dev/null || true
find . -type d -name "target" -prune -o -type f -name "*.rmeta" -mtime +7 -delete 2>/dev/null || true

echo ""
echo "== Disk Usage After Cleanup =="
df -h || true
DISK_USAGE_AFTER=$(df -BG . | tail -1 | awk '{print $3}' | sed 's/G//' || echo "0")
echo "Disk usage after cleanup: ${DISK_USAGE_AFTER}GB (cap: ${DISK_CAP_GB}GB)"

if [ "$DISK_USAGE_AFTER" -gt "$DISK_CAP_GB" ]; then
  echo "⚠️ WARNING: Disk usage still exceeds cap after cleanup"
  exit 1
else
  echo "✅ Disk usage within cap"
fi

# Print summary
echo ""
echo "== Cleanup Summary =="
echo "Disk cap: ${DISK_CAP_GB}GB"
echo "Log size cap: ${MAX_LOG_MB}MB per file"
echo "Disk usage before: ${DISK_USAGE_GB}GB"
echo "Disk usage after: ${DISK_USAGE_AFTER}GB"
if [ "$DISK_USAGE_GB" != "0" ] && [ "$DISK_USAGE_AFTER" != "0" ]; then
  RECLAIMED=$((DISK_USAGE_GB - DISK_USAGE_AFTER))
  echo "Space reclaimed: ${RECLAIMED}GB"
fi

