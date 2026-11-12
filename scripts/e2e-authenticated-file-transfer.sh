#!/bin/bash
# © 2025 Thor Thor
# SPDX-License-Identifier: MIT
#
# Two-peer end-to-end test of the authenticated tunnel.
#
# Starts two separate cryprq processes on loopback, each with its own identity
# and its own pinned-peer file, and checks that:
#   1. two mutually pinned peers complete the handshake and a file arrives
#      byte-identical on the receiver;
#   2. a sender the receiver has not pinned gets no tunnel and delivers nothing;
#   3. a sender expecting another identity at the receiver's address gets no
#      tunnel;
#   4. a node without an identity, or without pinned peers, refuses to start;
#   5. the fixed test keys are refused unless test mode is switched on
#      explicitly, and then warn that they are insecure.
#
# Usage: scripts/e2e-authenticated-file-transfer.sh [path/to/cryprq]

set -u

BINARY="${1:-./target/debug/cryprq}"
if [ ! -x "$BINARY" ]; then
    echo "ERROR: $BINARY not found. Build it first: cargo build -p cryprq" >&2
    exit 2
fi
BINARY="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/cryprq-e2e.XXXXXX")"
chmod 700 "$WORK"
RECEIVER_PID=""
FAILURES=0
# Never inherit test mode from the caller's environment.
unset CRYPRQ_TEST_MODE

cleanup() {
    [ -n "$RECEIVER_PID" ] && kill "$RECEIVER_PID" 2>/dev/null
    wait 2>/dev/null
    if [ "$FAILURES" -eq 0 ]; then
        rm -rf "$WORK"
    else
        echo "Logs kept in $WORK"
    fi
}
trap cleanup EXIT

pass() { echo "PASS: $1"; }
fail() {
    echo "FAIL: $1"
    FAILURES=$((FAILURES + 1))
}

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

