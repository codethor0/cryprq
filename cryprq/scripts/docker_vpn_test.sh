#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

IMG="${IMG:-cryprq-node}"
NET="${NET:-vpn-test}"
PORT="${PORT:-9999}"
PROTO="${PROTO:-quic-v1}"

echo "== build image =="
docker build -t "$IMG" -f Dockerfile .

echo "== cleanup old runs =="
docker rm -f vpn1 vpn2 >/dev/null 2>&1 || true
docker network rm "$NET" >/dev/null 2>&1 || true

# Clear out weird swarm leftovers that can cause "invalid cluster node"
docker swarm leave --force >/dev/null 2>&1 || true

echo "== create network =="
docker network create "$NET" >/dev/null

echo "== start vpn1 (listener) =="
docker run -d --name vpn1 --network "$NET" "$IMG" >/dev/null

# Wait for vpn1 to be running and log identity
echo "== wait for vpn1 identity =="
VPN1_ID=""
for i in {1..50}; do
  # Expect log line like: "Local identity: <PEER_ID>"
  line="$(docker logs vpn1 2>&1 | grep -E 'Local identity:' || true)"
  if [ -n "$line" ]; then
    # grab everything after the colon & space
    VPN1_ID="$(printf "%s" "$line" | sed -E 's/.*Local identity:\s*([[:alnum:]:._-]+).*/\1/' | tail -n1)"
    # If multiple lines appeared, take the last
  fi
  if [ -n "$VPN1_ID" ]; then break; fi
  sleep 0.2
done
if [ -z "$VPN1_ID" ]; then
  echo "Failed to read vpn1 identity from logs"; docker logs vpn1; exit 1
fi

VPN1_IP="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' vpn1)"
if [ -z "$VPN1_IP" ]; then
  echo "Could not get vpn1 IP on network $NET"; exit 1
fi

echo "vpn1 IP: $VPN1_IP"
echo "vpn1 ID: $VPN1_ID"

MADDR="/ip4/${VPN1_IP}/udp/${PORT}/${PROTO}/p2p/${VPN1_ID}"
echo "dial multiaddr: $MADDR"

echo "== start vpn2 (dialer) =="
docker run -d --name vpn2 --network "$NET" "$IMG" --peer "$MADDR" >/dev/null

sleep 2

# Check if vpn2 rejected the multiaddr format; if so, retry with plain ID
if docker logs vpn2 2>&1 | grep -q "Invalid peer address format"; then
  echo "vpn2 complained about multiaddr; retrying with plain peer id..."
  docker rm -f vpn2 >/dev/null
  docker run -d --name vpn2 --network "$NET" "$IMG" --peer "$VPN1_ID" >/dev/null
  sleep 2
fi

echo "--- vpn1 logs ---"
docker logs vpn1 --since 0s || true
echo "--- vpn2 logs ---"
docker logs vpn2 --since 0s || true

echo "Done."