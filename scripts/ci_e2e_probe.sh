#!/usr/bin/env bash
set -euo pipefail

# CrypRQ CI/E2E Probe Script
# Run a local CI/E2E probe on the current HEAD and print a summary matching the CI/E2E Investigation Runbook.
#
# Security and privacy:
# - Does not read environment variables or shell history
# - Does not access files outside the repository
# - Does not collect Docker environment variables
# - Builds/tests locally and starts an isolated Docker test stack
#
# ENV LIMITATION handling:
# - Reports Docker/network failures as environment limitations
# - Classifies environment failures as ENV_LIMITATION in the summary
# - Exits 1 for failed checks and 2 when required checks could not run

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output (optional, fallback to plain text if not supported)
RED=""
GREEN=""
YELLOW=""
NC=""
if [ -t 1 ]; then
  RED="\033[0;31m"
  GREEN="\033[0;32m"
  YELLOW="\033[0;33m"
  NC="\033[0m"
fi

# Results tracking
HAS_FAILURE=0
HAS_LIMITATION=0

record_result() {
  local key="$1" result="$2"
  printf -v "RESULT_${key}" '%s' "$result"
  case "$result" in
    FAIL) HAS_FAILURE=1 ;;
    *ENV_LIMITATION*|UNAVAILABLE) HAS_LIMITATION=1 ;;
  esac
}

# Helper function to classify command output
classify_result() {
  local cmd="$1"
  local exit_code="$2"
  local output="$3"
  
  if [ "$exit_code" -eq 0 ]; then
    echo "PASS"
    return 0
  fi
  
  # Check for known ENV LIMITATION patterns
  if echo "$output" | grep -qiE "(Could not resolve host|SSL certificate|index\.crates\.io|EPERM|permission denied|Docker daemon|Cannot connect to the Docker daemon)"; then
    echo "ENV_LIMITATION"
    return 0
  fi
  
  echo "FAIL"
  return 0
}

# Print header
echo "============================================================"
echo "CrypRQ CI/E2E Probe"
echo "============================================================"
echo ""
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
echo "Timestamp: $TIMESTAMP"
echo ""

# Step 0: Git state
echo "[PROBE] Step 0: Git state"
HEAD_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
HEAD_MSG=$(git log -1 --oneline 2>/dev/null || echo "unknown")
echo "HEAD: $HEAD_SHA"
echo "Commit: $HEAD_MSG"
echo ""
git status -sb 2>/dev/null || echo "git status failed"
echo ""
git branch -a 2>/dev/null | head -5 || echo "git branch failed"
echo ""

# Step 1: Static gates
echo "[PROBE] Step 1: Static gates"
echo ""

# 1.1 Governance shield
echo -n "  governance_shield: "
EXIT_CODE=0
OUTPUT=$(python3 "$REPO_ROOT/scripts/governance_shield.py" 2>&1) || EXIT_CODE=$?
RESULT=$(classify_result "governance_shield" "$EXIT_CODE" "$OUTPUT")
record_result "governance_shield" "$RESULT"
if [ "$RESULT" = "PASS" ]; then
  echo -e "${GREEN}PASS${NC}"
elif [ "$RESULT" = "ENV_LIMITATION" ]; then
  echo -e "${YELLOW}ENV_LIMITATION${NC}"
else
  echo -e "${RED}FAIL${NC}"
fi

# 1.2 Cargo fmt
echo -n "  cargo fmt:         "
EXIT_CODE=0
OUTPUT=$(cd "$REPO_ROOT" && cargo fmt --all --check 2>&1) || EXIT_CODE=$?
RESULT=$(classify_result "cargo fmt" "$EXIT_CODE" "$OUTPUT")
record_result "cargo_fmt" "$RESULT"
if [ "$RESULT" = "PASS" ]; then
  echo -e "${GREEN}PASS${NC}"
