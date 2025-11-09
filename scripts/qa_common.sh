#!/usr/bin/env bash
# shellcheck shell=bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

LISTEN_PORT=${LISTEN_PORT:-9999}
BASE_NET=${BASE_NET:-cryprq-qa}
LISTENER_NAME=${LISTENER_NAME:-cryprq-listener}
DIAL_PREFIX=${DIAL_PREFIX:-cryprq-dialer}
LOCAL_IMAGE=${LOCAL_IMAGE:-cryprq-local:qa}
ROTATE_SOAK=${ROTATE_SOAK:-300}
PARALLEL_COUNT=${PARALLEL_COUNT:-5}
ROTATE_60_DURATION=${ROTATE_60_DURATION:-240}
SOAK_DURATION=${SOAK_DURATION:-2700}
METRIC_INTERVAL=${METRIC_INTERVAL:-10}
RECONNECT_DELAY=${RECONNECT_DELAY:-5}

log()        { printf '\n[%s] %s\n' "$(date +'%F %T')" "$*"; }
require_cmd(){ command -v "$1" >/dev/null || { echo "Missing command: $1"; exit 1; }; }

cleanup() {
  log "Cleaning up containers/networks..."
  docker rm -f "${LISTENER_NAME}" >/dev/null 2>&1 || true
  local dialers
  dialers=$(docker ps -a --filter "name=${DIAL_PREFIX}" -q) || dialers=""
  if [[ -n "${dialers}" ]]; then
    docker rm -f ${dialers} >/dev/null 2>&1 || true
  fi
  docker network rm "${BASE_NET}" >/dev/null 2>&1 || true
}

create_network() {
  docker network inspect "${BASE_NET}" >/dev/null 2>&1 || docker network create "${BASE_NET}" >/dev/null
}

wait_for_log() {
  local cname="$1" pattern="$2" timeout_s="$3" elapsed=0
  while (( elapsed < timeout_s )); do
    if docker logs "$cname" 2>&1 | grep -qE "$pattern"; then
      return 0
    fi
    sleep 1
    elapsed=$(( elapsed + 1 ))
  done
  return 1
}

start_listener() {
  local rotate="$1" metrics_addr="${2:-0.0.0.0:9464}"
  docker rm -f "${LISTENER_NAME}" >/dev/null 2>&1 || true
  docker run -d --name "${LISTENER_NAME}" --network "${BASE_NET}" \
    -p "${LISTEN_PORT}:${LISTEN_PORT}/udp" \
    -e CRYPRQ_ROTATE_SECS="${rotate}" \
    -e CRYPRQ_METRICS_ADDR="${metrics_addr}" \
    -e RUST_LOG=info \
    "${LOCAL_IMAGE}" \
    --listen "/ip4/0.0.0.0/udp/${LISTEN_PORT}/quic-v1" >/dev/null
  wait_for_log "${LISTENER_NAME}" "Listening on" 30 || { echo "Listener failed to start"; exit 1; }
  docker logs "${LISTENER_NAME}" | tail -n 10
}

start_dialer() {
  local name="$1" target="$2" reconnect="${3:-false}" delay="${4:-${RECONNECT_DELAY}}"
  docker rm -f "$name" >/dev/null 2>&1 || true
  if [[ "${reconnect}" == "true" ]]; then
    docker run -d --name "$name" --network "${BASE_NET}" \
      -e TARGET="${target}" \
      -e RECONNECT_DELAY="${delay}" \
      --entrypoint /bin/sh \
      "${LOCAL_IMAGE}" \
      -c 'while true; do cryprq --peer "$TARGET"; sleep "$RECONNECT_DELAY"; done' >/dev/null
  else
    docker run -d --name "$name" --network "${BASE_NET}" "${LOCAL_IMAGE}" --peer "${target}" >/dev/null
  fi
}

baseline_handshake() {
  : "${QA_DIR:?QA_DIR must be set before running scenarios}"
  log "QA #1 baseline handshake"
  cleanup
  create_network
  start_listener 300 "0.0.0.0:9464"
  local peer_id listener_ip
  peer_id="$(docker logs "${LISTENER_NAME}" | awk '/Local peer id:/ {print $4; exit}')"
  listener_ip="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "${LISTENER_NAME}")"
  [[ -n "${peer_id}" && -n "${listener_ip}" ]] || { echo "Failed to obtain listener details"; exit 1; }
  local target="/ip4/${listener_ip}/udp/${LISTEN_PORT}/quic-v1/p2p/${peer_id}"
  start_dialer "${DIAL_PREFIX}-baseline" "${target}" false
  wait_for_log "${DIAL_PREFIX}-baseline" "Connected to" 30 || { echo "Baseline handshake failed"; exit 1; }
  docker logs "${DIAL_PREFIX}-baseline" > "${QA_DIR}/baseline.log"
  log "Baseline handshake PASS"
}

