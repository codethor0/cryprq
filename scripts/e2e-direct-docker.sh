#!/usr/bin/env bash
# Copyright (c) 2026 Thor Thor
# SPDX-License-Identifier: MIT
#
# Two-container direct-file regression on ONE Docker host, using
# docker-compose.direct.yml. It proves packaging, isolation and the
# authenticated QUIC/TCP path between containers. It is not a WAN witness.
#
# Usage: bash scripts/e2e-direct-docker.sh
#   CRYPRQ_DIRECT_IMAGE  image to run (default cryprq-web:latest, built from
#                        Dockerfile.web if missing)
#   CRYPRQ_E2E_KEEP_LOGS=1 keep the log directory after a pass

set -euo pipefail
cd "$(dirname "$0")/.."

PROJECT="cryprq-direct-e2e-$$"
COMPOSE=(docker compose -f docker-compose.direct.yml -p "$PROJECT")
RX="$PROJECT-rx"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/cryprq-direct-docker-e2e.XXXXXX")"
CHECKS=0
PASSED=0

pass() { CHECKS=$((CHECKS + 1)); echo "PASS: $*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

cleanup() {
  if [ "$PASSED" != 1 ] && docker inspect "$RX" >/dev/null 2>&1; then
    docker logs "$RX" >"$WORK/receiver-last.log" 2>&1 || true
  fi
  docker rm -f "$RX" >/dev/null 2>&1 || true
  "${COMPOSE[@]}" down -v --remove-orphans >/dev/null 2>&1 || true
  if [ "$PASSED" = 1 ] && [ "${CRYPRQ_E2E_KEEP_LOGS:-}" != 1 ]; then
    rm -rf "$WORK"
  else
    echo "Direct Docker E2E evidence: $WORK"
  fi
}
trap cleanup EXIT

# The CLI with this node's private state; extra `docker run` flags go first.
cli() {
  local node=$1; shift
  "${COMPOSE[@]}" run --rm -T "$node" \
    --identity /data/identity.key --trusted-peers /data/trusted_peers "$@"
}
# A shell inside a node's container, for setup and inspection only.
in_node() {
  local node=$1; shift
  "${COMPOSE[@]}" run --rm -T --entrypoint /bin/sh "$node" -c "umask 077; $*"
}
key_of() {
  local key
  key="$(cli "$1" identity "$2" 2>>"$WORK/$1-identity.err" | tr -d '\r' | tail -n 1)"
  [[ "$key" =~ ^ed25519:[0-9a-f]{64}$ ]] || fail "$1: identity $2 printed no public key"
  printf '%s' "$key"
}
running() { [ "$(docker inspect -f '{{.State.Running}}' "$RX" 2>/dev/null)" = true ]; }
wait_exit() {
  local limit=$1 waited=0
  while running; do
    [ "$waited" -lt "$limit" ] || fail "receiver still running after ${limit}s"
    sleep 1; waited=$((waited + 1))
  done
}
exit_code() { docker inspect -f '{{.State.ExitCode}}' "$RX"; }

# Starts one receiver on node-b; sets ADDR to the address node-a must dial.
start_receiver() {
  local label=$1 transport=$2 expected=$3 listen suffix waited=0 ip
  in_node node-b "mkdir -m 700 /data/inbox-$label"
  if [ "$transport" = quic ]; then suffix="udp/20440/quic-v1"; else suffix="tcp/20440"; fi
  listen="/ip4/0.0.0.0/$suffix"
  docker rm -f "$RX" >/dev/null 2>&1 || true
  "${COMPOSE[@]}" run -d --name "$RX" node-b \
    --identity /data/identity.key --trusted-peers /data/trusted_peers \
    direct-receive-file --listen "$listen" --peer-key "$expected" \
    --output-dir "/data/inbox-$label" >/dev/null
  until docker logs "$RX" 2>&1 | grep -q 'event=direct_file_listening'; do
    running || { docker logs "$RX" >"$WORK/$label-receiver.log" 2>&1; fail "$label: receiver exited before listening"; }
    [ "$waited" -lt 30 ] || fail "$label: receiver did not start listening"
    sleep 1; waited=$((waited + 1))
  done
  ip="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$RX")"
  [[ "$ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "$label: no container IPv4 address"
  ADDR="/ip4/$ip/$suffix"
}
inbox() { in_node node-b "ls -A /data/inbox-$1"; }

# --- Identities and trust -------------------------------------------------
KEY_A="$(key_of node-a init)"
KEY_B="$(key_of node-b init)"
KEY_C="$(key_of node-c init)"
[ "$KEY_A" != "$KEY_B" ] && [ "$KEY_B" != "$KEY_C" ] && [ "$KEY_A" != "$KEY_C" ] \
  || fail "identities are not distinct"
[ "$(key_of node-a show)" = "$KEY_A" ] || fail "node-a identity did not persist in its volume"
[ "$(key_of node-b show)" = "$KEY_B" ] || fail "node-b identity did not persist in its volume"
pass "three distinct identities persist across fresh containers"

# A pins B and C (C only for the wrong-receiver case); B pins only A; C pins B.
in_node node-a "printf '%s\n%s\n' '$KEY_B' '$KEY_C' > /data/trusted_peers"
in_node node-b "printf '%s\n' '$KEY_A' > /data/trusted_peers"
in_node node-c "printf '%s\n' '$KEY_B' > /data/trusted_peers"
in_node node-a "mkdir -p /data/outbox && head -c 300017 /dev/urandom > /data/outbox/payload.bin && : > /data/outbox/empty.bin"
in_node node-c "mkdir -p /data/outbox && head -c 4096 /dev/urandom > /data/outbox/payload.bin"

# --- Container posture (checked on a live receiver) ------------------------
start_receiver posture quic "$KEY_A"
[ "$(docker inspect -f '{{.Config.User}}' "$RX")" = node ] || fail "receiver is not running as the non-root node user"
[ "$(docker inspect -f '{{.HostConfig.ReadonlyRootfs}} {{.HostConfig.Privileged}}' "$RX")" = "true false" ] \
  || fail "receiver root filesystem is writable or the container is privileged"
docker inspect -f '{{json .HostConfig.CapDrop}}' "$RX" | grep -q '"ALL"' || fail "capabilities were not dropped"
docker inspect -f '{{json .HostConfig.SecurityOpt}}' "$RX" | grep -q 'no-new-privileges' || fail "no-new-privileges is not set"
# The image EXPOSEs the portal port as metadata ("8787/tcp": null); only a host
# binding would publish anything, so no HostPort may appear anywhere.
ports="$(docker inspect -f '{{json .HostConfig.PortBindings}}|{{json .NetworkSettings.Ports}}|{{.HostConfig.PublishAllPorts}}' "$RX")"
case "$ports" in *HostPort*|*'|true') fail "receiver publishes ports: $ports" ;; esac
[ "$(docker network inspect -f '{{.Internal}}' "${PROJECT}_direct")" = true ] || fail "network is not internal"
docker stop -t 10 "$RX" >/dev/null
pass "non-root, read-only, no capabilities, no published ports, internal network"

# --- Authenticated delivery over QUIC and TCP ------------------------------
for transport in quic tcp; do
  for file in payload.bin empty.bin; do
    label="$transport-${file%.bin}"
    start_receiver "$label" "$transport" "$KEY_A"
    sent="$(cli node-a direct-send-file --peer "$ADDR" --peer-key "$KEY_B" \
      --file "/data/outbox/$file" 2>"$WORK/$label-sender.log")" || fail "$label: sender failed"
    wait_exit 30
    docker logs "$RX" >"$WORK/$label-receiver.log" 2>&1
    [ "$(exit_code)" = 0 ] || fail "$label: receiver exited $(exit_code)"
    want="$(in_node node-a "sha256sum /data/outbox/$file" | cut -d ' ' -f 1)"
    got="$(in_node node-b "sha256sum /data/inbox-$label/$file" | cut -d ' ' -f 1)"
    [ "$want" = "$got" ] || fail "$label: received hash differs"
    echo "$sent" | grep -q "^route=direct .*sha256=$want\$" || fail "$label: no authenticated sender receipt"
    grep -q "sha256=$want" "$WORK/$label-receiver.log" || fail "$label: no receiver receipt"
    grep -q 'event=direct_file_authenticated role=initiator' "$WORK/$label-sender.log" || fail "$label: sender not authenticated"
    grep -q 'event=direct_file_authenticated role=responder' "$WORK/$label-receiver.log" || fail "$label: receiver not authenticated"
    ! grep -qF "$file" "$WORK/$label-receiver.log" || fail "$label: receiver log leaked the filename"
    [ "$(inbox "$label")" = "$file" ] || fail "$label: unexpected inbox contents"
    pass "$label: container-to-container delivery, matching SHA-256 $want"
  done
done

# --- Rejections: nothing is published --------------------------------------
start_receiver wrong-sender quic "$KEY_A"
if cli node-c direct-send-file --peer "$ADDR" --peer-key "$KEY_B" \
    --file /data/outbox/payload.bin >/dev/null 2>"$WORK/wrong-sender.log"; then
  fail "wrong-sender: an unpinned identity delivered a file"
fi
docker stop -t 10 "$RX" >/dev/null
[ -z "$(inbox wrong-sender)" ] || fail "wrong-sender: file published"
pass "wrong-sender: identity the receiver does not expect is rejected"

start_receiver wrong-receiver quic "$KEY_A"
if cli node-a direct-send-file --peer "$ADDR" --peer-key "$KEY_C" \
    --file /data/outbox/payload.bin >/dev/null 2>"$WORK/wrong-receiver.log"; then
  fail "wrong-receiver: sender accepted a listener with a different key"
fi
docker stop -t 10 "$RX" >/dev/null
[ -z "$(inbox wrong-receiver)" ] || fail "wrong-receiver: file published"
pass "wrong-receiver: sender refuses a listener that is not the pinned peer"

if "${COMPOSE[@]}" run --rm -T -e CRYPRQ_TEST_MODE=1 node-a \
    --identity /data/identity.key --trusted-peers /data/trusted_peers \
    direct-send-file --peer /ip4/127.0.0.1/tcp/9 --peer-key "$KEY_B" \
    --file /data/outbox/payload.bin >/dev/null 2>"$WORK/test-mode.log"; then
  fail "test-mode: insecure mode was accepted"
fi
grep -q 'requires authenticated mode' "$WORK/test-mode.log" || fail "test-mode: wrong refusal"
pass "test-mode: CRYPRQ_TEST_MODE refused inside the container"

# --- Clean shutdown ---------------------------------------------------------
start_receiver cancel tcp "$KEY_A"
docker stop -t 10 "$RX" >/dev/null
code="$(exit_code)"
[ "$code" != 137 ] || fail "cancel: receiver ignored SIGTERM and was killed"
[ "$code" != 0 ] || fail "cancel: a cancelled receiver reported success"
[ -z "$(inbox cancel)" ] || fail "cancel: files left in the inbox"
pass "cancel: SIGTERM stops the receiver promptly with nothing published"

PASSED=1
echo "DIRECT DOCKER E2E: PASSED ($CHECKS checks); route=single-docker-host; WAN TEST NOT VERIFIED"