elif [ "$RESULT" = "ENV_LIMITATION" ]; then
  echo -e "${YELLOW}ENV_LIMITATION${NC}"
else
  echo -e "${RED}FAIL${NC}"
fi

# 1.3 Cargo clippy (optional)
echo -n "  clippy:            "
EXIT_CODE=0
OUTPUT=$(cd "$REPO_ROOT" && timeout 60 cargo clippy --all-targets --all-features -- -D warnings 2>&1) || EXIT_CODE=$?
RESULT=$(classify_result "clippy" "$EXIT_CODE" "$OUTPUT")
record_result "clippy" "$RESULT"
if [ "$RESULT" = "PASS" ]; then
  echo -e "${GREEN}PASS${NC}"
elif [ "$RESULT" = "ENV_LIMITATION" ]; then
  echo -e "${YELLOW}ENV_LIMITATION${NC}"
else
  echo -e "${RED}FAIL${NC}"
fi

# 1.4 Cargo test (optional)
echo -n "  cargo test:        "
EXIT_CODE=0
OUTPUT=$(cd "$REPO_ROOT" && timeout 120 cargo test --workspace --lib 2>&1) || EXIT_CODE=$?
RESULT=$(classify_result "cargo test" "$EXIT_CODE" "$OUTPUT")
record_result "cargo_test" "$RESULT"
if [ "$RESULT" = "PASS" ]; then
  echo -e "${GREEN}PASS${NC}"
elif [ "$RESULT" = "ENV_LIMITATION" ]; then
  echo -e "${YELLOW}ENV_LIMITATION${NC}"
else
  echo -e "${RED}FAIL${NC}"
fi

# 1.5 npm build
echo -n "  npm build:         "
EXIT_CODE=0
OUTPUT=$(cd "$REPO_ROOT/web" && npm run build 2>&1) || EXIT_CODE=$?
RESULT=$(classify_result "npm build" "$EXIT_CODE" "$OUTPUT")
record_result "npm_build" "$RESULT"
if [ "$RESULT" = "PASS" ]; then
  echo -e "${GREEN}PASS${NC}"
elif [ "$RESULT" = "ENV_LIMITATION" ]; then
  echo -e "${YELLOW}ENV_LIMITATION${NC}"
else
  echo -e "${RED}FAIL${NC}"
fi

echo ""

# Step 2: E2E probe
echo "[PROBE] Step 2: legacy transport E2E (Docker)"
echo ""

# Check Docker availability
DOCKER_AVAILABLE=false
if command -v docker >/dev/null 2>&1; then
  if timeout 5 docker info >/dev/null 2>&1; then
    DOCKER_AVAILABLE=true
  fi
fi

if [ "$DOCKER_AVAILABLE" = false ]; then
  echo -e "  docker:            ${YELLOW}UNAVAILABLE${NC}"
  echo -e "  e2e run:           ${YELLOW}NOT_RUN (ENV_LIMITATION: Docker not available)${NC}"
  record_result "docker" "UNAVAILABLE"
  record_result "e2e_run" "NOT_RUN_ENV_LIMITATION"
