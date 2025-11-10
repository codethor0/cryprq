#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET_DIR="${ROOT}/android/rust/libs"
INCLUDE_DIR="${ROOT}/android/rust/include"
TARGETS=("aarch64-linux-android" "x86_64-linux-android")

mkdir -p "${TARGET_DIR}" "${INCLUDE_DIR}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found in PATH" >&2
  exit 1
fi

if ! command -v cbindgen >/dev/null 2>&1; then
  echo "cbindgen not found; install with 'cargo install cbindgen'" >&2
  exit 1
fi

for target in "${TARGETS[@]}"; do
  echo ":: building cryprq_core for ${target}"
  cargo build --release -p cryp-rq-core --target "${target}"
  ARCH_DIR="${TARGET_DIR}/${target}"
  mkdir -p "${ARCH_DIR}"
  cp "${ROOT}/target/${target}/release/libcryprq_core.so" "${ARCH_DIR}/"
done

cbindgen --config "${ROOT}/cbindgen.toml" --crate cryprq_core --output "${INCLUDE_DIR}/cryprq_core.h"

echo "Artifacts written to ${TARGET_DIR}"

