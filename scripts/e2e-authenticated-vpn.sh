#!/bin/bash
# © 2025 Thor Thor
# SPDX-License-Identifier: MIT
#
# Two-peer end-to-end test of the authenticated VPN link (cryprq --vpn).
#
# Runs two separate cryprq processes, each with its own identity and its own
# pinned-peer file, connected over QUIC on loopback through a UDP relay that
# this script controls. IP packets enter and leave through --packet-bridge, a
# loopback UDP socket that stands in for the TUN device, so no root is needed.
# The TUN device itself is NOT exercised by this script.
#
# Checks:
#   1. mutually pinned peers establish one session and exchange IP packets
#      in both directions, and no packet payload is visible on the wire;
#   2. datagrams duplicated and replayed by the relay never deliver a packet
#      twice;
#   3. datagrams corrupted by the relay never deliver a modified packet;
#   4. keys rotate and the session is renewed while packets cross the link
#      without pause, and none is lost, changed or repeated;
#   5. teardown drops the session, packets are then dropped instead of sent,
#      and the link recovers after either side restarts;
#   6. an unpinned dialer, a listener with an unexpected identity, missing or
#      exposed identities, empty, missing or malformed trust files and the
#      unauthenticated test mode are all refused, each for the stated reason;
#   7. every peer shuts down by itself on SIGTERM and the test leaves no
#      process and no bound port behind.
#
# The relay sits below QUIC, so in checks 2 and 3 a bad datagram is normally
# rejected by QUIC before the CrypRQ record layer sees it. The record layer's
# own tamper and replay protection is tested without any transport protection
# in `cargo test -p p2p secure_vpn`.
#
# Usage: scripts/e2e-authenticated-vpn.sh [path/to/cryprq]
# Needs bash and python3; no root and no network beyond loopback. Logs are
# kept when a check fails, or always with CRYPRQ_E2E_KEEP_LOGS=1.

set -u

BINARY="${1:-./target/debug/cryprq}"
if [ ! -x "$BINARY" ]; then
    echo "ERROR: $BINARY not found. Build it first: cargo build -p cryprq" >&2
    exit 2
fi
BINARY="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/cryprq-vpn-e2e.XXXXXX")"
chmod 700 "$WORK"
FAILURES=0
PIDS=""
unset CRYPRQ_TEST_MODE
export RUST_LOG=info

# PIDS holds only children of this script that have not been reaped yet, so
# cleanup can never signal a process this test does not own.
forget() {
    local pid kept=""
    for pid in $PIDS; do
        [ "$pid" = "$1" ] || kept="$kept $pid"
    done
    PIDS="$kept"
}

any_alive() {
    local pid
    for pid in $PIDS; do
        kill -0 "$pid" 2>/dev/null && return 0
    done
    return 1
}

