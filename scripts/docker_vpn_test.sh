#!/usr/bin/env bash
# CrypRQ Docker smoke test: build image, run listener & dialer containers, assert handshake logs.
set -euo pipefail

IMG="${IMG:-cryprq-node}"
NET="${NET:-cryprq-test}"
PORT="${PORT:-9999}"
TRANSPORT="${TRANSPORT:-tcp}"
case "$TRANSPORT" in
  tcp)
    LISTEN_ADDR="/ip4/0.0.0.0/tcp/${PORT}"
    build_peer_addr() {
      echo "/ip4/${1}/tcp/${PORT}"
    }
    ;;
  quic)
    LISTEN_ADDR="/ip4/0.0.0.0/udp/${PORT}/quic-v1"
    build_peer_addr() {
      echo "/ip4/${1}/udp/${PORT}/quic-v1/p2p/${2}"
    }
    ;;
  *)
    echo "Unsupported TRANSPORT value: $TRANSPORT" >&2
    exit 1
    ;;
esac

log() {
  printf '\n== %s ==\n' "$*"
}

wait_for_log() {
  local cname="$1" pattern="$2" timeout="$3" elapsed=0
  while (( elapsed < timeout )); do
    if docker logs "$cname" 2>&1 | grep -qE "$pattern"; then
      return 0
    fi
    sleep 0.5
    elapsed=$(( elapsed + 1 ))
  done
  return 1
}

cleanup() {
  docker rm -f vpn-listener vpn-dialer >/dev/null 2>&1 || true
  docker network rm "$NET" >/dev/null 2>&1 || true
}

log "build image"
docker build -t "$IMG" -f Dockerfile . >/dev/null

log "cleanup old runs"
cleanup
trap cleanup EXIT

docker swarm leave --force >/dev/null 2>&1 || true

log "create network"
docker network create "$NET" >/dev/null

log "start listener"
docker run -d --name vpn-listener --network "$NET" "$IMG" --listen "$LISTEN_ADDR" >/dev/null

log "wait for listener logs"
if ! wait_for_log vpn-listener "Listening on" 30; then
  echo "Listener did not report a listening address" >&2
  docker logs vpn-listener >&2 || true
  exit 1
fi

LISTEN_IP="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' vpn-listener)"
if [[ -z "$LISTEN_IP" ]]; then
  echo "Failed to resolve listener IP" >&2
  exit 1
fi

LISTEN_PEER_ID="$(docker logs vpn-listener 2>&1 | awk '/Local peer id:/ {print $4}' | tail -n1)"
if [[ -z "$LISTEN_PEER_ID" ]]; then
  echo "Failed to parse listener peer id" >&2
  docker logs vpn-listener >&2 || true
  exit 1
fi

PEER_ADDR=$(build_peer_addr "$LISTEN_IP" "$LISTEN_PEER_ID")
echo "Dial address: $PEER_ADDR"

log "start dialer"
docker run -d --name vpn-dialer --network "$NET" "$IMG" --peer "$PEER_ADDR" >/dev/null

log "wait for dialer connection"
if ! wait_for_log vpn-dialer "Connected to" 40; then
  echo "Dialer did not connect" >&2
  docker logs vpn-dialer >&2 || true
  exit 1
fi

echo "\n--- vpn-listener logs ---"
docker logs vpn-listener 2>&1 || true
echo "\n--- vpn-dialer logs ---"
docker logs vpn-dialer 2>&1 || true

echo "\nSmoke test succeeded."
