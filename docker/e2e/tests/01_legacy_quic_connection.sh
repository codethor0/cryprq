#!/bin/bash
# Copyright (c) 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Test: QUIC Handshake
# Verifies that QUIC connection is established between listener and dialer

set -euo pipefail

LISTENER_HOST="${LISTENER_CONTAINER:-cryprq-e2e-listener}"
DIALER_HOST="${DIALER_CONTAINER:-cryprq-e2e-dialer}"

echo "Test: legacy QUIC transport connection (test mode)"
echo "Verifying QUIC connection establishment..."

# Docker-available path: use strict container-based checks
if command -v docker >/dev/null 2>&1; then
    echo "[INFO] Using docker-based verification for QUIC handshake..."

    # Check if listener process is running
    if docker exec "$LISTENER_HOST" pgrep -f cryprq > /dev/null; then
        echo "[PASS] Listener process is running"
    else
        echo "[FAIL] Listener process not found"
        exit 1
    fi

    # Check if dialer process is running
    if docker exec "$DIALER_HOST" pgrep -f cryprq > /dev/null; then
        echo "[PASS] Dialer process is running"
    else
        echo "[FAIL] Dialer process not found"
        exit 1
    fi

    LISTENER_LOGS=$(docker logs "$LISTENER_HOST" 2>&1)
    DIALER_LOGS=$(docker logs "$DIALER_HOST" 2>&1)
    # Check listener logs for transport handshake completion
    if grep -qE "event=(handshake_complete|connection_established)" <<< "$LISTENER_LOGS"; then
        echo "[PASS] Handshake completed (from listener logs)"
    else
        echo "[FAIL] Handshake completion not found in listener logs"
        exit 1
    fi

    # Check dialer logs for handshake completion
    if grep -qE "event=(handshake_complete|connection_established)" <<< "$DIALER_LOGS"; then
        echo "[PASS] Handshake completed (from dialer logs)"
    else
        echo "[FAIL] Handshake completion not found in dialer logs"
        exit 1
    fi
else
    echo "[FAIL] UNRESOLVED: Docker CLI is required to verify both peers."
    exit 2
fi

echo "[PASS] Legacy QUIC transport connection events verified (this does not verify the CrypRQ hybrid handshake; see docs/E2E_COVERAGE_MAP.md)"
exit 0