cleanup() {
    local pid
    for pid in $PIDS; do kill -TERM "$pid" 2>/dev/null; done
    for _ in $(seq 1 50); do
        any_alive || break
        sleep 0.1
    done
    for pid in $PIDS; do kill -KILL "$pid" 2>/dev/null; done
    wait 2>/dev/null
    # The identities are throwaway, but no secret key is left behind anyway.
    rm -f "$WORK"/*/identity.key
    if [ "$FAILURES" -eq 0 ] && [ -z "${CRYPRQ_E2E_KEEP_LOGS:-}" ]; then
        rm -rf "$WORK"
    else
        echo "Logs kept in $WORK"
    fi
}
trap cleanup EXIT
trap 'FAILURES=$((FAILURES + 1)); exit 130' INT TERM

pass() { echo "PASS: $1"; }
fail() {
    echo "FAIL: $1"
    FAILURES=$((FAILURES + 1))
}
check() {
    # check <description> <command...>
    local what="$1"
    shift
    if "$@"; then pass "$what"; else fail "$what"; fi
}

cat >"$WORK/relay.py" <<'PY'
"""UDP relay between the dialer and the listener, with fault injection.

The mode is read from a file before every datagram:
  pass     forward unchanged
  replay   forward every datagram twice and re-send an older one as well
  corrupt  flip one bit in every datagram from the dialer
Every datagram seen is appended to the capture file as hex.
"""
import socket, sys, select, collections

listen_port, server_port, mode_file, capture_file = sys.argv[1:5]
server = ("127.0.0.1", int(server_port))
front = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
front.bind(("127.0.0.1", int(listen_port)))
back = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
back.bind(("127.0.0.1", 0))
client = None
history = collections.deque(maxlen=64)
capture = open(capture_file, "a")

def mode():
    try:
        return open(mode_file).read().strip()
    except OSError:
        return "pass"

while True:
    ready, _, _ = select.select([front, back], [], [])
    for sock in ready:
        data, addr = sock.recvfrom(65535)
        capture.write(data.hex() + "\n")
        capture.flush()
        from_client = sock is front
        if from_client:
            client = addr
        elif client is None:
            continue
        send = (lambda d: back.sendto(d, server)) if from_client else (lambda d: front.sendto(d, client))
        current = mode()
        if current == "corrupt" and from_client:
            damaged = bytearray(data)
            damaged[len(damaged) // 2] ^= 0x10
            send(bytes(damaged))
            continue
        send(data)
        if current == "replay":
            send(data)
            if from_client and history:
                back.sendto(history[0], server)
        if from_client:
            history.append(data)
PY

cat >"$WORK/traffic.py" <<'PY'
"""Sends and receives IP packets through two packet bridges.

  exchange A B COUNT TAG   COUNT ICMP echo requests A->B and replies B->A;
                           every packet must arrive exactly once, unmodified
  blackhole A B TAG [gone] packets sent into A must not come out of B; with
                           "gone" the process behind B was stopped on purpose
  intact A B COUNT TAG S   send COUNT packets A->B; whatever comes out of B
                           within S seconds must be one of them, unmodified
                           and at most once
  stream A B SECONDS TAG   for SECONDS, a packet B->A and then one A->B, without
                           pause; every packet must arrive exactly once and
                           unmodified. Prints the number of round trips.
  settle A B               wait until both bridges are quiet; everything that
                           still arrives must be a well-formed echo request
"""
import socket, struct, sys, time

def checksum(data):
    if len(data) % 2:
        data += b"\0"
    total = sum(struct.unpack("!%dH" % (len(data) // 2), data))
    while total >> 16:
        total = (total & 0xFFFF) + (total >> 16)
    return ~total & 0xFFFF

def icmp(src, dst, kind, seq, payload):
    body = struct.pack("!BBHHH", kind, 0, 0, 0x4352, seq) + payload
    body = body[:2] + struct.pack("!H", checksum(body)) + body[4:]
    header = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + len(body), seq, 0, 64, 1, 0,
                         socket.inet_aton(src), socket.inet_aton(dst))
    header = header[:10] + struct.pack("!H", checksum(header)) + header[12:]
    return header + body

def bridge(port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind(("127.0.0.1", 0))
    sock.connect(("127.0.0.1", int(port)))
    sock.send(b"register")  # not an IP packet: only tells the bridge where we are
    return sock

def drain(sock, seconds, gone=False):
    """Everything that arrives within `seconds`.

    A refused connection means the process behind the bridge is not there. That
    is expected only where the caller stopped it (`gone`); anywhere else the
    peer died during the test, which is a failure.
    """
    out, deadline = [], time.time() + seconds
    while True:
        left = deadline - time.time()
        if left <= 0:
            return out
        sock.settimeout(left)
        try:
            out.append(sock.recv(65535))
        except socket.timeout:
            return out
        except ConnectionRefusedError:
            if not gone:
                sys.exit("a packet bridge refused the connection: its process is not running")
            time.sleep(max(0.0, deadline - time.time()))
            return out

def drain_refused(sock):
    """Whether the bridge behind `sock` is closed (an ICMP port unreachable)."""
    for _ in range(10):
        try:
            sock.send(b"register")
            sock.settimeout(0.1)
            sock.recv(65535)
        except ConnectionRefusedError:
            return True
        except socket.timeout:
            pass
    return False

def expect(sock, packet, what):
    sock.settimeout(10)
    try:
        got = sock.recv(65535)
    except socket.timeout:
        sys.exit("%s: nothing arrived" % what)
    except ConnectionRefusedError:
        sys.exit("%s: the packet bridge refused the connection: its process is not running" % what)
    if got != packet:
        sys.exit("%s: a different packet arrived" % what)

command = sys.argv[1]
a, b = bridge(sys.argv[2]), bridge(sys.argv[3])
time.sleep(0.2)
if command == "exchange":
    count, tag = int(sys.argv[4]), sys.argv[5].encode()
    for seq in range(count):
        request = icmp("10.0.0.2", "10.0.0.1", 8, seq, tag + b" request %d" % seq)
        a.send(request)
        expect(b, request, "request %d" % seq)
        reply = icmp("10.0.0.1", "10.0.0.2", 0, seq, tag + b" reply %d" % seq)
        b.send(reply)
        expect(a, reply, "reply %d" % seq)
    extra = drain(a, 1.0) + drain(b, 1.0)
    if extra:
        sys.exit("%d packets were delivered more than once" % len(extra))
elif command == "stream":
    seconds, tag = float(sys.argv[4]), sys.argv[5].encode()
    deadline, rounds = time.time() + seconds, 0
    while time.time() < deadline:
        seq = rounds & 0xFFFF
        first = icmp("10.0.0.1", "10.0.0.2", 8, seq, tag + b" listener first %d" % rounds)
        b.send(first)
        expect(a, first, "round %d, listener to dialer" % rounds)
        answer = icmp("10.0.0.2", "10.0.0.1", 0, seq, tag + b" dialer answers %d" % rounds)
        a.send(answer)
        expect(b, answer, "round %d, dialer to listener" % rounds)
        rounds += 1
    extra = drain(a, 1.0) + drain(b, 1.0)
    if extra:
        sys.exit("%d packets were delivered more than once" % len(extra))
    print(rounds)
elif command == "blackhole":
    tag = sys.argv[4].encode()
    for seq in range(5):
        a.send(icmp("10.0.0.2", "10.0.0.1", 8, seq, tag))
    gone = sys.argv[5:] == ["gone"]
    if drain(b, 2.0, gone):
        sys.exit("a packet came out although none should")
    if gone and not drain_refused(b):
        sys.exit("the bridge that should be gone still accepts packets")
elif command == "intact":
    count, tag = int(sys.argv[4]), sys.argv[5].encode()
    sent = [icmp("10.0.0.2", "10.0.0.1", 8, seq, tag + b" %d" % seq) for seq in range(count)]
    for packet in sent:
        a.send(packet)
        time.sleep(0.02)
    got = drain(b, float(sys.argv[6]))
    for packet in got:
        if packet not in sent:
            sys.exit("a modified packet was delivered")
    if len(set(got)) != len(got):
        sys.exit("a packet was delivered twice")
    print(len(got))
elif command == "settle":
    late = 0
    for _ in range(20):
        got = drain(a, 0.75) + drain(b, 0.75)
        if not got:
            break
        for packet in got:
            ok = len(packet) >= 28 and packet[0] == 0x45 and checksum(packet[:20]) == 0 \
                and checksum(packet[20:]) == 0 and b"corrupted" in packet
            if not ok:
                sys.exit("a damaged packet arrived")
        late += len(got)
    else:
        sys.exit("the link did not settle")
    print(late)
else:
    sys.exit("unknown command")
PY

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

# spawn <name> <log> <args...>: start cryprq as the named peer in the
# background. The binary is started directly, not through a shell function,
# so that SPAWNED is the PID of cryprq itself and can be signalled.
spawn() {
    local name="$1" log="$2"
    shift 2
    "$BINARY" --identity "$WORK/$name/identity.key" \
        --trusted-peers "$WORK/$name/trusted_peers" "$@" >>"$log" 2>&1 &
    SPAWNED=$!
    PIDS="$PIDS $SPAWNED"
}

# wait_for <file> <pattern> [count]: until the pattern occurs count times.
wait_for() {
    local file="$1" pattern="$2" count="${3:-1}"
    for _ in $(seq 1 300); do
        if [ "$(grep -c -- "$pattern" "$file" 2>/dev/null)" -ge "$count" ]; then
            return 0
        fi
        sleep 0.1
    done
    return 1
}

never() {
    # never <seconds> <file> <pattern>: the pattern must not appear.
    sleep "$1"
    ! grep -q -- "$3" "$2" 2>/dev/null
}

# nth_session <log> <n>: id of the n-th session the log reports.
nth_session() {
    sed -n 's/.*event=vpn_session_established.*session_id=\([0-9a-f]*\).*/\1/p' "$1" | sed -n "${2}p"
}

