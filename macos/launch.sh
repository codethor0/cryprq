#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
# macOS launch script for CrypRQ (if no GUI packager)
# Usage: bash macos/launch.sh [listener|dialer] [port] [peer]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:-listener}"
PORT="${2:-9999}"
PEER="${3:-/ip4/127.0.0.1/udp/9999/quic-v1}"

BIN="${ROOT}/target/aarch64-apple-darwin/release/cryprq"

if [[ ! -x "$BIN" ]]; then
  echo "Error: Binary not found at $BIN"
  echo "Run: cargo build --release -p cryprq --target aarch64-apple-darwin"
  exit 1
fi

case "$MODE" in
  listener)
    echo "Starting listener on UDP/$PORT..."
    "$BIN" --listen "/ip4/0.0.0.0/udp/${PORT}/quic-v1"
    ;;
  dialer)
    echo "Connecting to $PEER..."
    "$BIN" --peer "$PEER"
    ;;
  *)
    echo "Usage: $0 [listener|dialer] [port] [peer]"
    exit 1
    ;;
esac

