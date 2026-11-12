#!/usr/bin/env bash
set -euo pipefail

# CrypRQ Release Probe Script
# Run a release pre-flight probe on the current HEAD, using quick_smoke + ci_e2e_probe + git sanity checks.
#
# Security and privacy:
# - Does not read environment variables or shell history
# - Does not access files outside the repository
# - Does not collect Docker environment variables
# - Only runs read-only validation commands
#
# Exit code:
# - Always exits 0 unless script itself fails (syntax error, missing file, etc.)
# - This is a probe, not a strict gate; the red/green decision is left to the human reading the summary

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
declare -A RESULTS
WARNINGS=()

# Print header
echo "============================================================"
echo "CrypRQ Release Probe"
echo "============================================================"
echo ""
TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
echo "Timestamp: $TIMESTAMP"
echo ""

# Step 0: Git state
echo "[PROBE] Step 0: Git state"
HEAD_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
HEAD_MSG=$(git log -1 --oneline 2>/dev/null || echo "unknown")
echo "HEAD: $HEAD_SHA"
echo "Commit: $HEAD_MSG"
echo ""

# Check branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
echo "Branch: $CURRENT_BRANCH"
if [ "$CURRENT_BRANCH" != "main" ]; then
  WARNINGS+=("Not on main branch (current: $CURRENT_BRANCH)")
fi

# Check working tree
echo ""
git status -sb 2>/dev/null || echo "git status failed"
WORKING_TREE_CLEAN=true
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
  WORKING_TREE_CLEAN=false
  WARNINGS+=("Working tree not clean - not recommended for release")
fi

# Check origin/main
ORIGIN_MAIN_REACHABLE=true
ORIGIN_MAIN_SHA=$(git rev-parse origin/main 2>/dev/null || echo "")
if [ -z "$ORIGIN_MAIN_SHA" ]; then
  ORIGIN_MAIN_REACHABLE=false
  WARNINGS+=("origin/main not reachable (ENV LIMITATION: network or permissions)")
else
  echo "origin/main: $ORIGIN_MAIN_SHA"
  if [ "$HEAD_SHA" != "$ORIGIN_MAIN_SHA" ]; then
    WARNINGS+=("HEAD ($HEAD_SHA) != origin/main ($ORIGIN_MAIN_SHA)")
  fi
fi

echo ""

# Step 1: Quick smoke
echo "[PROBE] Step 1: Quick smoke (scripts/quick_smoke.sh)"
echo ""

QUICK_SMOKE_OUTPUT=""
QUICK_SMOKE_EXIT=0

# Run quick_smoke.sh and capture output
if [ -f "$REPO_ROOT/scripts/quick_smoke.sh" ]; then
  QUICK_SMOKE_OUTPUT=$(cd "$REPO_ROOT" && timeout 600 bash scripts/quick_smoke.sh 2>&1) || QUICK_SMOKE_EXIT=$?
  
  # Classify result
  if [ "$QUICK_SMOKE_EXIT" -eq 0 ]; then
    RESULTS["quick_smoke"]="PASS"
    echo -e "${GREEN}PASS${NC}"
  elif echo "$QUICK_SMOKE_OUTPUT" | grep -qiE "(permission denied|Cannot connect to the Docker daemon|Docker daemon|Could not resolve host|SSL certificate|index\.crates\.io|EPERM)"; then
    RESULTS["quick_smoke"]="ENV_LIMITATION"
    echo -e "${YELLOW}ENV_LIMITATION${NC}"
  else
    RESULTS["quick_smoke"]="FAIL"
    echo -e "${RED}FAIL${NC}"
  fi
else
  RESULTS["quick_smoke"]="NOT_RUN"
  WARNINGS+=("scripts/quick_smoke.sh not found")
  echo -e "${YELLOW}NOT_RUN (script not found)${NC}"
fi

echo ""

# Step 2: CI/E2E probe
echo "[PROBE] Step 2: CI/E2E probe (scripts/ci_e2e_probe.sh)"
echo ""

CI_E2E_OUTPUT=""
CI_E2E_EXIT=0

if [ -f "$REPO_ROOT/scripts/ci_e2e_probe.sh" ]; then
  # Run ci_e2e_probe.sh and capture output
  CI_E2E_OUTPUT=$(cd "$REPO_ROOT" && bash scripts/ci_e2e_probe.sh 2>&1) || CI_E2E_EXIT=$?
  
  # Extract key classifications from ci_e2e_probe output
  if echo "$CI_E2E_OUTPUT" | grep -qE "governance_shield:.*PASS"; then
    RESULTS["governance_shield"]="PASS"
  elif echo "$CI_E2E_OUTPUT" | grep -qE "governance_shield:.*ENV_LIMITATION"; then
    RESULTS["governance_shield"]="ENV_LIMITATION"
  else
    RESULTS["governance_shield"]="FAIL"
  fi
  
  if echo "$CI_E2E_OUTPUT" | grep -qE "cargo fmt:.*PASS"; then
    RESULTS["cargo_fmt"]="PASS"
  else
    RESULTS["cargo_fmt"]="FAIL"
  fi
  
  if echo "$CI_E2E_OUTPUT" | grep -qE "npm build:.*PASS"; then
    RESULTS["npm_build"]="PASS"
  elif echo "$CI_E2E_OUTPUT" | grep -qE "npm build:.*ENV_LIMITATION"; then
    RESULTS["npm_build"]="ENV_LIMITATION"
  else
    RESULTS["npm_build"]="FAIL"
  fi
  
  if echo "$CI_E2E_OUTPUT" | grep -qE "e2e run:.*PASS"; then
    RESULTS["e2e_run"]="PASS"
  elif echo "$CI_E2E_OUTPUT" | grep -qE "e2e run:.*NOT_RUN.*ENV_LIMITATION"; then
    RESULTS["e2e_run"]="NOT_RUN_ENV_LIMITATION"
  else
    RESULTS["e2e_run"]="FAIL"
  fi
  
  echo "CI/E2E probe completed (see summary below)"
else
  WARNINGS+=("scripts/ci_e2e_probe.sh not found")
  echo -e "${YELLOW}NOT_RUN (script not found)${NC}"
fi

echo ""

# Summary
echo "============================================================"
echo "RELEASE PROBE SUMMARY"
echo "============================================================"
echo ""
echo "Timestamp: $TIMESTAMP"
echo "HEAD: ${HEAD_SHA:0:8} ($HEAD_MSG)"
echo "Branch: $CURRENT_BRANCH"
echo "Working tree clean: $([ "$WORKING_TREE_CLEAN" = true ] && echo "YES" || echo "NO")"
echo ""

echo "Quick Smoke:"
printf "  scripts/quick_smoke.sh: %s\n" "${RESULTS["quick_smoke"]:-UNKNOWN}"
echo ""

echo "CI/E2E Probe (scripts/ci_e2e_probe.sh):"
printf "  governance_shield: %s\n" "${RESULTS["governance_shield"]:-UNKNOWN}"
printf "  cargo fmt:         %s\n" "${RESULTS["cargo_fmt"]:-UNKNOWN}"
printf "  npm build:         %s\n" "${RESULTS["npm_build"]:-UNKNOWN}"
printf "  e2e run:           %s\n" "${RESULTS["e2e_run"]:-UNKNOWN}"
echo ""

if [ ${#WARNINGS[@]} -gt 0 ]; then
  echo "Notes:"
  for warning in "${WARNINGS[@]}"; do
    echo "  - WARN: $warning"
  done
  echo ""
fi

echo "============================================================"
echo ""

# Always exit 0 (this is a probe, not a strict gate)
exit 0