# listener_has <id>: the listener established the session with this id.
#
# The dialer reports a session only after it has verified the listener's key
# confirmation, the last message of the handshake, and the listener installs
# the session one message earlier. So an id taken from the dialer's log is
# one both ends have confirmed, and it must be in the listener's log; only
# the writing of that log line is waited for.
listener_has() {
    [ -n "$1" ] && wait_for "$B_LOG" "event=vpn_session_established.*session_id=$1"
}

sessions() {
    grep -c "event=vpn_session_established" "$1" 2>/dev/null
}

# exited <pid>: wait up to 10 s for the process to exit; false if it has not.
exited() {
    for _ in $(seq 1 100); do
        kill -0 "$1" 2>/dev/null || return 0
        sleep 0.1
    done
    return 1
}

# stop <pid>: ask the process to shut down with SIGTERM. Succeeds only if it
# was still running, exits by itself within 10 s and reports success. A process
# that had already died, or that has to be killed, is a failure.
stop() {
    local pid="$1" status
    if ! kill -TERM "$pid" 2>/dev/null; then
        wait "$pid" 2>/dev/null
        forget "$pid"
        echo "  process $pid was not running any more"
        return 1
    fi
    if ! exited "$pid"; then
        kill -KILL "$pid" 2>/dev/null
        wait "$pid" 2>/dev/null
        forget "$pid"
        echo "  process $pid ignored SIGTERM for 10 s and was killed"
        return 1
    fi
    wait "$pid" 2>/dev/null
    status=$?
    forget "$pid"
    [ "$status" -eq 0 ] || echo "  process $pid exited with status $status"
    return "$status"
}

