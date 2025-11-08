#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -Eeuo pipefail

# --- Tunables ---------------------------------------------------------------
IMG="cryprq-node"           # the image name built by your Dockerfile
NET="vpn-test"              # docker network name
P1_PORT=4011
P2_PORT=4012

# How long to wait for logs to show identity/handshake
WAIT_SECS=20
# ---------------------------------------------------------------------------

# Helper: get IP on a network with a hyphen in its name
get_ip () {
  local cname="$1"
  docker inspect -f "{{range \$n,\$v := .NetworkSettings.Networks}}{{if eq \$n \"$NET\"}}{{\$v.IPAddress}}{{end}}{{end}}" "$cname"
}

# Helper: wait for a log line starting with a prefix, return last match
wait_for_log () {
  local cname="$1" prefix="$2" secs="$3"
  local out=""
  for ((i=0;i<secs*10;i++)); do
    out="$(docker logs "$cname" 2>&1 | awk -v pfx="$prefix" 'index($0, pfx)==1{print substr($0, length(pfx)+1)}' | tail -n1 | tr -d '\r')"
    if [[ -n "$out" ]]; then
      echo "$out"
      return 0
    fi
    sleep 0.1
  done
  return 1
}

# Helper: wait until any of the patterns appears in logs
wait_for_any_log_pattern () {
  local cname="$1" secs="$2"; shift 2
  local patterns=("$@")
  for ((i=0;i<secs*10;i++)); do
    local logs
    logs="$(docker logs "$cname" 2>&1 | tail -n 500)"
    for pat in "${patterns[@]}"; do
      if echo "$logs" | grep -Eiq "$pat"; then
        echo "$pat"
        return 0
      fi
    done
    sleep 0.1
  done
  return 1
}

echo
echo "== build image =="
docker build -t "$IMG" .

echo
echo "== cleanup old runs =="
docker rm -f vpn1 vpn2 >/dev/null 2>&1 || true
docker network rm "$NET" >/dev/null 2>&1 || true

echo
echo "== create network =="
docker network create "$NET" >/dev/null

echo
echo "== start vpn1 (listener) =="
docker run -d --name vpn1 --network "$NET" "$IMG" >/dev/null
sleep 2  # Give container time to start and get IP
VPN1_IP="$(get_ip vpn1)"
if [[ -z "${VPN1_IP:-}" ]]; then
  echo "ERROR: Could not get vpn1 IP on network $NET"; exit 1
fi
echo "vpn1 IP: $VPN1_IP"

echo
echo "== wait for vpn1 identity =="
ID1="$(wait_for_log vpn1 'Local identity: ' "$WAIT_SECS" || true)"
if [[ -z "${ID1:-}" ]]; then
  echo "ERROR: vpn1 did not print a 'Local identity:' line within ${WAIT_SECS}s"
  docker logs vpn1 | tail -n 100
  exit 1
fi
if [[ "$ID1" == *"…"* ]]; then
  echo "ERROR: vpn1 identity appears truncated ('$ID1')."
  exit 1
fi
echo "vpn1 PeerId: $ID1"

ADDR1="/ip4/$VPN1_IP/tcp/$P1_PORT/p2p/$ID1"

echo
echo "== start vpn2 (dialer) =="
docker run -d --name vpn2 --network "$NET" "$IMG" --peer "$ADDR1" >/dev/null
VPN2_IP="$(get_ip vpn2)"
echo "vpn2 IP: $VPN2_IP"

echo
echo "== wait for handshake marker =="
HMARK="$(wait_for_any_log_pattern vpn2 "$WAIT_SECS" \
          "handshake (complete|done|established)" \
          "connected to peer" \
          "Established .* session" \
          "Kyber .* shared secret" \
          "Connection established" || true)"

if [[ -n "${HMARK:-}" ]]; then
  echo "Handshake marker seen on vpn2 logs: '$HMARK'"
  echo
  echo "== summary =="
  echo "vpn1: PeerId=$ID1  ip=$VPN1_IP"
  echo "vpn2: dialed $ADDR1"
  exit 0
else
  echo "No handshake markers yet. Last 120 lines from both containers:"
  echo "---- vpn1 ----"; docker logs vpn1 --tail 120 || true
  echo "---- vpn2 ----"; docker logs vpn2 --tail 120 || true
  exit 2
fi
