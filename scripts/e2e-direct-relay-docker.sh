#!/usr/bin/env bash
# Copyright (c) 2026 Thor Thor
# SPDX-License-Identifier: MIT
#
# Relayed direct-file regression on ONE Docker host, using
# docker-compose.relay.yml: node-a and node-c on net-a, node-b on net-b, and a
# bounded relay on both. A direct path between the sides does not exist, so a
# delivery proves the relay route. It is not a WAN or NAT-traversal witness.
#
# Usage: bash scripts/e2e-direct-relay-docker.sh
#   CRYPRQ_DIRECT_IMAGE    image to run (default cryprq-web:latest)
#   CRYPRQ_E2E_KEEP_LOGS=1 keep the log directory after a pass

set -euo pipefail
cd "$(dirname "$0")/.."

PROJECT="cryprq-relay-e2e-$$"
COMPOSE=(docker compose -f docker-compose.relay.yml -p "$PROJECT")
RX="$PROJECT-rx"
RELAY_A_IP="${CRYPRQ_RELAY_A_IP:-172.31.231.10}"
RELAY_B_IP="${CRYPRQ_RELAY_B_IP:-172.31.232.10}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/cryprq-relay-docker-e2e.XXXXXX")"
CHECKS=0
PASSED=0

pass() { CHECKS=$((CHECKS + 1)); echo "PASS: $*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

relay_container() { "${COMPOSE[@]}" ps -q relay 2>/dev/null; }

cleanup() {
  if [ "$PASSED" != 1 ]; then
    docker logs "$RX" >"$WORK/receiver-last.log" 2>&1 || true
    local relay; relay="$(relay_container)"
    [ -z "$relay" ] || docker logs "$relay" >"$WORK/relay.log" 2>&1 || true
  fi
  docker rm -f "$RX" >/dev/null 2>&1 || true
  "${COMPOSE[@]}" down -v --remove-orphans >/dev/null 2>&1 || true
  if [ "$PASSED" = 1 ] && [ "${CRYPRQ_E2E_KEEP_LOGS:-}" != 1 ]; then
    rm -rf "$WORK"
  else
    echo "Relay Docker E2E evidence: $WORK"
  fi
}
trap cleanup EXIT

cli() {
  local node=$1; shift
  "${COMPOSE[@]}" run --rm -T "$node" \
    --identity /data/identity.key --trusted-peers /data/trusted_peers "$@"
}
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
networks_of() {
  docker inspect -f '{{range $name, $_ := .NetworkSettings.Networks}}{{$name}} {{end}}' "$1" | xargs -n1 | sort | xargs
}
posture() {
  local container=$1 what=$2 ports
  [ "$(docker inspect -f '{{.Config.User}}' "$container")" = node ] || fail "$what: not the non-root node user"
  [ "$(docker inspect -f '{{.HostConfig.ReadonlyRootfs}} {{.HostConfig.Privileged}}' "$container")" = "true false" ] \
    || fail "$what: writable root filesystem or privileged"
  docker inspect -f '{{json .HostConfig.CapDrop}}' "$container" | grep -q '"ALL"' || fail "$what: capabilities kept"
  docker inspect -f '{{json .HostConfig.SecurityOpt}}' "$container" | grep -q 'no-new-privileges' || fail "$what: no-new-privileges missing"
  ports="$(docker inspect -f '{{json .HostConfig.PortBindings}}|{{json .NetworkSettings.Ports}}|{{.HostConfig.PublishAllPorts}}' "$container")"
  case "$ports" in *HostPort*|*'|true') fail "$what publishes ports: $ports" ;; esac
}

# Relay arguments as seen from each side of the relay.
relay_args() {
  local side=$1 transport=$2 key=$3 ip
  if [ "$side" = a ]; then ip="$RELAY_A_IP"; else ip="$RELAY_B_IP"; fi
  if [ "$transport" = quic ]; then
    printf '%s\n' --relay "/ip4/$ip/udp/20450/quic-v1" --relay-key "$key"
  else
    printf '%s\n' --relay "/ip4/$ip/tcp/20450" --relay-key "$key"
  fi
}

