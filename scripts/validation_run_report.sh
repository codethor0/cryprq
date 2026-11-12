#!/usr/bin/env bash
set -euo pipefail

# CrypRQ Validation Run Report Generator
# Generate a Markdown validation run report combining probe outputs.
#
# Security and privacy:
# - Does not capture environment variables
# - Does not send data off-host
# - Uses only existing scripts/tools and writes to docs/
#
# Scope:
# - Intended to be run by maintainers before or after a release, not as a CI gate
# - Always exits 0 unless script itself fails (syntax error, missing file, etc.)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Get HEAD info
HEAD_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
HEAD_SHORT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
HEAD_MSG=$(git log -1 --oneline 2>/dev/null || echo "unknown")

# Compute timestamp
TS=$(date -u +"%Y%m%d_%H%M%SZ")
TS_READABLE=$(date -u +"%Y-%m-%d %H:%M:%S UTC")

# Set output path
OUT_DOC="${REPO_ROOT}/docs/VALIDATION_RUN_${TS}_${HEAD_SHORT}.md"

# Helper function to run and capture command output
run_capture() {
  local label="$1"
  shift
  local cmd=("$@")
  local output=""
  
  echo "[REPORT] Running: ${cmd[*]}" >&2
  
  if [ -f "${cmd[0]}" ] || command -v "${cmd[0]}" >/dev/null 2>&1; then
    # Run command and capture output; never fail the whole script
    output=$("${cmd[@]}" 2>&1) || {
      echo "[WARN] ${label} exited non-zero (recorded above; may be ENV LIMITATION or product issue)." >&2
      echo "$output"
      echo "[WARN] ${label} exited non-zero (recorded above; may be ENV LIMITATION or product issue)."
      return 0
    }
    echo "$output"
  else
    echo "[WARN] ${label}: command not found." >&2
    echo "[WARN] ${label}: command not found."
  fi
}

# Run probes and capture output
echo "[REPORT] Generating validation run report..."
echo "[REPORT] Output will be written to: $OUT_DOC"
echo ""

QS_OUT=""
CI_PROBE_OUT=""
REL_PROBE_OUT=""
VPN_PROBE_OUT=""

# Quick smoke
if [ -f "${REPO_ROOT}/scripts/quick_smoke.sh" ]; then
  QS_OUT=$(run_capture "QUICK_SMOKE" bash "${REPO_ROOT}/scripts/quick_smoke.sh" || true)
else
  QS_OUT="[WARN] scripts/quick_smoke.sh not found"
fi

# CI/E2E probe
if [ -f "${REPO_ROOT}/scripts/ci_e2e_probe.sh" ]; then
  CI_PROBE_OUT=$(run_capture "CI_E2E_PROBE" bash "${REPO_ROOT}/scripts/ci_e2e_probe.sh" || true)
else
  CI_PROBE_OUT="[WARN] scripts/ci_e2e_probe.sh not found"
fi

# Release probe
if [ -f "${REPO_ROOT}/scripts/release_probe.sh" ]; then
  REL_PROBE_OUT=$(run_capture "RELEASE_PROBE" bash "${REPO_ROOT}/scripts/release_probe.sh" || true)
else
  REL_PROBE_OUT="[WARN] scripts/release_probe.sh not found"
fi

# VPN host probe
if [ -f "${REPO_ROOT}/scripts/vpn_host_probe.sh" ]; then
  VPN_PROBE_OUT=$(run_capture "VPN_HOST_PROBE" bash "${REPO_ROOT}/scripts/vpn_host_probe.sh" || true)
else
  VPN_PROBE_OUT="[WARN] scripts/vpn_host_probe.sh not found"
fi

# Generate report
cat > "$OUT_DOC" <<EOF
# CrypRQ Validation Run Report

STATUS: TODO (GREEN/YELLOW/RED) — <fill in after review>

HEAD:
- Commit: \`${HEAD_SHA}\`
- Short: \`${HEAD_SHORT}\`
- Subject: ${HEAD_MSG}
- Timestamp (UTC): ${TS_READABLE}

WHAT I RAN (automated via scripts/validation_run_report.sh):
- scripts/quick_smoke.sh
- scripts/ci_e2e_probe.sh
- scripts/release_probe.sh
- scripts/vpn_host_probe.sh

ENVIRONMENT & LIMITATIONS (auto-notes):
- Note: Probe scripts may classify TLS/SSL, missing Docker/TUN, or npm permission issues as ENV LIMITATION, not product bugs.
- Review the probe outputs below for details.

## QUICK SMOKE OUTPUT

\`\`\`
${QS_OUT}
\`\`\`

## CI/E2E PROBE OUTPUT

\`\`\`
${CI_PROBE_OUT}
\`\`\`

## RELEASE PROBE OUTPUT

\`\`\`
${REL_PROBE_OUT}
\`\`\`

## VPN HOST PROBE OUTPUT

\`\`\`
${VPN_PROBE_OUT}
\`\`\`

## NEXT ACTION (manual):

- Fill in STATUS line (GREEN/YELLOW/RED) based on probe results
- Copy relevant excerpts into release notes or validation docs as needed
- Attach this file and a support bundle when filing bugs
- See \`docs/VALIDATION_RUN_TEMPLATE.md\` for a complete template structure

## Additional Resources:

- \`docs/VALIDATION_RUN_TEMPLATE.md\`: Template for manual validation runs
- \`docs/RELEASE_CHECKLIST.md\`: Release-specific validation procedures
- \`docs/CI_E2E_INVESTIGATION_RUNBOOK.md\`: How to investigate CI/E2E failures
- \`scripts/support_bundle.sh\`: Automated support bundle collection
EOF

echo "[REPORT] Validation run report generated: $OUT_DOC"
echo "[REPORT] Review the report and fill in the STATUS line manually."

# Always exit 0 (this is a report generator, not a strict gate)
exit 0

