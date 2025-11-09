#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}" )" && pwd)"
cd "${SCRIPT_DIR}"

log() {
  printf '\n[%s] %s\n' "$(date +'%F %T')" "$*"
}

need() {
  command -v "$1" >/dev/null || { echo "[error] Missing command: $1"; exit 1; }
}

# --- Requirements ---
for tool in docker awk grep sed tee mktemp; do need "$tool"; done

if ! docker version >/dev/null 2>&1; then
  echo "[error] Docker daemon is not ready" >&2
  exit 1
fi

# --- Optional soak patch so SOAK_DIALERS is honoured ---
PATCH_SOAK_DEFAULT="1"
if [[ "${PATCH_SOAK:-$PATCH_SOAK_DEFAULT}" == "1" ]] && [[ -f scripts/qa_common.sh ]]; then
  if grep -qE 'seq[[:space:]]+1[[:space:]]+3' scripts/qa_common.sh; then
    log "Patching soak loop to honour SOAK_DIALERS"
    # macOS/BSD sed requires the empty string argument to -i
    sed -i '' -e 's/seq[[:space:]]\+1[[:space:]]\+3/seq 1 ${SOAK_DIALERS:-3}/' scripts/qa_common.sh
    if [[ "${AUTO_COMMIT_PATCH:-0}" == "1" && -d .git ]]; then
      git add scripts/qa_common.sh >/dev/null 2>&1 || true
      if ! git diff --cached --quiet; then
        git commit -m "QA: honour SOAK_DIALERS for soak scenario" >/dev/null 2>&1 || true
      fi
    fi
  fi
fi