else
  echo -e "  docker:            ${GREEN}AVAILABLE${NC}"
  record_result "docker" "AVAILABLE"
  
  # Run E2E suite
  echo -n "  e2e run:           "
  cd "$REPO_ROOT"
  
  # Clean up any existing containers
  docker compose -f docker/e2e/docker-compose.e2e.yml down --remove-orphans >/dev/null 2>&1 || true
  
  # Run E2E with timeout
  EXIT_CODE=0
  OUTPUT=$(docker compose -f docker/e2e/docker-compose.e2e.yml up --build -d listener dialer 2>&1 && timeout 300 bash "$REPO_ROOT/docker/e2e/test_suite.sh" 2>&1) || EXIT_CODE=$?
  
  # Clean up
  docker compose -f docker/e2e/docker-compose.e2e.yml down --remove-orphans >/dev/null 2>&1 || true
  
  # Check for environment issues
  if echo "$OUTPUT" | grep -qiE "(permission denied|Cannot connect to the Docker daemon|Docker daemon)"; then
    RESULT="NOT_RUN_ENV_LIMITATION"
    echo -e "${YELLOW}NOT_RUN (ENV_LIMITATION: Docker daemon issue)${NC}"
  elif [ "$EXIT_CODE" -eq 0 ] && echo "$OUTPUT" | grep -qx 'E2E_RESULT passed=2 failed=0 skipped=0'; then
    RESULT="PASS"
    echo -e "${GREEN}PASS${NC}"
  elif [ "$EXIT_CODE" -ne 0 ]; then
    RESULT="FAIL"
    echo -e "${RED}FAIL${NC}"
  else
    RESULT="FAIL"
    echo -e "${RED}FAIL${NC}"
  fi
  
  record_result "e2e_run" "$RESULT"
fi

echo ""

# Step 3: authenticated E2E. These two scripts carry the assertions about the
# hybrid handshake, file integrity and record validation that the legacy
# Docker harness never made (docs/E2E_COVERAGE_MAP.md). They need no Docker.
echo "[PROBE] Step 3: authenticated E2E (two processes)"
echo ""
cd "$REPO_ROOT"
AUTH_BINARY="$REPO_ROOT/target/release/cryprq"
BUILD_EXIT=0
cargo build --release -p cryprq --locked >/dev/null 2>&1 || BUILD_EXIT=$?
run_auth_e2e() {
  local key="$1" label="$2" script="$3" result=FAIL
  if [ "$BUILD_EXIT" -eq 0 ] && [ -x "$AUTH_BINARY" ] && [ -f "$script" ] &&
    env -u CRYPRQ_TEST_MODE bash "$script" "$AUTH_BINARY" >/dev/null 2>&1; then
    result=PASS
  fi
  if [ "$result" = PASS ]; then
    echo -e "  ${label} ${GREEN}PASS${NC}"
  else
    echo -e "  ${label} ${RED}FAIL${NC}"
  fi
  record_result "$key" "$result"
}
run_auth_e2e auth_vpn "vpn link:         " "$REPO_ROOT/scripts/e2e-authenticated-vpn.sh"
run_auth_e2e auth_transfer "file transfer:    " "$REPO_ROOT/scripts/e2e-authenticated-file-transfer.sh"

echo ""

# Summary
echo "============================================================"
echo "CI/E2E Probe Summary (HEAD: ${HEAD_SHA:0:8})"
echo "============================================================"
echo ""
echo "Static Gates:"
printf "  governance_shield: %s\n" "${RESULT_governance_shield:-UNKNOWN}"
printf "  cargo fmt:         %s\n" "${RESULT_cargo_fmt:-UNKNOWN}"
printf "  clippy:            %s\n" "${RESULT_clippy:-UNKNOWN}"
printf "  cargo test:        %s\n" "${RESULT_cargo_test:-UNKNOWN}"
printf "  npm build:         %s\n" "${RESULT_npm_build:-UNKNOWN}"
echo ""
echo "Legacy transport E2E (Docker):"
printf "  docker:            %s\n" "${RESULT_docker:-UNKNOWN}"
printf "  e2e run:           %s\n" "${RESULT_e2e_run:-UNKNOWN}"
echo ""
echo "Authenticated E2E:"
printf "  vpn link:          %s\n" "${RESULT_auth_vpn:-UNKNOWN}"
printf "  file transfer:     %s\n" "${RESULT_auth_transfer:-UNKNOWN}"
echo ""
echo "============================================================"
echo ""

# A summary is not success if a required check failed or could not run.
if [ "$HAS_FAILURE" -ne 0 ]; then exit 1; fi
if [ "$HAS_LIMITATION" -ne 0 ]; then exit 2; fi
exit 0