# refused <log> <pattern> <command...>: the command must give up by itself
# within 10 s, with a failure status and the stated reason in its output. A
# command that keeps running, or fails for another reason, does not count.
refused() {
    local log="$1" pattern="$2" pid status
    shift 2
    "$@" >"$log" 2>&1 &
    pid=$!
    PIDS="$PIDS $pid"
    if ! exited "$pid"; then
        kill -KILL "$pid" 2>/dev/null
        wait "$pid" 2>/dev/null
        forget "$pid"
        echo "  the command was still running after 10 s"
        return 1
    fi
    wait "$pid" 2>/dev/null
    status=$?
    forget "$pid"
    if [ "$status" -eq 0 ] || [ "$status" -ge 128 ]; then
        echo "  the command ended with status $status"
        return 1
    fi
    if ! grep -q -- "$pattern" "$log"; then
        echo "  the command failed without the expected reason: $pattern"
        return 1
    fi
}

# running <pid>: the process is still alive.
running() { kill -0 "$1" 2>/dev/null; }

# port_free <port>: nothing is bound to the UDP port on loopback.
port_free() {
    python3 -c 'import socket, sys; socket.socket(socket.AF_INET, socket.SOCK_DGRAM).bind(("127.0.0.1", int(sys.argv[1])))' "$1" 2>/dev/null
}

start_listener() {
    # start_listener <log> [extra env assignments are taken from the caller]
    spawn bob "$1" --vpn --listen "/ip4/127.0.0.1/udp/$BOB_PORT/quic-v1" \
        --packet-bridge "127.0.0.1:$BOB_BRIDGE"
    BOB_PID=$SPAWNED
    wait_for "$1" "event=vpn_listening"
}

start_dialer() {
    spawn alice "$1" --vpn --peer "/ip4/127.0.0.1/udp/$RELAY_PORT/quic-v1" --peer-key "$BOB_KEY" \
        --packet-bridge "127.0.0.1:$ALICE_BRIDGE"
    ALICE_PID=$SPAWNED
}

traffic() { python3 "$WORK/traffic.py" "$@"; }

echo "== Provisioning identities in $WORK"
for name in alice bob mallory; do
    mkdir -m 700 "$WORK/$name"
    if ! node "$name" identity init >"$WORK/$name/public_key" 2>"$WORK/$name/init.log"; then
        echo "ERROR: identity init failed for $name" >&2
        FAILURES=1
        exit 1
    fi
