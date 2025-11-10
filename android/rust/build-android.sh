#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET_DIR="${ROOT}/android/rust/libs"
INCLUDE_DIR="${ROOT}/android/rust/include"
mkdir -p "${TARGET_DIR}" "${INCLUDE_DIR}"
[ -d "${TARGET_DIR}" ] && rm -rf "${TARGET_DIR:?}/"*

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found in PATH" >&2
  exit 1
fi

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "cargo-ndk not found; install with 'cargo install cargo-ndk --version 3.5.4'" >&2
  exit 1
fi

if ! command -v cbindgen >/dev/null 2>&1; then
  echo "cbindgen not found; install with 'cargo install cbindgen'" >&2
  exit 1
fi

if [ -z "${ANDROID_NDK_HOME:-}" ]; then
  DEFAULT_NDK="/opt/homebrew/share/android-ndk"
  if [ -d "${DEFAULT_NDK}" ]; then
    export ANDROID_NDK_HOME="${DEFAULT_NDK}"
  else
    echo "ANDROID_NDK_HOME is unset and default NDK path not found. Set ANDROID_NDK_HOME to your NDK root." >&2
    exit 1
  fi
fi

declare -a TARGETS=("aarch64-linux-android" "x86_64-linux-android")

for target in "${TARGETS[@]}"; do
  case "${target}" in
    "aarch64-linux-android") abi="arm64-v8a" ;;
    "x86_64-linux-android") abi="x86_64" ;;
    *) echo "Unsupported target ${target}" >&2; exit 1 ;;
  esac
  echo ":: building cryprq_core for ${target} (${abi})"
  cargo ndk \
    --target "${target}" \
    --platform 26 \
    -o "${TARGET_DIR}" \
    build --release -p cryp-rq-core
done

(
  cd "${ROOT}/core"
  cbindgen --config "${ROOT}/cbindgen.toml" --output "${INCLUDE_DIR}/cryprq_core.h"
)

echo "Artifacts written to ${TARGET_DIR}"

