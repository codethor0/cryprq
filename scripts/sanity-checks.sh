#!/bin/bash
set -euo pipefail

# One-Time Sanity Checks Before Shipping
# Run these checks manually before release

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🔒 Pre-Release Sanity Checks"
echo "=============================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

success() {
  echo -e "${GREEN}✅${NC} $1"
}

warning() {
  echo -e "${YELLOW}⚠️${NC} $1"
}

error() {
  echo -e "${RED}❌${NC} $1"
}

info() {
  echo -e "ℹ️  $1"
}

# Check 1: Kill-Switch
echo "1. Kill-Switch Test"
echo "-------------------"
info "Manual test required:"
echo "  1. Start CrypRQ"
echo "  2. Connect to a peer"
echo "  3. Quit app (Cmd+Q / Alt+F4)"
echo "  4. Verify session stops within ≤1s"
echo ""
read -p "Did kill-switch work? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  success "Kill-switch verified"
else
  error "Kill-switch failed - check implementation"
fi

# Check 2: HTTPS Enforcement
echo ""
echo "2. HTTPS Enforcement Test"
echo "--------------------------"
info "Manual test required:"
echo "  Desktop:"
echo "    1. Settings → Security → Manage allowlist → Add 'example.com'"
echo "    2. Try REMOTE endpoint: http://example.com"
echo "    3. Verify inline error shown"
echo ""
echo "  Mobile:"
echo "    1. Settings → Profile → REMOTE"
echo "    2. Enter http://example.com"
echo "    3. Verify validation error shown"
echo ""
read -p "Did HTTPS enforcement work? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  success "HTTPS enforcement verified"
else
  error "HTTPS enforcement failed - check validation"
fi

# Check 3: Redaction
echo ""
echo "3. Redaction Check"
echo "------------------"
info "Export diagnostics ZIP, then run:"
echo "  unzip -q cryprq-diagnostics-*.zip -d /tmp/redact-check"
echo "  grep -r -E 'bearer |privKey=|authorization:' /tmp/redact-check || echo '✅ No secrets'"
echo ""
read -p "Did redaction check pass? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  success "Redaction verified"
else
  error "Redaction failed - secrets found in diagnostics"
fi

# Check 4: Crash Symbols
echo ""
echo "4. Crash Symbols Check"
echo "----------------------"
info "Check GitHub Actions logs for:"
echo "  - macOS: 'Uploading dSYM' step"
echo "  - Android: 'Uploading ProGuard mapping' step"
echo ""
read -p "Did crash symbol uploads run? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  success "Crash symbols verified"
else
  warning "Crash symbols not uploaded (may be OK if secrets missing)"
fi

# Check 5: Structured Logs
echo ""
echo "5. Structured Logs Check"
echo "------------------------"
if [ -d ~/.cryprq/logs ]; then
  LOG_FILE=$(find ~/.cryprq/logs -name "cryprq-*.log" -type f -mtime -1 | head -1)
  if [ -n "$LOG_FILE" ]; then
    STRUCTURED_COUNT=$(jq -c 'fromjson | select(.v==1)' "$LOG_FILE" 2>/dev/null | wc -l || echo "0")
    if [ "$STRUCTURED_COUNT" -gt 0 ]; then
      success "Structured logs (JSONL v1) found: $STRUCTURED_COUNT entries"
    else
      warning "No structured logs found (may be OK if no activity)"
    fi
  else
    warning "No recent log files found"
  fi
else
  warning "Log directory not found: ~/.cryprq/logs"
fi

# Check 6: Feature Flags & Telemetry Smoke (Optional, 30 seconds)
echo ""
echo "6. Feature Flags & Telemetry Smoke (Optional)"
echo "----------------------------------------------"
read -p "Run flags/telemetry smoke test? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  info "Running 30-second flags/telemetry smoke..."
  
  # Check flags file exists and is valid JSON
  if [ -f "$REPO_ROOT/config/flags.json" ]; then
    if command -v jq >/dev/null; then
      if jq . "$REPO_ROOT/config/flags.json" >/dev/null 2>&1; then
        success "Flags file valid JSON"
        FLAGS_CONTENT=$(jq -c . "$REPO_ROOT/config/flags.json")
        info "Flags: $FLAGS_CONTENT"
      else
        error "Flags file invalid JSON"
      fi
    else
      warning "jq not available - skipping flags validation"
    fi
  else
    warning "Flags file not found: config/flags.json"
  fi
  
  # Check telemetry directory (if telemetry was enabled)
  if [ -d ~/.cryprq/telemetry ]; then
    success "Telemetry directory exists"
    
    # Check latest telemetry file
    TODAY_FILE="$HOME/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl"
    if [ -f "$TODAY_FILE" ]; then
      if command -v jq >/dev/null; then
        COUNT=$(wc -l < "$TODAY_FILE" 2>/dev/null || echo "0")
        if [ "$COUNT" -gt 0 ]; then
          success "Telemetry active: $COUNT events today"
          
          # Check for redaction (should not find secrets)
          if grep -q -E "bearer |privKey=|authorization:" "$TODAY_FILE" 2>/dev/null; then
            error "Secrets found in telemetry file (redaction failed)"
          else
            success "Telemetry redaction OK (no secrets found)"
          fi
          
          # Show sample events
          info "Sample events:"
          head -3 "$TODAY_FILE" | jq -c '{event,ts,appVersion}' 2>/dev/null || head -3 "$TODAY_FILE"
        else
          warning "Telemetry file exists but is empty"
        fi
      else
        COUNT=$(wc -l < "$TODAY_FILE" 2>/dev/null || echo "0")
        if [ "$COUNT" -gt 0 ]; then
          success "Telemetry active: $COUNT events today (jq not available for details)"
        fi
      fi
    else
      info "No telemetry events today (telemetry may be disabled or no activity)"
    fi
  else
    info "Telemetry directory not found (telemetry not enabled - this is OK, default is OFF)"
  fi
  
  success "Flags/telemetry smoke test complete"
else
  info "Skipped flags/telemetry smoke test"
fi

# Summary
echo ""
echo "=============================="
echo "Sanity Checks Summary"
echo "=============================="
echo ""
echo "All checks completed. Review results above."
echo ""
echo "If any checks failed, fix issues before release."
echo "See docs/GO_LIVE_SEQUENCE.md for detailed instructions."