done
ALICE_KEY="$(cat "$WORK/alice/public_key")"
BOB_KEY="$(cat "$WORK/bob/public_key")"
MALLORY_KEY="$(cat "$WORK/mallory/public_key")"
echo "$BOB_KEY bob" >"$WORK/alice/trusted_peers"
echo "$ALICE_KEY alice" >"$WORK/bob/trusted_peers"
echo "$BOB_KEY bob" >"$WORK/mallory/trusted_peers"

BOB_PORT="$(free_port)"
RELAY_PORT="$(free_port)"
ALICE_BRIDGE="$(free_port)"
BOB_BRIDGE="$(free_port)"
MALLORY_BRIDGE="$(free_port)"
A_LOG="$WORK/alice.log"
B_LOG="$WORK/bob.log"
echo pass >"$WORK/relay.mode"
python3 "$WORK/relay.py" "$RELAY_PORT" "$BOB_PORT" "$WORK/relay.mode" "$WORK/capture.hex" &
RELAY_PID=$!
PIDS="$PIDS $RELAY_PID"

echo "== 1. Mutually pinned peers"
# Keys rotate every 2 s for the whole run; the session is renewed every 8 s.
export CRYPRQ_ROTATE_SECS=2
export CRYPRQ_REHANDSHAKE_SECS=8
check "listener started" start_listener "$B_LOG"
start_dialer "$A_LOG"
check "dialer established a session" wait_for "$A_LOG" "event=vpn_session_established role=Initiator peer=$BOB_KEY"
check "listener established a session" wait_for "$B_LOG" "event=vpn_session_established role=Responder peer=$ALICE_KEY"
FIRST_SESSION="$(nth_session "$A_LOG" 1)"
if listener_has "$FIRST_SESSION"; then
    pass "both processes derived the same session id ($FIRST_SESSION)"
else
    fail "the listener has no session ${FIRST_SESSION:-(none at the dialer)}"
fi
MARKER="plaintext-marker-$(date +%s)-0123456789abcdef"
check "20 ICMP round trips arrive exactly once and unmodified" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 20 "$MARKER"
MARKER_HEX="$(printf '%s' "$MARKER" | od -An -tx1 | tr -d ' \n')"
if [ -s "$WORK/capture.hex" ] && ! grep -q "$MARKER_HEX" "$WORK/capture.hex"; then
    pass "packet payload is not visible in $(wc -l <"$WORK/capture.hex" | tr -d ' ') captured datagrams"
else
    fail "packet payload is visible on the wire, or nothing was captured"
fi

echo "== 2. Duplicated and replayed datagrams"
echo replay >"$WORK/relay.mode"
check "no packet is delivered twice under replay" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 15 "replayed"
echo pass >"$WORK/relay.mode"

echo "== 3. Corrupted datagrams"
echo corrupt >"$WORK/relay.mode"
DELIVERED="$(traffic intact "$ALICE_BRIDGE" "$BOB_BRIDGE" 20 "corrupted" 2)"
if [ $? -eq 0 ]; then
    pass "no modified packet was delivered while datagrams were corrupted ($DELIVERED of 20 arrived)"
else
    fail "a modified or duplicated packet was delivered: $DELIVERED"
fi
echo pass >"$WORK/relay.mode"
# Packets held back during the corruption may still arrive, late but intact.
LATE="$(traffic settle "$ALICE_BRIDGE" "$BOB_BRIDGE")"
if [ $? -eq 0 ]; then
    pass "packets delayed by the corruption arrived intact or not at all ($LATE late)"
else
    fail "a damaged packet arrived after the corruption: $LATE"
fi
check "traffic flows again once the corruption stops" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 5 "recovered"

echo "== 4. Key rotation and session renewal"
check "dialer rotated its keys" wait_for "$A_LOG" "event=vpn_keys_rotated"
check "listener rotated its keys" wait_for "$B_LOG" "event=vpn_keys_rotated"
check "traffic flows after key rotation" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 5 "rotated"
# Packets cross the link without pause for longer than one renewal interval,
# the listener speaking first each time. A packet sent while the two ends
# are between the old and the new session would be missing here.
BEFORE="$(sessions "$A_LOG")"
ROUNDS="$(traffic stream "$ALICE_BRIDGE" "$BOB_BRIDGE" 10 "through-renewal")"
if [ $? -eq 0 ]; then
    pass "$ROUNDS round trips arrived exactly once while the session was renewed"
