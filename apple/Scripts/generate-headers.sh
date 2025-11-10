#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUTPUT_DIR="${ROOT}/apple/Shared/include"
mkdir -p "${OUTPUT_DIR}"

if ! command -v cbindgen >/dev/null 2>&1; then
  echo "cbindgen not found; install with 'cargo install cbindgen'" >&2
  exit 1
fi

(
  cd "${ROOT}/core"
  cbindgen --config "${ROOT}/cbindgen.toml" --output "${OUTPUT_DIR}/cryprq_core.h"
)

echo "Header written to ${OUTPUT_DIR}/cryprq_core.h"

