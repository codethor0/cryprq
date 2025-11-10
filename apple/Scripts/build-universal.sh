#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUILD_DIR="${ROOT}/apple/build"
mkdir -p "${BUILD_DIR}"

echo "This is a placeholder script. Future iterations will produce xcframeworks by combining"
echo "aarch64-apple-ios/macOS static libraries compiled from cryp-rq-core."
echo "See docs/apple.md for the full plan."

