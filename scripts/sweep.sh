#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/qa_common.sh"

SCENARIOS=()
QA_DIR=${QA_DIR:-"$(pwd)/qa-$(date +%Y%m%d-%H%M%S)"}
BUILD_IMAGE=false
LOCAL_IMAGE=${LOCAL_IMAGE:-"cryprq-local:qa"}

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Options:
  --scenario <name>   Run a specific scenario (may repeat). Choices:
                      baseline-docker-handshake | rotation-60s | parallel-dialers | soak
  --all               Run all scenarios serially.
  --image <name>      Docker image to use (default: ${LOCAL_IMAGE}).
  --build             Build the Docker image before running.
  --output <dir>      Directory to store QA artifacts (default: ${QA_DIR}).
  -h, --help          Show this help message.

Environment overrides:
  LISTEN_PORT, BASE_NET, LISTENER_NAME, DIAL_PREFIX, LOCAL_IMAGE,
  SOAK_DURATION, ROTATE_SOAK, PARALLEL_COUNT, ROTATE_60_DURATION.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      [[ $# -ge 2 ]] || { echo "Missing value for --scenario"; exit 1; }
      SCENARIOS+=("$2")
      shift 2
      ;;
    --all)
      SCENARIOS=(baseline-docker-handshake rotation-60s parallel-dialers soak)
      shift
      ;;
    --image)
      [[ $# -ge 2 ]] || { echo "Missing value for --image"; exit 1; }
      LOCAL_IMAGE="$2"
      shift 2
      ;;
    --build)
      BUILD_IMAGE=true
      shift
      ;;
    --output)
      [[ $# -ge 2 ]] || { echo "Missing value for --output"; exit 1; }
      QA_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ ${#SCENARIOS[@]} -eq 0 ]]; then
  echo "No scenarios selected. Use --scenario or --all."
  usage
  exit 1
fi

LOCAL_IMAGE="${LOCAL_IMAGE}"

for cmd in docker timeout; do require_cmd "$cmd"; done

if [[ "$BUILD_IMAGE" == true ]]; then
  log "Building test image (${LOCAL_IMAGE})"
  docker build -t "${LOCAL_IMAGE}" "${SCRIPT_DIR}/.."
fi

mkdir -p "${QA_DIR}"
log "QA artifacts directory: ${QA_DIR}"

trap cleanup EXIT

run_scenario() {
  local name="$1"
  case "$name" in
    baseline-docker-handshake)
      baseline_handshake
      ;;
    rotation-60s)
      rotation_60s
      ;;
    parallel-dialers)
      parallel_dialers
      ;;
    soak)
      soak_test
      ;;
    *)
      echo "Unknown scenario: $name" && exit 1
      ;;
  esac
}

for scen in "${SCENARIOS[@]}"; do
  run_scenario "$scen"
  log "Scenario '$scen' complete."
  log "Artifacts stored in ${QA_DIR}"
  # ensure clean slate between runs handled by scenario cleanup, but double-check
  cleanup
done

log "All requested scenarios finished successfully."