free_port() {
    python3 -c 'import socket; s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}

# node <name> <args...>: run cryprq as the named peer.
node() {
    local name="$1"
    shift
    "$BINARY" --identity "$WORK/$name/identity.key" \
        --trusted-peers "$WORK/$name/trusted_peers" "$@"
}

start_receiver() {
    local port="$1" log="$2"
    rm -rf "$WORK/received"
    mkdir -p "$WORK/received"
    # Started directly, not through the node() function, so that RECEIVER_PID
    # is the PID of cryprq itself and stop_receiver really stops it.
    "$BINARY" --identity "$WORK/bob/identity.key" --trusted-peers "$WORK/bob/trusted_peers" \
        receive-file --listen "/ip4/127.0.0.1/udp/$port" \
        --output-dir "$WORK/received" >"$log" 2>&1 &
    RECEIVER_PID=$!
    for _ in $(seq 1 100); do
        grep -q "event=listening" "$log" 2>/dev/null && return 0
        kill -0 "$RECEIVER_PID" 2>/dev/null || return 1
        sleep 0.1
    done
    return 1
}

stop_receiver() {
    [ -n "$RECEIVER_PID" ] && kill "$RECEIVER_PID" 2>/dev/null
    wait "$RECEIVER_PID" 2>/dev/null
    RECEIVER_PID=""
}

wait_for_file() {
    local path="$1" expected_size="$2"
    for _ in $(seq 1 100); do
        if [ -f "$path" ] && [ "$(wc -c <"$path" | tr -d ' ')" = "$expected_size" ]; then
            return 0
        fi
        sleep 0.1
    done
    return 1
}

echo "== Provisioning three identities in $WORK"
for name in alice bob mallory; do
    mkdir -m 700 "$WORK/$name"
    if ! node "$name" identity init >"$WORK/$name/public_key" 2>"$WORK/$name/init.log"; then
        echo "ERROR: identity init failed for $name" >&2
        cat "$WORK/$name/init.log" >&2
        FAILURES=1
        exit 1
    fi
done
ALICE_KEY="$(cat "$WORK/alice/public_key")"
BOB_KEY="$(cat "$WORK/bob/public_key")"
MALLORY_KEY="$(cat "$WORK/mallory/public_key")"

# Alice and Bob pin each other. Mallory pins Bob, but Bob does not pin Mallory.
echo "$BOB_KEY bob" >"$WORK/alice/trusted_peers"
echo "$ALICE_KEY alice" >"$WORK/bob/trusted_peers"
echo "$BOB_KEY bob" >"$WORK/mallory/trusted_peers"

if [ "$(node alice identity show)" = "$ALICE_KEY" ] && [ "$ALICE_KEY" != "$BOB_KEY" ]; then
    pass "identities persist and differ per node"
else
    fail "identity show does not match identity init"
fi
if node alice identity init >/dev/null 2>&1; then
    fail "identity init replaced an existing key"
else
    pass "identity init refuses to replace an existing key"
fi

# A payload large enough to need many records, small enough for one burst.
head -c 300000 /dev/urandom >"$WORK/payload.bin"
PAYLOAD_SHA="$(sha256 "$WORK/payload.bin")"

echo "== 1. Mutually pinned peers"
PORT="$(free_port)"
if start_receiver "$PORT" "$WORK/receiver-1.log"; then
    if node alice send-file --peer "/ip4/127.0.0.1/udp/$PORT" --file "$WORK/payload.bin" \
        --peer-key "$BOB_KEY" >"$WORK/sender-1.log" 2>&1; then
        if wait_for_file "$WORK/received/payload.bin" 300000 &&
            [ "$(sha256 "$WORK/received/payload.bin")" = "$PAYLOAD_SHA" ]; then
            pass "file arrived byte-identical (sha256 $PAYLOAD_SHA)"
        else
            fail "received file is missing or differs"
        fi
    else
        fail "authenticated send failed"
    fi
    if grep -q "event=tunnel_established role=responder peer=$ALICE_KEY" "$WORK/receiver-1.log"; then
        pass "receiver authenticated the sender as alice"
    else
        fail "receiver did not report alice as the authenticated peer"
    fi
    if grep -q "event=tunnel_established role=initiator peer=$BOB_KEY" "$WORK/sender-1.log"; then
        pass "sender authenticated the receiver as bob"
    else
        fail "sender did not report bob as the authenticated peer"
    fi
    SENDER_SESSION="$(sed -n 's/.*role=initiator.*session_id=\([0-9a-f]*\).*/\1/p' "$WORK/sender-1.log")"
    RECEIVER_SESSION="$(sed -n 's/.*role=responder.*session_id=\([0-9a-f]*\).*/\1/p' "$WORK/receiver-1.log")"
    if [ -n "$SENDER_SESSION" ] && [ "$SENDER_SESSION" = "$RECEIVER_SESSION" ]; then
        pass "both processes derived the same session id ($SENDER_SESSION)"
    else
        fail "session ids differ: '$SENDER_SESSION' vs '$RECEIVER_SESSION'"
    fi
else
    fail "receiver did not start"
fi
stop_receiver

echo "== 2. Sender that the receiver has not pinned"
PORT="$(free_port)"
if start_receiver "$PORT" "$WORK/receiver-2.log"; then
    if node mallory send-file --peer "/ip4/127.0.0.1/udp/$PORT" --file "$WORK/payload.bin" \
        --peer-key "$BOB_KEY" >"$WORK/sender-2.log" 2>&1; then
        fail "an unpinned sender was given a tunnel"
    else
        pass "unpinned sender was refused"
    fi
    if [ -z "$(ls -A "$WORK/received")" ] && ! grep -q "event=tunnel_established" "$WORK/receiver-2.log"; then
        pass "receiver established nothing and wrote nothing"
    else
        fail "receiver accepted data from an unpinned sender"
    fi
    if kill -0 "$RECEIVER_PID" 2>/dev/null; then
        # The rejected attempt must not disturb a pinned peer arriving later.
        if node alice send-file --peer "/ip4/127.0.0.1/udp/$PORT" --file "$WORK/payload.bin" \
            >"$WORK/sender-2b.log" 2>&1 &&
            wait_for_file "$WORK/received/payload.bin" 300000 &&
            [ "$(sha256 "$WORK/received/payload.bin")" = "$PAYLOAD_SHA" ]; then
            pass "pinned peer still connects after the rejected attempt"
        else
            fail "pinned peer could not connect after the rejected attempt"
        fi
    else
        fail "receiver exited after the rejected attempt"
    fi
else
    fail "receiver did not start"
fi
stop_receiver

echo "== 3. Receiver is not the identity the sender expects"
PORT="$(free_port)"
echo "$BOB_KEY bob" >"$WORK/alice/trusted_peers"
echo "$MALLORY_KEY mallory" >>"$WORK/alice/trusted_peers"
if start_receiver "$PORT" "$WORK/receiver-3.log"; then
    if node alice send-file --peer "/ip4/127.0.0.1/udp/$PORT" --file "$WORK/payload.bin" \
        --peer-key "$MALLORY_KEY" >"$WORK/sender-3.log" 2>&1; then
        fail "sender accepted a receiver with another identity"
    else
        pass "sender refused a receiver with another identity"
    fi
    if [ -z "$(ls -A "$WORK/received")" ]; then
        pass "nothing was delivered"
    else
        fail "data was delivered to the wrong identity"
    fi
    if node alice send-file --peer "/ip4/127.0.0.1/udp/$PORT" --file "$WORK/payload.bin" \
        >"$WORK/sender-3b.log" 2>&1; then
        fail "sender picked a peer although several are pinned"
    else
        pass "sender requires --peer-key when several peers are pinned"
    fi
else
    fail "receiver did not start"
fi
stop_receiver
echo "$BOB_KEY bob" >"$WORK/alice/trusted_peers"

echo "== 4. Missing identity or trust configuration"
PORT="$(free_port)"
mkdir -m 700 "$WORK/nobody"
echo "$BOB_KEY bob" >"$WORK/nobody/trusted_peers"
if node nobody send-file --peer "/ip4/127.0.0.1/udp/$PORT" --file "$WORK/payload.bin" \
    >"$WORK/sender-4.log" 2>&1; then
    fail "a node without an identity started"
elif [ -e "$WORK/nobody/identity.key" ]; then
    fail "an identity was generated implicitly"
else
    pass "missing identity fails closed and nothing is generated"
fi
: >"$WORK/bob/empty_trust"
if "$BINARY" --identity "$WORK/bob/identity.key" --trusted-peers "$WORK/bob/empty_trust" \
    receive-file --listen "/ip4/127.0.0.1/udp/$PORT" --output-dir "$WORK/received" \
    >"$WORK/receiver-4.log" 2>&1; then
    fail "a receiver that trusts nobody started listening"
else
    pass "receiver without pinned peers refuses to listen"
fi
chmod 644 "$WORK/alice/identity.key"
if node alice identity show >/dev/null 2>&1; then
    fail "a world-readable identity key was accepted"
else
    pass "world-readable identity key is rejected"
fi
chmod 600 "$WORK/alice/identity.key"

echo "== 5. Fixed test keys"
if "$BINARY" --test-mode send-file --peer "/ip4/127.0.0.1/udp/$PORT" \
    --file "$WORK/payload.bin" >"$WORK/sender-5.log" 2>&1; then
    fail "--test-mode worked without CRYPRQ_TEST_MODE"
else
    pass "--test-mode alone is refused"
fi
# Explicitly enabled on both ends, the insecure mode still interoperates.
PORT="$(free_port)"
rm -rf "$WORK/received"
mkdir -p "$WORK/received"
CRYPRQ_TEST_MODE=1 "$BINARY" --test-mode receive-file --listen "/ip4/127.0.0.1/udp/$PORT" \
    --output-dir "$WORK/received" >"$WORK/receiver-5.log" 2>&1 &
RECEIVER_PID=$!
sleep 1
if CRYPRQ_TEST_MODE=1 "$BINARY" --test-mode send-file --peer "/ip4/127.0.0.1/udp/$PORT" \
    --file "$WORK/payload.bin" >"$WORK/sender-5b.log" 2>&1 &&
    wait_for_file "$WORK/received/payload.bin" 300000 &&
    [ "$(sha256 "$WORK/received/payload.bin")" = "$PAYLOAD_SHA" ]; then
    pass "test mode transfers a file when enabled on both ends"
else
    fail "test mode did not transfer the file"
fi
if grep -q "INSECURE" "$WORK/receiver-5.log" && grep -q "INSECURE" "$WORK/sender-5b.log"; then
    pass "test mode warns that it is insecure"
else
    fail "test mode did not warn"
fi
stop_receiver

echo
if [ "$FAILURES" -eq 0 ]; then
    echo "E2E RESULT: PASSED"
    exit 0
fi
echo "E2E RESULT: FAILED ($FAILURES checks)"
exit 1