# Starts one receiver on node-b, expecting `expected`. mode: quic|tcp (relay)
# or direct (a plain TCP listener, used only to prove isolation).
start_receiver() {
  local label=$1 mode=$2 expected=$3 waited=0 route=()
  in_node node-b "mkdir -m 700 /data/inbox-$label"
  if [ "$mode" = direct ]; then
    route=(--listen /ip4/0.0.0.0/tcp/20440)
  else
    while IFS= read -r arg; do route+=("$arg"); done < <(relay_args b "$mode" "$KEY_R")
  fi
  docker rm -f "$RX" >/dev/null 2>&1 || true
  "${COMPOSE[@]}" run -d --name "$RX" node-b \
    --identity /data/identity.key --trusted-peers /data/trusted_peers \
    direct-receive-file "${route[@]}" --peer-key "$expected" \
    --output-dir "/data/inbox-$label" >/dev/null
  until docker logs "$RX" 2>&1 | grep -q 'event=direct_file_listening'; do
    running || { docker logs "$RX" >"$WORK/$label-receiver.log" 2>&1; fail "$label: receiver exited before listening"; }
    [ "$waited" -lt 30 ] || fail "$label: receiver did not start listening"
    sleep 1; waited=$((waited + 1))
  done
}
inbox() { in_node node-b "ls -A /data/inbox-$1"; }

# Sends through the relay from node-a's (or node-c's) side.
send_relayed() {
  local node=$1 transport=$2 expected=$3 relay_key=$4 file=$5 label=$6 route=()
  while IFS= read -r arg; do route+=("$arg"); done < <(relay_args a "$transport" "$relay_key")
  cli "$node" direct-send-file "${route[@]}" --peer-key "$expected" \
    --file "/data/outbox/$file" 2>"$WORK/$label-sender.log"
}

# --- Identities, trust and relay ------------------------------------------
KEY_R="$(key_of relay init)"
KEY_A="$(key_of node-a init)"
KEY_B="$(key_of node-b init)"
KEY_C="$(key_of node-c init)"
[ "$(printf '%s\n' "$KEY_R" "$KEY_A" "$KEY_B" "$KEY_C" | sort -u | wc -l | tr -d ' ')" = 4 ] \
  || fail "identities are not distinct"
pass "relay and three node identities are distinct"

# A pins B and C (C only for the unexpected-target case); B pins only A;
# C pins B. Nobody pins the relay: it is never an application peer.
in_node node-a "printf '%s\n%s\n' '$KEY_B' '$KEY_C' > /data/trusted_peers"
in_node node-b "printf '%s\n' '$KEY_A' > /data/trusted_peers"
in_node node-c "printf '%s\n' '$KEY_B' > /data/trusted_peers"
in_node node-a "mkdir -p /data/outbox && head -c 300017 /dev/urandom > /data/outbox/payload.bin && : > /data/outbox/empty.bin && head -c 2097152 /dev/urandom > /data/outbox/large.bin"
in_node node-c "mkdir -p /data/outbox && head -c 4096 /dev/urandom > /data/outbox/payload.bin"

"${COMPOSE[@]}" up -d --no-build relay >/dev/null
RELAY="$(relay_container)"
waited=0
until [ "$(docker logs "$RELAY" 2>&1 | grep -c 'event=relay_advertised')" -ge 4 ]; do
  [ "$waited" -lt 30 ] || { docker logs "$RELAY" >"$WORK/relay.log" 2>&1; fail "relay did not start"; }
  sleep 1; waited=$((waited + 1))
done

# --- Topology and posture --------------------------------------------------
start_receiver isolation direct "$KEY_A"
posture "$RELAY" relay
posture "$RX" receiver
[ "$(networks_of "$RELAY")" = "${PROJECT}_net-a ${PROJECT}_net-b" ] || fail "relay is not on exactly both networks"
[ "$(networks_of "$RX")" = "${PROJECT}_net-b" ] || fail "node-b is not only on net-b"
for net in net-a net-b; do
  [ "$(docker network inspect -f '{{.Internal}}' "${PROJECT}_$net")" = true ] || fail "$net is not internal"
done
pass "non-root, read-only, no capabilities, no published ports; net-a and net-b internal, relay on both"

# --- No direct path exists ---------------------------------------------------
B_IP="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$RX")"
if cli node-a direct-send-file --peer "/ip4/$B_IP/tcp/20440" --peer-key "$KEY_B" \
    --file /data/outbox/payload.bin >/dev/null 2>"$WORK/isolation-sender.log"; then
  fail "isolation: node-a reached node-b directly"
fi
docker stop -t 10 "$RX" >/dev/null
[ -z "$(inbox isolation)" ] || fail "isolation: file published"
pass "isolation: node-a cannot reach node-b without the relay"

