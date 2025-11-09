#!/usr/bin/env bash
set -euo pipefail

### CONFIG ###############################################################
VERSION="${VERSION:-v0.1.0-alpha.1}"
REPO_SLUG="codethor0/cryprq"
GHCR_IMAGE="ghcr.io/${REPO_SLUG}"
TAG_BRANCH="main"
LISTEN_PORT=9999
TEMP_DIR="$(mktemp -d)"
QA_DIR="${TEMP_DIR}/qa"
BASE_NET="cryprq-qa"
LISTENER_NAME="cryprq-listener"
DIAL_PREFIX="cryprq-dialer"
LOCAL_IMAGE="cryprq-local:${VERSION}"
SOAK_DURATION=${SOAK_DURATION:-2700}   # seconds (45 min default)
ROTATE_SOAK=${ROTATE_SOAK:-300}
PARALLEL_COUNT=${PARALLEL_COUNT:-5}
ROTATE_60_DURATION=${ROTATE_60_DURATION:-240}
METRIC_INTERVAL=10
 RECONNECT_DELAY=${RECONNECT_DELAY:-5}
##########################################################################

source "$(dirname "$0")/qa_common.sh"

trap cleanup EXIT

for cmd in git docker jq curl timeout zip; do require_cmd "$cmd"; done

mkdir -p "${QA_DIR}"
log "QA artifacts directory: ${QA_DIR}"

ensure_git_clean() {
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Git tree must be clean." && exit 1
  fi
}
ensure_git_clean

log "Checking out ${TAG_BRANCH} and tagging ${VERSION}"
git checkout "${TAG_BRANCH}"
git pull --ff-only
git tag -a "${VERSION}" -m "CrypRQ ${VERSION}"
git push origin "${VERSION}"

log "Building local test image (${LOCAL_IMAGE})"
docker build -t "${LOCAL_IMAGE}" .

baseline_handshake
rotation_60s
parallel_dialers
soak_test

zip -r "${TEMP_DIR}/qa-artifacts-${VERSION}.zip" "${QA_DIR}"

if [[ -n "${GHCR_PAT:-}" ]]; then
  echo "${GHCR_PAT}" | docker login ghcr.io -u "${REPO_SLUG%%/*}" --password-stdin
else
  log "Skipping GHCR login (set GHCR_PAT to push automatically)."
fi

log "Running docker buildx multi-arch push"
docker buildx create --use >/dev/null 2>&1 || true
docker buildx build --platform linux/amd64,linux/arm64 \
  --push \
  --tag "${GHCR_IMAGE}:${VERSION}" \
  --tag "${GHCR_IMAGE}:latest" \
  -f Dockerfile .

log "Building release binaries and SBOM"
SHA_FILE="${TEMP_DIR}/SHA256SUMS"
: > "${SHA_FILE}"

./scripts/build-linux.sh
tar -C target/x86_64-unknown-linux-musl/release -czf "${TEMP_DIR}/cryprq-${VERSION}-linux-amd64.tar.gz" cryprq
sha256sum "${TEMP_DIR}/cryprq-${VERSION}-linux-amd64.tar.gz" >> "${SHA_FILE}"

# arm64 platform build (requires cross env)
docker buildx build \
  --platform linux/arm64 \
  -o type=local,dest="${TEMP_DIR}/arm64-out" \
  -f Dockerfile.reproducible .
tar -C "${TEMP_DIR}/arm64-out/usr/local/bin" -czf "${TEMP_DIR}/cryprq-${VERSION}-linux-arm64.tar.gz" cryprq
sha256sum "${TEMP_DIR}/cryprq-${VERSION}-linux-arm64.tar.gz" >> "${SHA_FILE}"

./scripts/build-macos.sh
tar -C target/release -czf "${TEMP_DIR}/cryprq-${VERSION}-macos-arm64.tar.gz" cryprq
sha256sum "${TEMP_DIR}/cryprq-${VERSION}-macos-arm64.tar.gz" >> "${SHA_FILE}"

cargo install cargo-cyclonedx --version 0.6.0 --locked >/dev/null 2>&1 || true
cargo cyclonedx --output-file "${TEMP_DIR}/SBOM-cyclonedx.json"

cat <<EOF

============================================================
Release ${VERSION} ready.

Artifacts:
  ${TEMP_DIR}/cryprq-${VERSION}-linux-amd64.tar.gz
  ${TEMP_DIR}/cryprq-${VERSION}-linux-arm64.tar.gz
  ${TEMP_DIR}/cryprq-${VERSION}-macos-arm64.tar.gz
  ${SHA_FILE}
  ${TEMP_DIR}/SBOM-cyclonedx.json
  ${TEMP_DIR}/qa-artifacts-${VERSION}.zip

Next steps:
  1. Publish GitHub Release with files above.
  2. Include QA summary and GHCR digests (check build output).
============================================================

EOF
