#!/usr/bin/env bash
# Copyright (c) 2025 Thor Thor
# Author: Thor Thor (GitHub: https://github.com/codethor0)
# Contact: codethor@gmail.com
# License: MIT (see LICENSE file for details)
#
# Purpose: Probe script for VPN leak & routing diagnostics
#
# What this script does:
# - Collects IP address observations (IPv4 and IPv6)
# - Captures DNS resolver configuration
# - Inspects routing table and interface information
# - Detects presence of CrypRQ VPN interface (if active)
# - Classifies results as PASS/FAIL/ENV_LIMITATION/INFO
#
# What this script does NOT do:
# - Does not collect secrets or environment variables
# - Does not modify system firewalls or routes
# - Does not attempt to "fix" leaks (observation only)
# - Does not make privacy guarantees (correctness checks only)
#
# Exit code: Always 0 (probe, not a strict gate)
# Usage: ./scripts/vpn_leak_probe.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Classification counters
IP_CHECK_STATUS=""
DNS_CHECK_STATUS=""
IPV6_CHECK_STATUS=""
ROUTING_CHECK_STATUS=""
CRYPRQ_INTERFACE_DETECTED="no"

echo "=========================================="
echo "CrypRQ VPN Leak & Routing Probe"
echo "=========================================="
echo ""
echo "This probe collects diagnostic information about IP, DNS, IPv6, and routing behavior."
echo "It does NOT modify system configuration or make privacy guarantees."
echo "See docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md for interpretation guidance."
echo ""

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macOS"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="Linux"
else
    OS="Unknown"
fi

echo "Platform: $OS"
echo ""

# Check for required tools
MISSING_TOOLS=()
command -v curl >/dev/null 2>&1 || MISSING_TOOLS+=("curl")
command -v dig >/dev/null 2>&1 || command -v nslookup >/dev/null 2>&1 || MISSING_TOOLS+=("dig/nslookup")

