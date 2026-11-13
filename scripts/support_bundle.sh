#!/usr/bin/env bash
set -euo pipefail

# CrypRQ Support Bundle Collection Script
# Collects diagnostic information for troubleshooting and bug reports.
# This script is read-only and does not modify system state or stop containers.
#
# Security and privacy:
# - Does not read environment variables or shell history
# - Does not access files outside the repository
# - Does not collect Docker environment variables
# - Does not inspect system network interfaces or routes
# - Only collects safe, non-secret metadata and logs

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BUNDLE_DIR="${REPO_ROOT}/support_bundles/cryprq_support_${TIMESTAMP}"

echo "[SUPPORT BUNDLE] Creating support bundle directory: ${BUNDLE_DIR}"
mkdir -p "${BUNDLE_DIR}"
mkdir -p "${BUNDLE_DIR}/docs"

# Collect git metadata (safe, non-secret information only)
# Includes: working tree status, branch list, recent commits, HEAD commit hash
echo "[SUPPORT BUNDLE] Collecting git metadata..."
{
  echo "=== Git Status ==="
  git status -sb || echo "git status failed"
  echo ""
  echo "=== Git Branches ==="
  git branch -a || echo "git branch failed"
  echo ""
  echo "=== Recent Commits ==="
  git log -5 --oneline || echo "git log failed"
  echo ""
  echo "=== HEAD Commit ==="
  git rev-parse HEAD || echo "git rev-parse HEAD failed"
} > "${BUNDLE_DIR}/git_info.txt" 2>&1

# Run repository health checks (read-only validation commands)
# Captures: Rust formatting check, governance shield results
# Note: Governance shield failures may be ENV LIMITATION (SSL/TLS, npm EPERM) and not product bugs
echo "[SUPPORT BUNDLE] Running health checks..."
{
  echo "=== Rust Formatting Check ==="
  cargo fmt --all --check || echo "cargo fmt --all --check failed (exit code: $?)"
  echo ""
  echo "=== Governance Shield ==="
  python3 scripts/governance_shield.py || echo "governance_shield.py failed (exit code: $?)"
  echo ""
  echo "Note: Failures in governance_shield.py may be ENV LIMITATION (SSL/TLS, npm EPERM) and not product bugs."
} > "${BUNDLE_DIR}/health_checks.txt" 2>&1

# Collect Docker state (if Docker is available)
# Captures: running containers, compose stack status, limited container logs (last 500 lines)
# Non-critical: if Docker is not installed, only logs a warning and continues
echo "[SUPPORT BUNDLE] Collecting Docker state..."
if command -v docker >/dev/null 2>&1; then
  {
    echo "=== Docker Containers ==="
    docker ps || echo "docker ps failed"
    echo ""
    echo "=== Portal Containers ==="
    docker compose -p cryprq-portal -f docker-compose.portal.yml ps || echo "docker compose portal ps failed"
    echo ""
    echo "=== E2E Stack Containers ==="
    docker compose -f docker-compose.e2e.yml ps || echo "docker compose e2e ps failed"
  } > "${BUNDLE_DIR}/docker_state.txt" 2>&1

  # Collect container logs (if containers exist and are running)
  # Only collects logs from cryprq-web and cryprq-e2e* containers
  # Limits to last 500 lines to keep bundle size reasonable
  if docker ps --format "{{.Names}}" | grep -q "^cryprq-web$"; then
    echo "[SUPPORT BUNDLE] Collecting cryprq-web logs..."
    docker logs --tail=500 cryprq-web > "${BUNDLE_DIR}/docker_logs_cryprq-web.txt" 2>&1 || echo "Failed to collect cryprq-web logs"
  fi

  for container in $(docker ps --format "{{.Names}}" | grep "^cryprq-e2e"); do
    echo "[SUPPORT BUNDLE] Collecting ${container} logs..."
    docker logs --tail=500 "${container}" > "${BUNDLE_DIR}/docker_logs_${container}.txt" 2>&1 || echo "Failed to collect ${container} logs"
  done
else
  echo "[SUPPORT BUNDLE] WARNING: Docker not available, skipping Docker state collection"
  echo "Docker not installed or not in PATH" > "${BUNDLE_DIR}/docker_state.txt"
fi

# Collect web build artifacts summary (directory tree structure only)
# Non-critical: if web/dist does not exist, logs that and continues
echo "[SUPPORT BUNDLE] Collecting web build artifacts summary..."
if [ -d "${REPO_ROOT}/web/dist" ]; then
  ls -R "${REPO_ROOT}/web/dist" > "${BUNDLE_DIR}/web_dist_tree.txt" 2>&1 || echo "Failed to list web/dist"
else
  echo "web/dist directory does not exist" > "${BUNDLE_DIR}/web_dist_tree.txt"
fi

# Copy relevant documentation files to bundle
# Includes: quickstart guides, test plans, troubleshooting docs
# Non-critical: if a doc is missing, creates a .missing file and continues
echo "[SUPPORT BUNDLE] Collecting documentation snapshot..."
DOCS=(
  "QUICKSTART_CLI.md"
  "WEB_TESTER_GUIDE.md"
  "VPN_TEST_PLAN.md"
  "FILE_TRANSFER_TEST_PLAN.md"
  "TROUBLESHOOTING_SUPPORT_BUNDLE.md"
)

for doc in "${DOCS[@]}"; do
  if [ -f "${REPO_ROOT}/docs/${doc}" ]; then
    cp "${REPO_ROOT}/docs/${doc}" "${BUNDLE_DIR}/docs/" || echo "Failed to copy ${doc}"
  else
    echo "Document ${doc} not found" > "${BUNDLE_DIR}/docs/${doc}.missing" 2>&1
  fi
done

# Create compressed tarball of the support bundle
# Critical step: if tarball creation fails, script exits with non-zero
echo "[SUPPORT BUNDLE] Creating compressed tarball..."
cd "$(dirname "${BUNDLE_DIR}")"
tar czf "$(basename "${BUNDLE_DIR}").tar.gz" "$(basename "${BUNDLE_DIR}")" || {
  echo "[SUPPORT BUNDLE] ERROR: Failed to create tarball"
  exit 1
}

echo "[SUPPORT BUNDLE] Support bundle created at: ${BUNDLE_DIR}.tar.gz"
echo "[SUPPORT BUNDLE] Bundle contents:"
ls -lh "${BUNDLE_DIR}.tar.gz"

