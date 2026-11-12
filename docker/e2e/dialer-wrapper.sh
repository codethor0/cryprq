#!/bin/bash
# Copyright (c) 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Wrapper script to resolve DNS and construct multiaddr for libp2p QUIC
# libp2p QUIC transport doesn't support /dns4/ multiaddrs, so we resolve to IP first

set -euo pipefail

PEER_HOST="${PEER_HOST:-listener}"
PEER_PORT="${PEER_PORT:-9999}"

# Resolve hostname to IP address
# Try getent first (faster), fall back to nslookup
if command -v getent >/dev/null 2>&1; then
    PEER_IP=$(getent hosts "$PEER_HOST" | awk '{print $1}' | head -1)
elif command -v nslookup >/dev/null 2>&1; then
    PEER_IP=$(nslookup "$PEER_HOST" 2>/dev/null | grep -A1 "Name:" | grep "Address:" | awk '{print $2}' | head -1)
else
    echo "Error: Neither getent nor nslookup available for DNS resolution" >&2
    exit 1
fi

if [ -z "$PEER_IP" ]; then
    echo "Error: Failed to resolve $PEER_HOST to IP address" >&2
    exit 1
fi

# Construct multiaddr with resolved IP
PEER_ADDR="/ip4/${PEER_IP}/udp/${PEER_PORT}/quic-v1"

echo "Resolved $PEER_HOST to $PEER_IP"
echo "Dialing peer $PEER_ADDR"
echo "DEBUG: Multiaddr constructed: $PEER_ADDR"

# Execute cryprq with resolved multiaddr
exec /usr/local/bin/cryprq --peer "$PEER_ADDR" "$@"