else
    fail "a packet was lost, changed or repeated during the renewal: $ROUNDS"
fi
if [ "$(sessions "$A_LOG")" -gt "$BEFORE" ] && grep -q "event=vpn_session_renewal" "$A_LOG"; then
    pass "the dialer renewed the session during that traffic"
else
    fail "no renewal happened during the traffic, so it proved nothing"
fi
RENEWED="$(nth_session "$A_LOG" "$((BEFORE + 1))")"
if [ "$RENEWED" != "$FIRST_SESSION" ] && listener_has "$RENEWED"; then
    pass "renewal produced a new session id on both sides ($RENEWED)"
else
    fail "renewal did not produce a new common session id (first $FIRST_SESSION, dialer ${RENEWED:-none})"
fi
check "traffic flows after the renewal" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 5 "renewed"

echo "== 5. Teardown and recovery"
# No renewals from here on, so session ids only change through reconnection.
export CRYPRQ_REHANDSHAKE_SECS=3600
check "dialer shut down cleanly on SIGTERM" stop "$ALICE_PID"
check "dialer released its packet bridge" port_free "$ALICE_BRIDGE"
check "listener dropped the session when the dialer left" wait_for "$B_LOG" "event=vpn_session_closed"
check "packets are dropped, not sent, without a session" traffic blackhole "$BOB_BRIDGE" "$ALICE_BRIDGE" "void" gone
check "the drop is reported" wait_for "$B_LOG" "event=vpn_packet_dropped reason=\"no session\""

BEFORE="$(sessions "$B_LOG")"
# A restarted peer writes a new log; the old one is kept for diagnosis.
A_LOG="$WORK/alice-restarted.log"
start_dialer "$A_LOG"
check "dialer re-established after its restart" wait_for "$A_LOG" "event=vpn_session_established"
check "listener accepted the new session" wait_for "$B_LOG" "event=vpn_session_established" "$((BEFORE + 1))"
AFTER_DIALER_RESTART="$(nth_session "$A_LOG" 1)"
if [ "$AFTER_DIALER_RESTART" != "$RENEWED" ] && ! grep -q "session_id=$AFTER_DIALER_RESTART" "$WORK/alice.log" &&
    listener_has "$AFTER_DIALER_RESTART"; then
    pass "recovery ran a new handshake (session $AFTER_DIALER_RESTART)"
else
    fail "recovery did not produce a new common session id"
fi
check "traffic flows after the dialer restart" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 5 "dialer-restart"

check "listener shut down cleanly on SIGTERM" stop "$BOB_PID"
check "dialer dropped the session when the listener left" wait_for "$A_LOG" "event=vpn_session_closed"
check "packets are dropped while the listener is away" traffic blackhole "$ALICE_BRIDGE" "$BOB_BRIDGE" "listener-away" gone
check "listener released its address" port_free "$BOB_PORT"
check "listener released its packet bridge" port_free "$BOB_BRIDGE"
B_LOG="$WORK/bob-restarted.log"
check "listener restarted on the same address" start_listener "$B_LOG"
check "dialer reconnected on its own" wait_for "$A_LOG" "event=vpn_session_established" 2
check "restarted listener has a session" wait_for "$B_LOG" "event=vpn_session_established"
AFTER_LISTENER_RESTART="$(nth_session "$A_LOG" 2)"
if [ "$AFTER_LISTENER_RESTART" != "$AFTER_DIALER_RESTART" ] && listener_has "$AFTER_LISTENER_RESTART"; then
    pass "reconnection ran a new handshake (session $AFTER_LISTENER_RESTART)"
else
    fail "reconnection did not produce a new common session id"
fi
check "traffic flows after the listener restart" traffic exchange "$ALICE_BRIDGE" "$BOB_BRIDGE" 5 "listener-restart"
check "dialer shut down cleanly a second time" stop "$ALICE_PID"

echo "== 6. Peers that must be refused"
# Mallory has Bob's address and key and pins him; Bob has not pinned Mallory.
BEFORE="$(sessions "$B_LOG")"
spawn mallory "$WORK/mallory.log" --vpn --peer "/ip4/127.0.0.1/udp/$BOB_PORT/quic-v1" \
    --peer-key "$BOB_KEY" --packet-bridge "127.0.0.1:$MALLORY_BRIDGE"
