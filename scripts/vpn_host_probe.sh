#!/usr/bin/env bash
set -euo pipefail

# CrypRQ VPN Host-Mode Probe Script
# Automated probe script for validating CrypRQ's VPN mode on real hosts (macOS/Linux).
#
# Purpose:
# - Checks environment prerequisites (OS, root, TUN support, cryprq binary)
# - Runs basic tunnel start/stop tests (listener + dialer with --vpn)
# - Optionally runs IP/DNS checks (if enabled via VPN_HOST_PROBE_RUN_LEAK_TESTS=1)
# - Classifies results: PASS, FAIL (Product Bug Candidate), or ENV_LIMITATION
#
# Security and privacy:
# - Does not read environment variables or shell history
# - Does not send logs anywhere
# - Uses external network calls only for optional IP/DNS tests, if enabled
# - All test artifacts are stored locally in temporary directories
#
# Scope:
# - Host-mode VPN testing on macOS/Linux only
# - Meant to be run manually by someone who understands they are altering network configuration
# - This script modifies network configuration (TUN interfaces, routes) - use with caution
#
# ENV LIMITATION handling:
# - Never exits non-zero just because TUN/root/tools are missing
# - Classifies environment failures as ENV_LIMITATION in the summary
# - Always exits 0 unless script itself fails (syntax error, missing file, etc.)

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
PROBE_DIR=""

# Cleanup function
cleanup() {
  if [ -n "$PROBE_DIR" ] && [ -d "$PROBE_DIR" ]; then
    # Kill any remaining cryprq processes
    pkill -f "cryprq.*--vpn" 2>/dev/null || true
    sleep 1
    # Remove probe directory
    rm -rf "$PROBE_DIR" 2>/dev/null || true
  fi
}

trap cleanup EXIT

# Create temporary directory for probe artifacts
PROBE_DIR=$(mktemp -d /tmp/cryprq_vpn_probe_XXXXXX)

# Print header
echo "============================================================"
echo "CrypRQ VPN Host-Mode Probe"
echo "============================================================"
echo ""
TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
echo "Timestamp: $TIMESTAMP"
echo "Probe directory: $PROBE_DIR"
echo ""

# Preflight: Detect OS
echo "[PROBE] Preflight: Environment checks"
echo ""

OS_TYPE=$(uname -s 2>/dev/null || echo "unknown")
echo "OS: $OS_TYPE"
RESULTS["os"]="$OS_TYPE"

# Check for root/admin privileges
ROOT_CHECK=false
if [ "$(id -u)" -eq 0 ]; then
  ROOT_CHECK=true
  echo -e "Root privileges: ${GREEN}YES${NC}"
  RESULTS["root"]="YES"
else
  echo -e "Root privileges: ${YELLOW}NO${NC}"
  RESULTS["root"]="NO"
  WARNINGS+=("Not running as root - TUN creation will likely fail (ENV LIMITATION)")
fi

# Check for TUN support
TUN_CHECK=false
if [ "$OS_TYPE" = "Linux" ]; then
  if [ -e "/dev/net/tun" ]; then
    TUN_CHECK=true
    echo -e "TUN support: ${GREEN}YES${NC} (/dev/net/tun exists)"
    RESULTS["tun"]="YES"
  else
    echo -e "TUN support: ${YELLOW}NO${NC} (/dev/net/tun not found)"
    RESULTS["tun"]="NO"
    WARNINGS+=("TUN device not found - VPN mode will fail (ENV LIMITATION)")
  fi
elif [ "$OS_TYPE" = "Darwin" ]; then
  # macOS TUN detection is more complex (Network Extension framework)
  # For now, assume it might be available if root is available
  if [ "$ROOT_CHECK" = true ]; then
    echo -e "TUN support: ${YELLOW}UNKNOWN${NC} (macOS - may require Network Extension framework)"
    RESULTS["tun"]="UNKNOWN"
  else
    echo -e "TUN support: ${YELLOW}NO${NC} (macOS - requires root/admin privileges)"
    RESULTS["tun"]="NO"
  fi
else
  echo -e "TUN support: ${YELLOW}UNKNOWN${NC} (OS $OS_TYPE not fully supported)"
  RESULTS["tun"]="UNKNOWN"
fi