# --- Relayed delivery ----------------------------------------------------------
for spec in quic:payload.bin tcp:payload.bin tcp:empty.bin; do
  transport="${spec%%:*}"; file="${spec#*:}"
  label="relay-$transport-${file%.bin}"
  circuits_before="$(docker logs "$RELAY" 2>&1 | grep -c 'event=relay_circuit_opened' || true)"
  start_receiver "$label" "$transport" "$KEY_A"
  sent="$(send_relayed node-a "$transport" "$KEY_B" "$KEY_R" "$file" "$label")" || fail "$label: sender failed"
  wait_exit 30
  docker logs "$RX" >"$WORK/$label-receiver.log" 2>&1
  [ "$(exit_code)" = 0 ] || fail "$label: receiver exited $(exit_code)"
  want="$(in_node node-a "sha256sum /data/outbox/$file" | cut -d ' ' -f 1)"
  got="$(in_node node-b "sha256sum /data/inbox-$label/$file" | cut -d ' ' -f 1)"
  [ "$want" = "$got" ] || fail "$label: received hash differs"
  echo "$sent" | grep -q "^route=relay .*sha256=$want\$" || fail "$label: no relayed sender receipt"
  grep -q "^route=relay .*sha256=$want" "$WORK/$label-receiver.log" || fail "$label: no relayed receiver receipt"
  grep -q 'event=direct_file_authenticated role=initiator .*route=relay' "$WORK/$label-sender.log" || fail "$label: sender not authenticated over the relay"
  grep -q 'event=direct_file_authenticated role=responder .*route=relay' "$WORK/$label-receiver.log" || fail "$label: receiver not authenticated over the relay"
  ! grep -qF "$file" "$WORK/$label-receiver.log" || fail "$label: receiver log leaked the filename"
  [ "$(inbox "$label")" = "$file" ] || fail "$label: unexpected inbox contents"
  circuits_after="$(docker logs "$RELAY" 2>&1 | grep -c 'event=relay_circuit_opened' || true)"
  [ "$circuits_after" -gt "$circuits_before" ] || fail "$label: relay recorded no circuit"
  pass "$label: delivered through the relay, matching SHA-256 $want"
done

# --- Rejections: nothing is published --------------------------------------
start_receiver wrong-relay-key quic "$KEY_A"
if send_relayed node-a quic "$KEY_B" "$KEY_C" payload.bin wrong-relay-key >/dev/null; then
  fail "wrong-relay-key: sender accepted a relay with a different key"
fi
docker stop -t 10 "$RX" >/dev/null
[ -z "$(inbox wrong-relay-key)" ] || fail "wrong-relay-key: file published"
pass "wrong-relay-key: a relay that is not the pinned relay is refused"

start_receiver wrong-target quic "$KEY_A"
if send_relayed node-a quic "$KEY_C" "$KEY_R" payload.bin wrong-target >/dev/null; then
  fail "wrong-target: the relay delivered to an unexpected peer"
fi
docker stop -t 10 "$RX" >/dev/null
[ -z "$(inbox wrong-target)" ] || fail "wrong-target: file published"
pass "wrong-target: the relay does not connect an unreserved, unexpected peer"

start_receiver wrong-sender quic "$KEY_A"
if send_relayed node-c quic "$KEY_B" "$KEY_R" payload.bin wrong-sender >/dev/null; then
  fail "wrong-sender: an unexpected identity delivered through the relay"
fi
docker stop -t 10 "$RX" >/dev/null
[ -z "$(inbox wrong-sender)" ] || fail "wrong-sender: file published"
pass "wrong-sender: the receiver rejects an identity it does not expect"

start_receiver circuit-limit tcp "$KEY_A"
if send_relayed node-a tcp "$KEY_B" "$KEY_R" large.bin circuit-limit >/dev/null; then
  fail "circuit-limit: 2 MiB crossed a 1 MiB circuit"
fi
wait_exit 30
[ "$(exit_code)" != 0 ] || fail "circuit-limit: receiver reported success"
[ -z "$(inbox circuit-limit)" ] || fail "circuit-limit: partial or unverified file left in the inbox"
pass "circuit-limit: a circuit cut mid-transfer publishes nothing and leaves no partial file"

# node-a's identity, because the running relay holds the relay's fixed addresses.
if "${COMPOSE[@]}" run --rm -T -e CRYPRQ_TEST_MODE=1 node-a \
    --identity /data/identity.key relay-serve --listen /ip4/127.0.0.1/tcp/20451 \
    >/dev/null 2>"$WORK/relay-test-mode.log"; then
  fail "relay-test-mode: insecure mode was accepted"
fi
grep -q 'requires authenticated mode' "$WORK/relay-test-mode.log" || fail "relay-test-mode: wrong refusal"
pass "relay-test-mode: relay-serve refuses CRYPRQ_TEST_MODE"

PASSED=1
echo "RELAY DOCKER E2E: PASSED ($CHECKS checks); route=relay on a single Docker host; WAN TEST NOT VERIFIED"