MALLORY_PID=$SPAWNED
check "listener refused the unpinned transport peer" wait_for "$B_LOG" "event=vpn_rejected.*not a pinned identity"
check "unpinned dialer got no session" never 3 "$WORK/mallory.log" "event=vpn_session_established"
check "unpinned dialer was still trying, not crashed" running "$MALLORY_PID"
if [ "$(sessions "$B_LOG")" = "$BEFORE" ]; then
    pass "listener established nothing for the unpinned dialer"
else
    fail "listener established a session for an unpinned dialer"
fi
check "packets of the unpinned dialer are not delivered" traffic blackhole "$MALLORY_BRIDGE" "$BOB_BRIDGE" "mallory"
check "unpinned dialer shut down cleanly" stop "$MALLORY_PID"

# Alice pins Bob and Mallory and dials Bob's address expecting Mallory.
echo "$MALLORY_KEY mallory" >>"$WORK/alice/trusted_peers"
spawn alice "$WORK/alice-wrong.log" --vpn --peer "/ip4/127.0.0.1/udp/$BOB_PORT/quic-v1" \
    --peer-key "$MALLORY_KEY" --packet-bridge "127.0.0.1:$ALICE_BRIDGE"
WRONG_PID=$SPAWNED
check "dialer reached the listener and refused its identity" wait_for "$WORK/alice-wrong.log" "event=vpn_rejected.*not the pinned peer"
check "dialer got no session with another identity" never 3 "$WORK/alice-wrong.log" "event=vpn_session_established"
check "dialer was still trying, not crashed" running "$WRONG_PID"
check "nothing is delivered to the wrong identity" traffic blackhole "$ALICE_BRIDGE" "$BOB_BRIDGE" "wrong-identity"
check "dialer shut down cleanly" stop "$WRONG_PID"
if [ "$(sessions "$B_LOG")" = "$BEFORE" ]; then
    pass "listener established nothing during the refused attempts"
else
    fail "listener established a session during the refused attempts"
fi

# Configurations that must not start at all. Each command has to give up by
# itself and name the reason; a start-up that merely fails does not count.
alice_vpn() {
    refused "$1" "$2" "$BINARY" --identity "$WORK/alice/identity.key" \
        --trusted-peers "$WORK/alice/trusted_peers" --vpn "${@:3}"
}
bob_listen() {
    # bob_listen <log> <pattern> <identity> <trusted-peers>
    refused "$1" "$2" "$BINARY" --identity "$3" --trusted-peers "$4" --vpn \
        --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1" --packet-bridge "127.0.0.1:$(free_port)"
}
BOB_ADDR="/ip4/127.0.0.1/udp/$BOB_PORT/quic-v1"
check "dialer requires --peer-key when several peers are pinned" \
    alice_vpn "$WORK/alice-ambiguous.log" "several peers are pinned" \
    --peer "$BOB_ADDR" --packet-bridge "127.0.0.1:$ALICE_BRIDGE"
echo "$BOB_KEY bob" >"$WORK/alice/trusted_peers"
check "dialer refuses a --peer-key that is not pinned" \
    alice_vpn "$WORK/alice-unpinned.log" "not a pinned identity" \
    --peer "$BOB_ADDR" --peer-key "$MALLORY_KEY" --packet-bridge "127.0.0.1:$ALICE_BRIDGE"
check "--vpn needs exactly one of --listen and --peer" \
    alice_vpn "$WORK/alice-both.log" "exactly one of --listen and --peer" \
    --peer "$BOB_ADDR" --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1" \
    --packet-bridge "127.0.0.1:$ALICE_BRIDGE"
check "the packet bridge is restricted to loopback" \
    alice_vpn "$WORK/bridge.log" "must be a loopback address" \
    --peer "$BOB_ADDR" --packet-bridge "0.0.0.0:$(free_port)"

mkdir -m 700 "$WORK/nobody"
echo "$BOB_KEY bob" >"$WORK/nobody/trusted_peers"
check "missing identity fails closed" \
    bob_listen "$WORK/nobody.log" "identity.key: not found" \
    "$WORK/nobody/identity.key" "$WORK/nobody/trusted_peers"