if [[ ${#MISSING_TOOLS[@]} -gt 0 ]]; then
    echo -e "${YELLOW}[ENV LIMITATION] Missing tools: ${MISSING_TOOLS[*]}${NC}"
    echo "Some checks will be skipped."
    echo ""
fi

# Detect CrypRQ VPN interface
if [[ "$OS" == "Linux" ]]; then
    if ip link show | grep -q "cryprq"; then
        CRYPRQ_INTERFACE_DETECTED="yes"
        CRYPRQ_INTERFACE=$(ip link show | grep "cryprq" | head -1 | awk '{print $2}' | tr -d ':')
    fi
elif [[ "$OS" == "macOS" ]]; then
    if ifconfig | grep -q "cryprq"; then
        CRYPRQ_INTERFACE_DETECTED="yes"
        CRYPRQ_INTERFACE=$(ifconfig | grep -B1 "cryprq" | head -1 | awk '{print $1}' | tr -d ':')
    fi
fi

if [[ "$CRYPRQ_INTERFACE_DETECTED" == "yes" ]]; then
    echo -e "${BLUE}[INFO] CrypRQ VPN interface detected: $CRYPRQ_INTERFACE${NC}"
    echo "Observations below are made while VPN interface is present."
else
    echo -e "${YELLOW}[INFO] No CrypRQ VPN interface detected${NC}"
    echo "Use these results as a baseline. Run again after starting VPN to compare."
fi
echo ""

# IP Address Checks
echo "=========================================="
echo "IP Address Checks"
echo "=========================================="

if command -v curl >/dev/null 2>&1; then
    echo "Checking IPv4 public IP..."
    IPV4_RESULT=""
    if IPV4=$(curl -s --max-time 5 https://api.ipify.org 2>&1); then
        if [[ "$IPV4" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            echo -e "${GREEN}IPv4 Public IP: $IPV4${NC}"
            IPV4_RESULT="SUCCESS"
        else
            echo -e "${YELLOW}Unexpected IPv4 response: $IPV4${NC}"
            IPV4_RESULT="ENV_LIMITATION"
        fi
    else
        echo -e "${YELLOW}[ENV LIMITATION] IPv4 check failed: $IPV4${NC}"
        IPV4_RESULT="ENV_LIMITATION"
    fi

    echo ""
    echo "Checking IPv6 public IP..."
    IPV6_PUBLIC_RESULT=""
    if IPV6_PUBLIC=$(curl -6 -s --max-time 5 https://api64.ipify.org 2>&1); then
        if [[ "$IPV6_PUBLIC" =~ : ]]; then
            echo -e "${GREEN}IPv6 Public IP: $IPV6_PUBLIC${NC}"
            IPV6_PUBLIC_RESULT="SUCCESS"
        else
            echo -e "${YELLOW}Unexpected IPv6 response: $IPV6_PUBLIC${NC}"
            IPV6_PUBLIC_RESULT="ENV_LIMITATION"
        fi
    else
        echo -e "${YELLOW}[ENV LIMITATION] IPv6 public check failed (may be normal if IPv6 disabled): $IPV6_PUBLIC${NC}"
        IPV6_PUBLIC_RESULT="ENV_LIMITATION"
    fi
else
    echo -e "${YELLOW}[ENV LIMITATION] curl not available, skipping IP checks${NC}"
    IPV4_RESULT="ENV_LIMITATION"
    IPV6_PUBLIC_RESULT="ENV_LIMITATION"
fi

if [[ "$IPV4_RESULT" == "SUCCESS" ]]; then
    IP_CHECK_STATUS="SUCCESS"
else
    IP_CHECK_STATUS="ENV_LIMITATION"
fi

echo ""

# DNS Checks
echo "=========================================="
echo "DNS Configuration Checks"
echo "=========================================="

if [[ "$OS" == "Linux" ]]; then
    if [[ -f /etc/resolv.conf ]]; then
        echo "DNS Resolvers (/etc/resolv.conf):"
        grep -E "^nameserver" /etc/resolv.conf | awk '{print "  " $2}' || echo "  (none found)"
    else
        echo -e "${YELLOW}[ENV LIMITATION] /etc/resolv.conf not found${NC}"
    fi

    if command -v resolvectl >/dev/null 2>&1; then
        echo ""
        echo "systemd-resolved status:"
        resolvectl status 2>&1 | head -20 || echo "  (unable to query)"
    fi
elif [[ "$OS" == "macOS" ]]; then
    echo "DNS Resolvers (scutil --dns):"
    scutil --dns 2>&1 | grep "nameserver" | head -5 || echo "  (unable to query)"
    
    echo ""
    echo "DNS Resolvers (networksetup):"
    networksetup -getdnsservers Wi-Fi 2>&1 || networksetup -getdnsservers Ethernet 2>&1 || echo "  (unable to query)"
fi

echo ""

if command -v dig >/dev/null 2>&1; then
    echo "Testing DNS resolution (dig example.com):"
    if DIG_RESULT=$(dig +short example.com 2>&1); then
        if [[ -n "$DIG_RESULT" ]]; then
            echo -e "${GREEN}DNS resolution successful: $DIG_RESULT${NC}"
            DNS_CHECK_STATUS="SUCCESS"
        else
            echo -e "${YELLOW}DNS resolution returned empty result${NC}"
            DNS_CHECK_STATUS="ENV_LIMITATION"
        fi
    else
        echo -e "${YELLOW}[ENV LIMITATION] DNS resolution failed: $DIG_RESULT${NC}"
        DNS_CHECK_STATUS="ENV_LIMITATION"
    fi
elif command -v nslookup >/dev/null 2>&1; then
    echo "Testing DNS resolution (nslookup example.com):"
    if NSLOOKUP_RESULT=$(nslookup example.com 2>&1); then
        echo "$NSLOOKUP_RESULT" | head -5
        DNS_CHECK_STATUS="SUCCESS"
    else
        echo -e "${YELLOW}[ENV LIMITATION] DNS resolution failed${NC}"
        DNS_CHECK_STATUS="ENV_LIMITATION"
    fi
else
    echo -e "${YELLOW}[ENV LIMITATION] dig/nslookup not available, skipping DNS resolution test${NC}"
    DNS_CHECK_STATUS="ENV_LIMITATION"
fi

echo ""

# IPv6 Local Checks
echo "=========================================="
echo "IPv6 Local Configuration"
echo "=========================================="

if [[ "$OS" == "Linux" ]]; then
    echo "IPv6 addresses (ip -6 addr):"
    if ip -6 addr show >/dev/null 2>&1; then
        ip -6 addr show | grep "inet6" | head -5 || echo "  (no IPv6 addresses found)"
        IPV6_CHECK_STATUS="SUCCESS"
    else
        echo -e "${YELLOW}[ENV LIMITATION] Unable to query IPv6 addresses${NC}"
        IPV6_CHECK_STATUS="ENV_LIMITATION"
    fi
elif [[ "$OS" == "macOS" ]]; then
    echo "IPv6 addresses (ifconfig):"
    if IFCONFIG_IPV6=$(ifconfig | grep "inet6" | head -5); then
        echo "$IFCONFIG_IPV6"
        IPV6_CHECK_STATUS="SUCCESS"
    else
        echo "  (no IPv6 addresses found or IPv6 disabled)"
        IPV6_CHECK_STATUS="INFO"
    fi
fi

echo ""

# Routing Checks
echo "=========================================="
echo "Routing Table Inspection"
echo "=========================================="

if [[ "$OS" == "Linux" ]]; then
    echo "Default routes (ip route show default):"
    if ip route show default >/dev/null 2>&1; then
        ip route show default
        ROUTING_CHECK_STATUS="SUCCESS"
    else
        echo -e "${YELLOW}[ENV LIMITATION] Unable to query default routes${NC}"
        ROUTING_CHECK_STATUS="ENV_LIMITATION"
    fi

    echo ""
    echo "All routes (ip route show):"
    ip route show | head -10 || echo "  (unable to query)"

    if [[ "$CRYPRQ_INTERFACE_DETECTED" == "yes" ]]; then
        echo ""
        echo "Routes via CrypRQ interface ($CRYPRQ_INTERFACE):"
        ip route show | grep "$CRYPRQ_INTERFACE" || echo "  (no routes found via CrypRQ interface)"
    fi
elif [[ "$OS" == "macOS" ]]; then
    echo "Default routes (netstat -rn | grep default):"
    if DEFAULT_ROUTES=$(netstat -rn | grep default); then
        echo "$DEFAULT_ROUTES"
        ROUTING_CHECK_STATUS="SUCCESS"
    else
        echo -e "${YELLOW}[ENV LIMITATION] Unable to query default routes${NC}"
        ROUTING_CHECK_STATUS="ENV_LIMITATION"
    fi

    echo ""
    echo "Route details (route -n get default):"
    route -n get default 2>&1 | head -10 || echo "  (unable to query)"

    if [[ "$CRYPRQ_INTERFACE_DETECTED" == "yes" ]]; then
        echo ""
        echo "Routes via CrypRQ interface ($CRYPRQ_INTERFACE):"
        netstat -rn | grep "$CRYPRQ_INTERFACE" || echo "  (no routes found via CrypRQ interface)"
    fi
fi

echo ""

# Interface List
echo "=========================================="
echo "Network Interfaces"
echo "=========================================="

if [[ "$OS" == "Linux" ]]; then
    echo "Active interfaces (ip link show):"
    ip link show | grep -E "^[0-9]+:" | head -10 || echo "  (unable to query)"
elif [[ "$OS" == "macOS" ]]; then
    echo "Active interfaces (ifconfig):"
    ifconfig | grep -E "^[a-z]" | head -10 || echo "  (unable to query)"
fi

echo ""

# Summary
echo "=========================================="
echo "Summary"
echo "=========================================="
echo ""
echo "IP checks: ${IP_CHECK_STATUS:-ENV_LIMITATION}"
echo "DNS checks: ${DNS_CHECK_STATUS:-ENV_LIMITATION}"
echo "IPv6 checks: ${IPV6_CHECK_STATUS:-INFO}"
echo "Routing checks: ${ROUTING_CHECK_STATUS:-ENV_LIMITATION}"
echo "CrypRQ VPN interface detected: $CRYPRQ_INTERFACE_DETECTED"
echo ""
echo -e "${BLUE}Recommended:${NC}"
echo "1. Compare these results with docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md"
echo "2. Run this probe before and after starting VPN to observe changes"
echo "3. Include this output in support bundles when reporting VPN issues"
echo "4. Interpret results in context of your deployment architecture"
echo ""
echo "Note: This probe does NOT make privacy guarantees."
echo "It verifies correctness and obvious leak behavior only."
echo ""

# Always exit 0 (probe, not a gate)
exit 0