# --- Helper to run a scenario and capture the QA directory ---
run_scenario() {
  local scenario=""
  local build_flag=""
  local -a env_pairs=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --build)
        build_flag="--build"; shift ;;
      --env)
        shift
        [[ $# -gt 0 ]] || { echo "run_scenario: --env requires KEY=VALUE" >&2; exit 1; }
        env_pairs+=("$1")
        shift ;;
      *)
        scenario="$1"; shift ;;
    esac
  done

  [[ -n "${scenario}" ]] || { echo "run_scenario: missing scenario" >&2; exit 1; }

  log "Running ${scenario} ${build_flag:+(build)}" >&2
  local tmp
  tmp="$(mktemp)"

  if ((${#env_pairs[@]} > 0)); then
    if ! command env "${env_pairs[@]}" ./scripts/sweep.sh --scenario "${scenario}" ${build_flag} | tee "${tmp}" >&2; then
      echo "[error] Scenario ${scenario} failed (see ${tmp})" >&2
      exit 1
    fi
  else
    if ! ./scripts/sweep.sh --scenario "${scenario}" ${build_flag} | tee "${tmp}" >&2; then
      echo "[error] Scenario ${scenario} failed (see ${tmp})" >&2
      exit 1
    fi
  fi

  local qa_dir
  qa_dir="$(awk -F': ' '/QA artifacts directory:/ {dir=$2} END {print dir}' "${tmp}")"
  rm -f "${tmp}" || true
  qa_dir="${qa_dir//$'\r'/}"
  qa_dir="${qa_dir##[$'\t' ]*}"  # remove leading tabs/spaces

  if [[ -z "${qa_dir}" || ! -d "${qa_dir}" ]]; then
    qa_dir="$(ls -td qa-* 2>/dev/null | head -1 || true)"
    echo "[warn] Could not parse QA directory from output; using newest: ${qa_dir:-<none>}" >&2
  fi

  [[ -n "${qa_dir}" && -d "${qa_dir}" ]] || { echo "[error] QA directory not found" >&2; exit 1; }
  printf '%s\n' "${qa_dir}"
}

find_latest_with() {
  local fname="$1"
  ls -td qa-* 2>/dev/null | head -40 | while read -r dir; do
    [[ -f "${dir}/${fname}" ]] && { echo "${dir}"; return 0; }
  done
  return 1
}

copy_qa_dir() {
  local src="$1"
  local dest_dir="$2"
  [[ -n "${src}" && -d "${src}" ]] || return 0
  local base
  base="$(basename "${src}")"
  rm -rf "${dest_dir}/${base}"
  cp -a "${src}" "${dest_dir}/${base}"
}

extract_bin() {
  local platform="$1" out="$2" image
  for image in "${IMAGE}:latest" "${LOCAL_IMAGE}"; do
    if docker manifest inspect "${image}" >/dev/null 2>&1 || docker image inspect "${image}" >/dev/null 2>&1; then
      log "Extracting binary from ${image} (${platform})"
      local cid
      set +e
      cid="$(docker create --platform "${platform}" "${image}" true 2>/dev/null)"
      set -e
      if [[ -n "${cid}" ]]; then
        if docker cp "${cid}:/usr/local/bin/cryprq" "${out}" >/dev/null 2>&1; then
          docker rm -v "${cid}" >/dev/null 2>&1 || true
          return 0
        fi
        docker rm -v "${cid}" >/dev/null 2>&1 || true
      fi
    fi
  done
  echo "[warn] Could not extract binary for ${platform}; skipped" >&2
  return 0
}

# --- Run remaining QA scenarios ---
PARALLEL_COUNT="${PARALLEL_COUNT:-5}"
SOAK_DURATION="${SOAK_DURATION:-600}"
ROTATE_SOAK="${ROTATE_SOAK:-120}"
SOAK_DIALERS="${SOAK_DIALERS:-3}"

PAR_DIR="$(run_scenario parallel-dialers --env "PARALLEL_COUNT=${PARALLEL_COUNT}")"
log "Parallel QA directory: ${PAR_DIR}"

SOAK_DIR="$(run_scenario soak --env "SOAK_DURATION=${SOAK_DURATION}" --env "ROTATE_SOAK=${ROTATE_SOAK}" --env "SOAK_DIALERS=${SOAK_DIALERS}")"
log "Soak QA directory: ${SOAK_DIR}"

BASE_DIR="$(find_latest_with baseline.log || true)"
ROT_DIR="$(find_latest_with rotation60.log || true)"
log "Baseline QA directory: ${BASE_DIR:-<none>}"
log "Rotation QA directory: ${ROT_DIR:-<none>}"

if [[ -n "${BASE_DIR}" ]] && [[ -f "${BASE_DIR}/baseline.log" ]]; then
  if grep -Eiq "Connected to|Inbound connection established" "${BASE_DIR}/baseline.log"; then
    log "Baseline check: PASS"
  else
    log "Baseline check: review ${BASE_DIR}/baseline.log"
  fi
else
  log "Baseline check: baseline.log not found (skipping)"
fi

if [[ -n "${ROT_DIR}" && -f "${ROT_DIR}/rotation60.log" ]]; then
  if grep -Eiq "Connected to|established" "${ROT_DIR}/rotation60.log"; then
    log "Rotation check: PASS"
  else
    log "Rotation check: review ${ROT_DIR}/rotation60.log"
  fi
else
  log "Rotation check: rotation60.log not found (skipping)"
fi

if compgen -G "${PAR_DIR}/parallel-*.log" >/dev/null; then
  if grep -Eiq "Connected to|established" "${PAR_DIR}"/parallel-*.log; then
    log "Parallel check: PASS"
  else
    log "Parallel check: review ${PAR_DIR}/parallel-*.log"
  fi
else
  log "Parallel check: logs not found in ${PAR_DIR}"
fi

if compgen -G "${SOAK_DIR}/soak-*.log" >/dev/null; then
  if grep -Eiq "Listening on|Connected to|established" "${SOAK_DIR}"/soak-*.log; then
    log "Soak check: PASS"
  else
    log "Soak check: review ${SOAK_DIR}/soak-*.log"
  fi
else
  log "Soak check: logs not found in ${SOAK_DIR}"
fi

# --- Package release artifacts ---
VERSION="${VERSION:-v0.1.0-alpha.1}"
IMAGE="${IMAGE:-ghcr.io/codethor0/cryprq}"
LOCAL_IMAGE="${LOCAL_IMAGE:-cryprq-local:qa}"
OUT_DIR="release-${VERSION}"

mkdir -p "${OUT_DIR}/qa" "${OUT_DIR}/bin" "${OUT_DIR}/images"

copy_qa_dir "${BASE_DIR}" "${OUT_DIR}/qa"
copy_qa_dir "${ROT_DIR}" "${OUT_DIR}/qa"
copy_qa_dir "${PAR_DIR}" "${OUT_DIR}/qa"
copy_qa_dir "${SOAK_DIR}" "${OUT_DIR}/qa"

log "Recording buildx imagetools output"
{
  echo "# imagetools (${IMAGE}:${VERSION})"
  docker buildx imagetools inspect "${IMAGE}:${VERSION}" || true
} > "${OUT_DIR}/images/imagetools-${VERSION}.txt" 2>&1
{
  echo "# imagetools (${IMAGE}:latest)"
  docker buildx imagetools inspect "${IMAGE}:latest" || true
} > "${OUT_DIR}/images/imagetools-latest.txt" 2>&1

extract_bin "linux/amd64" "${OUT_DIR}/bin/cryprq-linux-amd64"
extract_bin "linux/arm64" "${OUT_DIR}/bin/cryprq-linux-arm64"

if command -v shasum >/dev/null 2>&1; then
  (cd "${OUT_DIR}/bin" && shasum -a 256 * > ../checksums.txt)
elif command -v sha256sum >/dev/null 2>&1; then
  (cd "${OUT_DIR}/bin" && sha256sum * > ../checksums.txt)
else
  echo "[warn] No checksum utility (shasum/sha256sum) available" >&2
fi

if command -v syft >/dev/null 2>&1; then
  log "Generating SBOM using syft"
  syft "${IMAGE}:${VERSION}" -o spdx-json > "${OUT_DIR}/sbom-${VERSION}.spdx.json" || true
else
  log "syft not found; skipping SBOM"
fi

NOTES_FILE="${OUT_DIR}/RELEASE_NOTES.md"
{
  echo "# CrypRQ ${VERSION}"
  printf '\n**Date:** %s\n' "$(date -u +"%Y-%m-%d %H:%M:%SZ")"
  echo -e "\n## Images\n- ${IMAGE}:${VERSION}\n- ${IMAGE}:latest"
  echo -e "\n## Artifacts\n- bin/cryprq-linux-amd64\n- bin/cryprq-linux-arm64\n- checksums.txt\n- images/imagetools-${VERSION}.txt\n- images/imagetools-latest.txt\n- sbom-${VERSION}.spdx.json (if present)"
  echo -e "\n## QA Evidence"
  [[ -n "${BASE_DIR}" ]] && echo "- $(basename "${BASE_DIR}") (baseline)"
  [[ -n "${ROT_DIR}" ]] && echo "- $(basename "${ROT_DIR}") (rotation-60s)"
  echo "- $(basename "${PAR_DIR}") (parallel-dialers)"
  echo "- $(basename "${SOAK_DIR}") (soak)"
  echo -e "\n> Note: short rotation runs may emit \"Rotation wait loop failed (status 0)\" when the timeout ends normally."
} > "${NOTES_FILE}"

log "Bundle created at ${OUT_DIR}"
ls -lh "${OUT_DIR}" || true
log "QA directories inside bundle"
ls -lh "${OUT_DIR}/qa" || true

if [[ -f "${OUT_DIR}/checksums.txt" ]]; then
  log "Checksums"
  cat "${OUT_DIR}/checksums.txt"
fi

log "To publish with gh CLI (if desired):"
if ls "${OUT_DIR}"/sbom-*.spdx.json >/dev/null 2>&1; then
  echo "  gh release create ${VERSION} ${OUT_DIR}/bin/* ${OUT_DIR}/checksums.txt ${OUT_DIR}/images/*.txt ${OUT_DIR}/sbom-*.spdx.json -t ${VERSION} -F ${NOTES_FILE} --latest"
else
  echo "  gh release create ${VERSION} ${OUT_DIR}/bin/* ${OUT_DIR}/checksums.txt ${OUT_DIR}/images/*.txt -t ${VERSION} -F ${NOTES_FILE} --latest"
fi