if [ -e "$WORK/nobody/identity.key" ]; then
    fail "an identity was generated implicitly"
else
    pass "no identity was generated implicitly"
fi
cp "$WORK/bob/identity.key" "$WORK/bob/exposed.key"
chmod 644 "$WORK/bob/exposed.key"
check "identity readable by others is refused" \
    bob_listen "$WORK/exposed.log" "unsafe key storage" \
    "$WORK/bob/exposed.key" "$WORK/bob/trusted_peers"
rm -f "$WORK/bob/exposed.key"
: >"$WORK/bob/empty_trust"
check "VPN listener without pinned peers refuses to start" \
    bob_listen "$WORK/empty-trust.log" "pins no peers" \
    "$WORK/bob/identity.key" "$WORK/bob/empty_trust"
check "missing trusted-peers file is refused" \
    bob_listen "$WORK/missing-trust.log" "no_such_file: not found" \
    "$WORK/bob/identity.key" "$WORK/bob/no_such_file"
echo "ed25519:zz not-a-key" >"$WORK/bob/bad_trust"
check "malformed trusted-peers file is refused" \
    bob_listen "$WORK/bad-trust.log" "trusted-peers file is malformed" \
    "$WORK/bob/identity.key" "$WORK/bob/bad_trust"
check "a reserved TUN address is refused before a device is requested" \
    refused "$WORK/tun-address.log" "is reserved and cannot be assigned to a tunnel" \
    "$BINARY" --identity "$WORK/bob/identity.key" --trusted-peers "$WORK/bob/trusted_peers" \
    --vpn --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1" --tun-address 127.0.0.1
check "a netmask that is not a prefix is refused before a device is requested" \
    refused "$WORK/tun-netmask.log" "is not a contiguous prefix" \
    "$BINARY" --identity "$WORK/bob/identity.key" --trusted-peers "$WORK/bob/trusted_peers" \
    --vpn --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1" \
    --tun-address 172.31.250.1 --tun-netmask 255.0.255.0
check "the network address of the chosen netmask is refused as a TUN address" \
    refused "$WORK/tun-network.log" "is the network or broadcast address of its /30 subnet" \
    "$BINARY" --identity "$WORK/bob/identity.key" --trusted-peers "$WORK/bob/trusted_peers" \
    --vpn --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1" \
    --tun-address 172.31.250.4 --tun-netmask 255.255.255.252
check "the unauthenticated test-mode VPN is refused unless explicitly enabled" \
    refused "$WORK/test-mode.log" "requires CRYPRQ_TEST_MODE=1" \
    "$BINARY" --test-mode --vpn --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1"
check "the packet bridge is never available in test mode" \
    refused "$WORK/test-mode-bridge.log" "require --vpn without --test-mode" \
    env CRYPRQ_TEST_MODE=1 "$BINARY" --test-mode --vpn \
    --listen "/ip4/127.0.0.1/udp/$(free_port)/quic-v1" --packet-bridge "127.0.0.1:$(free_port)"

echo "== 7. Shutdown and cleanup"
check "listener was still running after every refused attempt" running "$BOB_PID"
check "listener shut down cleanly on SIGTERM" stop "$BOB_PID"
kill -TERM "$RELAY_PID" 2>/dev/null
if exited "$RELAY_PID"; then
    wait "$RELAY_PID" 2>/dev/null
    forget "$RELAY_PID"
    pass "relay stopped"
else
    fail "relay did not stop"
fi
if [ -z "$PIDS" ] && [ -z "$(jobs -pr)" ]; then
    pass "no process started by this test is left"
else
    fail "processes are left behind: $PIDS $(jobs -pr | tr '\n' ' ')"
fi
BUSY=""
for port in "$BOB_PORT" "$RELAY_PORT" "$ALICE_BRIDGE" "$BOB_BRIDGE" "$MALLORY_BRIDGE"; do
    port_free "$port" || BUSY="$BUSY $port"
done
if [ -z "$BUSY" ]; then
    pass "every UDP port used by this test is free again"
else
    fail "UDP ports still in use:$BUSY"
fi

echo
if [ "$FAILURES" -eq 0 ]; then
    echo "VPN E2E RESULT: PASSED"
    exit 0
fi
echo "VPN E2E RESULT: FAILED ($FAILURES checks)"
exit 1
