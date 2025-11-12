#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Setup script for dev-watcher service (systemd or launchd)
# Usage: bash scripts/setup-service.sh [systemd|launchd]

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SERVICE_TYPE="${1:-}"

if [[ -z "$SERVICE_TYPE" ]]; then
  echo "Usage: bash scripts/setup-service.sh [systemd|launchd]"
  echo ""
  echo "This script helps set up the dev-watcher as a background service."
  exit 1
fi

# Ensure .env.devwatch exists
if [[ ! -f ".env.devwatch" ]]; then
  echo "Creating .env.devwatch from template..."
  cp .env.devwatch.example .env.devwatch
  echo "✅ Created .env.devwatch - please edit with your settings"
fi

if [[ "$SERVICE_TYPE" == "systemd" ]]; then
  # Linux systemd setup
  SERVICE_DIR="$HOME/.config/systemd/user"
  SERVICE_FILE="$SERVICE_DIR/dev-watch.service"
  
  mkdir -p "$SERVICE_DIR"
  
  # Create service file from template
  sed -e "s|%h/path/to/your/repo|$ROOT|g" \
      scripts/dev-watch.service.example > "$SERVICE_FILE"
  
  echo "✅ Created systemd service: $SERVICE_FILE"
  echo ""
  echo "To enable and start:"
  echo "  systemctl --user daemon-reload"
  echo "  systemctl --user enable dev-watch.service"
  echo "  systemctl --user start dev-watch.service"
  echo ""
  echo "To check status:"
  echo "  systemctl --user status dev-watch.service"
  echo ""
  echo "To view logs:"
  echo "  journalctl --user -u dev-watch.service -f"
  
elif [[ "$SERVICE_TYPE" == "launchd" ]]; then
  # macOS launchd setup
  SERVICE_DIR="$HOME/Library/LaunchAgents"
  SERVICE_FILE="$SERVICE_DIR/com.cryrpq.devwatch.plist"
  
  mkdir -p "$SERVICE_DIR"
  
  # Create plist from template
  sed -e "s|/Users/YOU/path/to/repo|$ROOT|g" \
      scripts/com.cryrpq.devwatch.plist.example > "$SERVICE_FILE"
  
  echo "✅ Created launchd service: $SERVICE_FILE"
  echo ""
  echo "To load and start:"
  echo "  launchctl unload $SERVICE_FILE 2>/dev/null || true"
  echo "  launchctl load $SERVICE_FILE"
  echo "  launchctl start com.cryrpq.devwatch"
  echo ""
  echo "To check status:"
  echo "  launchctl list | grep cryrpq"
  echo ""
  echo "To view logs:"
  echo "  tail -f $ROOT/artifacts/dev-watch/launchd.out"
  echo "  tail -f $ROOT/artifacts/dev-watch/launchd.err"
  
else
  echo "Error: Unknown service type: $SERVICE_TYPE"
  echo "Supported types: systemd, launchd"
  exit 1
fi

