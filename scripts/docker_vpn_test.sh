#!/usr/bin/env bash
# Robust CrypRQ VPN test: builds image, launches containers, extracts full identity, dials peer, checks handshake
set -Eeuo pipefail

IMG="${IMG:-cryprq-node:latest}"
NET="vpn-test"
HANDSHAKE_PATTERN="Handshake|connected|Connected|handshake"

log(){ printf "\n== %s ==\n" "$*"; }

log "build image"
docker build -t "$IMG" . >/dev/null

log "cleanup old runs"
docker rm -f vpn1 vpn2 vpn1-node vpn2-node >/dev/null 2>&1 || true
docker network rm "$NET" >/dev/null 2>&1 || true

log "create network"
docker network create "$NET" >/dev/null

log "start vpn1 and vpn2 (idle)"
docker run -d --name vpn1 --network "$NET" --hostname vpn1 alpine:3.19 sleep 1d >/dev/null
docker run -d --name vpn2 --network "$NET" --hostname vpn2 alpine:3.19 sleep 1d >/dev/null

get_ip() {
	local name=$1
	local ip=""
	for _ in {1..20}; do
		ip="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$name" 2>/dev/null || true)"
		if [[ -n "$ip" ]]; then echo "$ip"; return 0; fi
		sleep 0.2
	done
	return 1
}

vpn1_ip="$(get_ip vpn1 || true)"
vpn2_ip="$(get_ip vpn2 || true)"
echo "vpn1 IP: ${vpn1_ip:-<none>}"
echo "vpn2 IP: ${vpn2_ip:-<none>}"

if [[ -z "${vpn1_ip:-}" || -z "${vpn2_ip:-}" ]]; then
	echo "ERROR: could not get container IPs on $NET. Ensure both containers are attached to '$NET'." >&2
	exit 1
fi

run_node_bg() {
	local base="$1"; shift
	docker run -d --name "${base}-node" --network "container:${base}" "$IMG" "$@" >/dev/null
}
}

logs_until() {
	local cname="$1" pattern="$2" timeout="$3"
	local start now
	start=$(date +%s)
	while true; do
		if docker logs "$cname" 2>&1 | grep -Eq "$pattern"; then return 0; fi
		now=$(date +%s)
		if (( now - start >= timeout )); then return 1; fi
		sleep 0.2
	done
}

extract_identity() {
	docker logs "$1" 2>&1 | grep -Eo '([1-9A-HJ-NP-Za-km-z]{46,64}|12D3K[A-Za-z0-9]{40,}|[0-9a-fA-F]{64})' | tail -n1 || true
}

log "launch cryprq on vpn1 (listener)"
run_node_bg vpn1 'command -v cryprq >/dev/null 2>&1 && cryprq --listen 0.0.0.0:0 || /usr/local/bin/cryprq --listen 0.0.0.0:0'

if ! logs_until vpn1-node 'Local identity|identity|Kyber pk|listening|Listening' 10; then
	echo "ERROR: vpn1-node did not start or print identity." >&2
	docker logs vpn1-node || true
	exit 1
fi

peer1="$(extract_identity vpn1-node)"
if [[ -z "$peer1" ]]; then
	echo "ERROR: could not parse vpn1 identity from logs. Full logs:" >&2
	docker logs vpn1-node >&2 || true
	exit 1
fi
echo "vpn1 identity: ${peer1:0:12}…"

log "launch cryprq on vpn2 (dialer -> vpn1)"
run_node_bg vpn2 "command -v cryprq >/dev/null 2>&1 && cryprq --listen 0.0.0.0:0 --peer '$peer1' || /usr/local/bin/cryprq --listen 0.0.0.0:0 --peer '$peer1'"

sleep 2

log "vpn1 logs"
docker logs --tail 200 vpn1-node 2>&1 || true
log "vpn2 logs"
docker logs --tail 200 vpn2-node 2>&1 || true

log "result"
if docker logs vpn1-node 2>&1 | grep -Eq "$HANDSHAKE_PATTERN"; then
	echo "vpn1: handshake seen"
else
	echo "vpn1: no handshake marker yet"
fi
if docker logs vpn2-node 2>&1 | grep -Eq "$HANDSHAKE_PATTERN"; then
	echo "vpn2: handshake seen"
else
	echo "vpn2: no handshake marker yet"
fi

echo -e "\nCleanup when done:\n  docker rm -f vpn1-node vpn2-node vpn1 vpn2 >/dev/null 2>&1 || true\n"
#!/bin/sh
set -e

# Build the Docker image for CrypRQ VPN node
#!/usr/bin/env bash
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
docker swarm leave --force >/dev/null 2>&1 || true

echo "== create network =="
docker network create "$NET" >/dev/null

echo "== start vpn1 (listener) =="
docker run -d --name vpn1 --network "$NET" "$IMG" >/dev/null

echo "== wait for vpn1 identity =="
VPN1_ID=""
for i in {1..50}; do
	line="$(docker logs vpn1 2>&1 | grep -E 'Local identity:' || true)"
	if [ -n "$line" ]; then
		VPN1_ID="$(printf "%s" "$line" | sed -E 's/.*Local identity:\s*([[:alnum:]:._-]+).*/\1/' | tail -n1)"
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
#!/usr/bin/env bash
set -Eeuo pipefail
echo "== start vpn2 (dialer) =="
IMAGE="cryprq-node"
NET="vpn-test"
VPN1="vpn1"
VPN2="vpn2"


sleep 2

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
