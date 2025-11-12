#!/bin/bash
set -euo pipefail

# Observability Quick Checks
# Run these after release to verify logs and redaction

echo "🔍 Running observability checks..."
echo ""

# Desktop logs sanity
echo "📊 Desktop Logs Sanity:"
if [ -d ~/.cryprq/logs ]; then
  jq -cr 'fromjson | select(.event=="session.state") | [.ts,.data.state] | @tsv' ~/.cryprq/logs/cryprq-*.log 2>/dev/null | tail -10 || echo "⚠️  No structured logs found or jq not installed"
else
  echo "⚠️  Log directory not found: ~/.cryprq/logs"
fi

echo ""

# Redaction guard
echo "🔒 Redaction Guard:"
if grep -R -E "bearer |privKey=|authorization:" ~/.cryprq/logs 2>/dev/null; then
  echo "❌ Secrets leaked in logs!"
  exit 1
else
  echo "✅ Redaction OK - No secrets found in logs"
fi

echo ""

# Structured log adoption
echo "📈 Structured Log Adoption:"
TOTAL=$(find ~/.cryprq/logs -name "cryprq-*.log" -type f 2>/dev/null | wc -l)
STRUCTURED=$(jq -c 'fromjson | select(.v==1)' ~/.cryprq/logs/cryprq-*.log 2>/dev/null | wc -l || echo "0")
if [ "$TOTAL" -gt 0 ]; then
  echo "Total log files: $TOTAL"
  echo "Structured entries: $STRUCTURED"
else
  echo "⚠️  No log files found"
fi

echo ""

# Post-ship observability metrics (when telemetry v0 is enabled)
# Uncomment these checks once telemetry is toggled on:
#
# echo ""
# echo "📊 Health KPIs (Telemetry v0):"
# echo "  Stability: Crash-free sessions ≥99.5%"
# echo "  Connectivity: Connect success ≥99%"
# echo "  Performance: Median latency <150ms"
# echo "  Security: Redaction/Audit checks 100% pass"
# echo "  UX: Report-issue success rate ≥98%"
#
# # Example telemetry checks (when implemented):
# # TELEMETRY_DIR="${TELEMETRY_DIR:-~/.cryprq/telemetry}"
# # if [ -d "$TELEMETRY_DIR" ]; then
# #   CONNECT_COUNT=$(jq -cr 'fromjson | select(.event=="connect")' "$TELEMETRY_DIR"/events-*.jsonl 2>/dev/null | wc -l || echo "0")
# #   DISCONNECT_COUNT=$(jq -cr 'fromjson | select(.event=="disconnect")' "$TELEMETRY_DIR"/events-*.jsonl 2>/dev/null | wc -l || echo "0")
# #   ERROR_COUNT=$(jq -cr 'fromjson | select(.event=="error")' "$TELEMETRY_DIR"/events-*.jsonl 2>/dev/null | wc -l || echo "0")
# #   ROTATION_COUNT=$(jq -cr 'fromjson | select(.event=="rotation.completed")' "$TELEMETRY_DIR"/events-*.jsonl 2>/dev/null | wc -l || echo "0")
# #   echo "  Connect events: $CONNECT_COUNT"
# #   echo "  Disconnect events: $DISCONNECT_COUNT"
# #   echo "  Error events: $ERROR_COUNT"
# #   echo "  Rotation events: $ROTATION_COUNT"
# #   if [ "$CONNECT_COUNT" -gt 0 ]; then
# #     SUCCESS_RATE=$(echo "scale=2; ($CONNECT_COUNT - $ERROR_COUNT) * 100 / $CONNECT_COUNT" | bc)
# #     echo "  Connect success rate: ${SUCCESS_RATE}%"
# #   fi
# # fi

echo ""
echo "✅ Observability checks complete"