rotation_60s() {
  : "${QA_DIR:?QA_DIR must be set before running scenarios}"
  log "QA #2 rotation 60s for ${ROTATE_60_DURATION}s"
  cleanup
  create_network
  start_listener 60 "0.0.0.0:9464"
  local peer_id listener_ip
  peer_id="$(docker logs "${LISTENER_NAME}" | awk '/Local peer id:/ {print $4; exit}')"
  listener_ip="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "${LISTENER_NAME}")"
  [[ -n "${peer_id}" && -n "${listener_ip}" ]] || { echo "Failed to obtain listener details"; exit 1; }
  local target="/ip4/${listener_ip}/udp/${LISTEN_PORT}/quic-v1/p2p/${peer_id}"
  start_dialer "${DIAL_PREFIX}-rot" "${target}" true
  set +e
  timeout "${ROTATE_60_DURATION}" bash -c 'while true; do sleep 5; done'
  status=$?
  set -e
  if [[ ${status} -ne 0 && ${status} -ne 124 ]]; then
    echo "Rotation wait loop failed (status ${status})" && exit 1
  fi
  log "Capturing rotation logs to ${QA_DIR}/rotation60.log"
  docker logs "${DIAL_PREFIX}-rot" > "${QA_DIR}/rotation60.log"
  if ! grep -q "Connected to" "${QA_DIR}/rotation60.log"; then
    echo "Rotation test missing success logs" && exit 1
  fi
  log "Rotation 60s PASS"
}

parallel_dialers() {
  : "${QA_DIR:?QA_DIR must be set before running scenarios}"
  log "QA #3 parallel dialers (${PARALLEL_COUNT})"
  cleanup
  create_network
  start_listener 300 "0.0.0.0:9464"
  local peer_id listener_ip
  peer_id="$(docker logs "${LISTENER_NAME}" | awk '/Local peer id:/ {print $4; exit}')"
  listener_ip="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "${LISTENER_NAME}")"
  [[ -n "${peer_id}" && -n "${listener_ip}" ]] || { echo "Failed to obtain listener details"; exit 1; }
  local target="/ip4/${listener_ip}/udp/${LISTEN_PORT}/quic-v1/p2p/${peer_id}"
  for i in $(seq 1 "${PARALLEL_COUNT}"); do
    start_dialer "${DIAL_PREFIX}-${i}" "${target}" false
  done
  sleep 20
  for i in $(seq 1 "${PARALLEL_COUNT}"); do
    docker logs "${DIAL_PREFIX}-${i}" > "${QA_DIR}/parallel-${i}.log"
    if ! grep -q "Connected to" "${QA_DIR}/parallel-${i}.log"; then
      echo "Dialer ${i} failed" && exit 1
    fi
  done
  log "Parallel dialers PASS"
}

soak_test() {
  : "${QA_DIR:?QA_DIR must be set before running scenarios}"
  local dialers="${SOAK_DIALERS:-3}"
  log "QA #4 soak ${SOAK_DURATION}s with ${dialers} dialers (rotate ${ROTATE_SOAK})"
  cleanup
  create_network
  start_listener "${ROTATE_SOAK}" "0.0.0.0:9464"
  local peer_id listener_ip
  peer_id="$(docker logs "${LISTENER_NAME}" | awk '/Local peer id:/ {print $4; exit}')"
  listener_ip="$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "${LISTENER_NAME}")"
  [[ -n "${peer_id}" && -n "${listener_ip}" ]] || { echo "Failed to obtain listener details"; exit 1; }
  local target="/ip4/${listener_ip}/udp/${LISTEN_PORT}/quic-v1/p2p/${peer_id}"
  log "Soak target multiaddr: ${target}"
  for i in $(seq 1 "${dialers}"); do
    start_dialer "${DIAL_PREFIX}-soak-${i}" "${target}" true
  done
  docker stats --no-stream "${LISTENER_NAME}" > "${QA_DIR}/soak_start_stats.txt"
  set +e
  timeout "${SOAK_DURATION}" bash -c 'while true; do sleep 30; done'
  status=$?
  set -e
  if [[ ${status} -ne 0 && ${status} -ne 124 ]]; then
    echo "Soak wait loop failed (status ${status})" && exit 1
  fi
  log "Capturing soak logs to ${QA_DIR}/soak-dialer-*.log"
  for i in $(seq 1 "${dialers}"); do
    docker logs "${DIAL_PREFIX}-soak-${i}" > "${QA_DIR}/soak-dialer-${i}.log"
    if ! grep -q "Connected to" "${QA_DIR}/soak-dialer-${i}.log"; then
      echo "Soak dialer ${i} missing success log"; exit 1
    fi
  done
  docker stats --no-stream "${LISTENER_NAME}" > "${QA_DIR}/soak_end_stats.txt"
  docker logs "${LISTENER_NAME}" > "${QA_DIR}/soak-listener.log"
  log "Soak PASS"
}