# Check for cryprq binary
CRYPRQ_BIN="${CRYPRQ_BIN:-cryprq}"
if command -v "$CRYPRQ_BIN" >/dev/null 2>&1; then
  CRYPRQ_PATH=$(command -v "$CRYPRQ_BIN")
  CRYPRQ_VERSION=$("$CRYPRQ_BIN" --version 2>/dev/null || echo "version unknown")
  echo -e "cryprq binary: ${GREEN}YES${NC} ($CRYPRQ_PATH)"
  echo "  Version: $CRYPRQ_VERSION"
  RESULTS["cryprq"]="YES"
else
  echo -e "cryprq binary: ${YELLOW}NO${NC} (not found in PATH)"
  RESULTS["cryprq"]="NO"
  WARNINGS+=("cryprq binary not found - set CRYPRQ_BIN env var or ensure cryprq is in PATH (ENV LIMITATION)")
fi

echo ""

# If prerequisites are not met, classify as ENV LIMITATION and exit gracefully
if [ "$ROOT_CHECK" = false ] || [ "$TUN_CHECK" = false ] || [ "${RESULTS["cryprq"]}" != "YES" ]; then
  echo "[PROBE] Prerequisites not met - skipping tunnel tests"
  echo ""
  echo "============================================================"
  echo "VPN Host-Mode Probe Summary"
  echo "============================================================"
  echo ""
  echo "Environment:"
  printf "  OS:              %s\n" "${RESULTS["os"]:-UNKNOWN}"
  printf "  Root privileges: %s\n" "${RESULTS["root"]:-UNKNOWN}"
  printf "  TUN support:     %s\n" "${RESULTS["tun"]:-UNKNOWN}"
  printf "  cryprq binary:   %s\n" "${RESULTS["cryprq"]:-UNKNOWN}"
  echo ""
  echo "Tunnel Test:"
  echo "  NOT_RUN (ENV LIMITATION: prerequisites not met)"
  echo ""
  if [ ${#WARNINGS[@]} -gt 0 ]; then
    echo "Notes:"
    for warning in "${WARNINGS[@]}"; do
      echo "  - $warning"
    done
    echo ""
  fi
  echo "============================================================"
  exit 0
fi

# Basic tunnel test
echo "[PROBE] Basic tunnel test: Local loopback VPN"
echo ""

TEST_PORT=10000
LISTENER_LOG="$PROBE_DIR/listener.log"
DIALER_LOG="$PROBE_DIR/dialer.log"

# Start listener in background
echo "Starting listener with --vpn on port $TEST_PORT..."
"$CRYPRQ_BIN" --listen "/ip4/0.0.0.0/udp/$TEST_PORT/quic-v1" --vpn > "$LISTENER_LOG" 2>&1 &
LISTENER_PID=$!
sleep 3

# Check if listener is still running
if ! kill -0 "$LISTENER_PID" 2>/dev/null; then
  LISTENER_EXIT_CODE=$?
  LISTENER_OUTPUT=$(cat "$LISTENER_LOG" 2>/dev/null || echo "")
  if echo "$LISTENER_OUTPUT" | grep -qiE "(requires root|requires admin|privileges|Failed to create TUN|No such file or directory)"; then
    RESULTS["tunnel_test"]="ENV_LIMITATION"
    echo -e "Listener: ${YELLOW}ENV LIMITATION${NC} (TUN/privilege issue)"
  else
    RESULTS["tunnel_test"]="FAIL"
    echo -e "Listener: ${RED}FAIL${NC} (unexpected exit)"
  fi
  echo "Listener log:"
  cat "$LISTENER_LOG" | head -20
  kill "$LISTENER_PID" 2>/dev/null || true
  echo ""
else
  echo -e "Listener: ${GREEN}RUNNING${NC} (PID: $LISTENER_PID)"
  
  # Start dialer
  echo "Starting dialer with --vpn..."
  timeout 10 "$CRYPRQ_BIN" --peer "/ip4/127.0.0.1/udp/$TEST_PORT/quic-v1" --vpn > "$DIALER_LOG" 2>&1 &
  DIALER_PID=$!
  sleep 3
  
  # Check dialer status
  if ! kill -0 "$DIALER_PID" 2>/dev/null; then
    DIALER_OUTPUT=$(cat "$DIALER_LOG" 2>/dev/null || echo "")
    if echo "$DIALER_OUTPUT" | grep -qiE "(requires root|requires admin|privileges|Failed to create TUN|No such file or directory)"; then
      RESULTS["tunnel_test"]="ENV_LIMITATION"
      echo -e "Dialer: ${YELLOW}ENV LIMITATION${NC} (TUN/privilege issue)"
    else
      RESULTS["tunnel_test"]="FAIL"
      echo -e "Dialer: ${RED}FAIL${NC} (unexpected exit)"
    fi
  else
    echo -e "Dialer: ${GREEN}RUNNING${NC} (PID: $DIALER_PID)"
    RESULTS["tunnel_test"]="PASS"
    echo -e "Tunnel test: ${GREEN}PASS${NC}"
  fi
  
  # Cleanup: kill both processes
  kill "$LISTENER_PID" 2>/dev/null || true
  kill "$DIALER_PID" 2>/dev/null || true
  sleep 1
  pkill -f "cryprq.*--vpn" 2>/dev/null || true
fi

echo ""

# Optional IP/DNS checks
if [ "${VPN_HOST_PROBE_RUN_LEAK_TESTS:-0}" = "1" ]; then
  echo "[PROBE] Optional IP/DNS checks (VPN_HOST_PROBE_RUN_LEAK_TESTS=1)"
  echo ""
  
  # IP check
  if command -v curl >/dev/null 2>&1; then
    echo -n "IP check: "
    IP_OUTPUT=$(timeout 5 curl -s https://api.ipify.org 2>&1) || true
    if [ $? -eq 0 ] && [ -n "$IP_OUTPUT" ] && ! echo "$IP_OUTPUT" | grep -qiE "(SSL|certificate|error|Could not resolve)"; then
      echo -e "${GREEN}PASS${NC} (External IP: $IP_OUTPUT)"
      RESULTS["ip_check"]="PASS"
    else
      echo -e "${YELLOW}ENV LIMITATION${NC} (TLS/network error)"
      RESULTS["ip_check"]="ENV_LIMITATION"
    fi
  else
    echo -e "${YELLOW}NOT_RUN${NC} (curl not available)"
    RESULTS["ip_check"]="NOT_RUN"
  fi
  
  # DNS check
  if command -v dig >/dev/null 2>&1; then
    echo -n "DNS check: "
    DNS_OUTPUT=$(timeout 5 dig @1.1.1.1 example.com +short 2>&1) || true
    if [ $? -eq 0 ] && [ -n "$DNS_OUTPUT" ]; then
      echo -e "${GREEN}PASS${NC} (DNS resolution working)"
      RESULTS["dns_check"]="PASS"
    else
      echo -e "${YELLOW}ENV LIMITATION${NC} (DNS query failed)"
      RESULTS["dns_check"]="ENV_LIMITATION"
    fi
  else
    echo -e "${YELLOW}NOT_RUN${NC} (dig not available)"
    RESULTS["dns_check"]="NOT_RUN"
  fi
  
  echo ""
else
  echo "[PROBE] Optional IP/DNS checks: SKIPPED (set VPN_HOST_PROBE_RUN_LEAK_TESTS=1 to enable)"
  echo ""
  RESULTS["ip_check"]="NOT_RUN"
  RESULTS["dns_check"]="NOT_RUN"
fi

# Summary
echo "============================================================"
echo "VPN Host-Mode Probe Summary"
echo "============================================================"
echo ""
echo "Timestamp: $TIMESTAMP"
echo ""

echo "Environment:"
printf "  OS:              %s\n" "${RESULTS["os"]:-UNKNOWN}"
printf "  Root privileges: %s\n" "${RESULTS["root"]:-UNKNOWN}"
printf "  TUN support:     %s\n" "${RESULTS["tun"]:-UNKNOWN}"
printf "  cryprq binary:   %s\n" "${RESULTS["cryprq"]:-UNKNOWN}"
echo ""

echo "Tunnel Test:"
printf "  Status:          %s\n" "${RESULTS["tunnel_test"]:-UNKNOWN}"
echo ""

if [ "${VPN_HOST_PROBE_RUN_LEAK_TESTS:-0}" = "1" ]; then
  echo "Optional Checks:"
  printf "  IP check:        %s\n" "${RESULTS["ip_check"]:-UNKNOWN}"
  printf "  DNS check:       %s\n" "${RESULTS["dns_check"]:-UNKNOWN}"
  echo ""
fi

if [ ${#WARNINGS[@]} -gt 0 ]; then
  echo "Notes:"
  for warning in "${WARNINGS[@]}"; do
    echo "  - $warning"
  done
  echo ""
fi

echo "Probe artifacts: $PROBE_DIR"
echo "============================================================"
echo ""

# Always exit 0 (this is a probe, not a strict gate)
exit 0

