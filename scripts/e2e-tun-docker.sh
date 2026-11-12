#!/usr/bin/env bash
# Copyright (c) 2025 Thor Thor
# License: MIT (see LICENSE file for details)
#
# Device test of `cryprq --vpn` on real TUN devices, in Linux containers.
#
# Two endpoints run in two containers, each in its own network namespace with
# its own TUN device, on a Docker network that has no route to the outside.
# The host's interfaces, routes and DNS are not touched: no sudo on the host,
# no host networking, no published ports, no mounts, no --privileged. The
# containers get NET_ADMIN and /dev/net/tun and nothing else.
#
# What is checked: the kernel of one container sends packets into its TUN
# device, they cross the authenticated link, and the kernel of the other
# container receives them from its TUN device, in both directions; the wire
# between the containers carries only QUIC; an identity that is not pinned
# gets no session; without a session nothing is forwarded; everything the
# test created is removed again, and the host's network state is unchanged.
#
# It does NOT check routes, DNS or leak protection: `cryprq --vpn` installs
# none, and a point-to-point link between two containers says nothing about
# a host's traffic.
#
# Usage: scripts/e2e-tun-docker.sh
#   CRYPRQ_TUN_EVIDENCE=<dir>   where evidence is kept (default ~/cryprq-tun-evidence)
#   CRYPRQ_TUN_KEEP_IMAGES=1    keep base images this run had to pull
# Needs: docker (Linux containers), git, bash, timeout. Builds the committed
# HEAD of this checkout, not the working tree.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ID="$(date +%Y%m%d-%H%M%S)-$$"
P="cryprq-tuntest-$ID"
LABEL="cryprq.tuntest=$ID"
IMAGE="$P:local"
E="${CRYPRQ_TUN_EVIDENCE:-$HOME/cryprq-tun-evidence}/$ID"

LINK_SUBNET=172.31.251.0/29 # between the containers, inside Docker's VM only
BOB_IP=172.31.251.2
ALICE_IP=172.31.251.3
MALLORY_IP=172.31.251.4
TUN_SUBNET=172.31.250.0/30 # inside the containers' namespaces only
TUN_BOB=172.31.250.1
TUN_ALICE=172.31.250.2
MASK=255.255.255.252
PORT=9999
PATTERN=c0ffee11deadbeef

FAILURES=0
CHECKS=0
NET_ID=""
IMAGE_ID=""
CONTAINERS=()
PULLED=()
CTX=""

say() { printf '%s\n' "$*"; }
pass() {
    CHECKS=$((CHECKS + 1))
    say "PASS: $1"
}
fail() {
    CHECKS=$((CHECKS + 1))
    FAILURES=$((FAILURES + 1))
    say "FAIL: $1"
}
# check "description" command...   passes when the command succeeds
check() {
    local what="$1"
    shift
    if "$@"; then pass "$what"; else fail "$what"; fi
}
abort() {
    say "ABORT: $1"
    FAILURES=$((FAILURES + 1))
    exit 2
}

# dx <seconds> <container> <shell command>: run inside a container, bounded.
dx() {
    local limit="$1" c="$2"
    shift 2
    timeout "$limit" docker exec "$c" sh -c "$*"
}
# waitfor <seconds> <container> <shell test>: poll until the test succeeds.
waitfor() {
    local limit="$1" c="$2" t="$3" start=$SECONDS
    while [ $((SECONDS - start)) -lt "$limit" ]; do
        if timeout 5 docker exec "$c" sh -c "$t" >/dev/null 2>&1; then return 0; fi
        sleep 0.5
    done
    return 1
}

# ---- host and Docker state, for the comparison at the end -------------------

host_state() {
    local tag="$1"
    if command -v scutil >/dev/null 2>&1 && command -v netstat >/dev/null 2>&1; then
        # macOS. Routes: destination, gateway, flags, interface. Link-layer
        # and cloned host entries come and go on their own and are left out.
        netstat -rn -f inet |
            awk 'NF>=4 && $3 !~ /[LWc]/ && $1 != "Destination" {print $1, $2, $3, $4}' | sort >"$E/host-routes4.$tag"
        netstat -rn -f inet6 |
            awk 'NF>=4 && $3 !~ /[LWc]/ && $1 != "Destination" {print $1, $2, $3, $4}' | sort >"$E/host-routes6.$tag"
        scutil --dns >"$E/host-dns.$tag"
        scutil --nc list >"$E/host-vpn.$tag"
        ifconfig -a | awk '/^[a-z0-9]+:/ {name=$1} /inet / {print name, $1, $2, $3, $4}' |
            sort >"$E/host-addresses.$tag"
        ifconfig -l | tr ' ' '\n' | sort >"$E/host-interfaces.$tag"
    elif command -v ip >/dev/null 2>&1; then
        # Linux. Lifetimes and carrier state change on their own.
        ip -4 route show | sed -E 's/ expires [0-9]+sec//; s/ linkdown//' | sort >"$E/host-routes4.$tag"
        ip -6 route show | sed -E 's/ expires [0-9]+sec//; s/ linkdown//' | sort >"$E/host-routes6.$tag"
        cat /etc/resolv.conf >"$E/host-dns.$tag" 2>/dev/null || : >"$E/host-dns.$tag"
        # No VPN service list on Linux; tunnel devices show up as interfaces.
        ip -o link show | awk -F': ' '$2 ~ /^(tun|tap|wg|ppp|utun)/ {print $2}' | sort >"$E/host-vpn.$tag"
        ip -o addr show | awk '{print $2, $3, $4}' | sort >"$E/host-addresses.$tag"
        ip -o link show | awk -F': ' '{print $2}' | sed 's/@.*//' | sort >"$E/host-interfaces.$tag"
    else
        return 1
    fi
    docker ps -a --format '{{.ID}} {{.Names}} {{.Image}} {{.State}}' | sort >"$E/docker-containers.$tag"
    docker network ls --format '{{.ID}} {{.Name}} {{.Driver}}' | sort >"$E/docker-networks.$tag"
    docker image ls --format '{{.ID}} {{.Repository}}:{{.Tag}}' | sort >"$E/docker-images.$tag"
    docker volume ls --format '{{.Name}}' | sort >"$E/docker-volumes.$tag"
}

unchanged() {
    local name="$1"
    if diff "$E/$name.before" "$E/$name.after" >"$E/$name.diff" 2>&1; then
        rm -f "$E/$name.diff"
        return 0
    fi
    say "  differences in $name:"
    sed 's/^/    /' "$E/$name.diff"
    return 1
}

# ---- cleanup: only what this run created, by id ------------------------------

cleanup() {
    local status=$?
    set +e
    say ""
    say "== cleanup"
    for c in "${CONTAINERS[@]:-}"; do
        [ -n "$c" ] || continue
        timeout 60 docker rm -f "$c" >/dev/null 2>&1
    done
    [ -n "$NET_ID" ] && timeout 60 docker network rm "$NET_ID" >/dev/null 2>&1
    [ -n "$IMAGE_ID" ] && timeout 60 docker image rm "$IMAGE" >/dev/null 2>&1
    if [ "${CRYPRQ_TUN_KEEP_IMAGES:-0}" != 1 ]; then
        for ref in "${PULLED[@]:-}"; do
            [ -n "$ref" ] || continue
            timeout 120 docker image rm "$ref" >/dev/null 2>&1
        done
    fi
    [ -n "$CTX" ] && rm -rf "$CTX"

    local left
    left="$(docker ps -a -q --filter "label=$LABEL")$(docker network ls -q --filter "label=$LABEL")$(docker image ls -q --filter "label=$LABEL")"
    check "nothing labelled $LABEL is left in Docker" [ -z "$left" ]

    if [ -f "$E/host-routes4.before" ]; then
        host_state after
        check "host IPv4 routes are unchanged" unchanged host-routes4
        check "host IPv6 routes are unchanged" unchanged host-routes6
        check "host DNS configuration is unchanged" unchanged host-dns
        check "host VPN services are in the same state" unchanged host-vpn
        check "host interfaces are unchanged" unchanged host-interfaces
        check "host addresses are unchanged" unchanged host-addresses
        check "Docker containers are the same as before" unchanged docker-containers
        check "Docker networks are the same as before" unchanged docker-networks
        check "Docker volumes are the same as before" unchanged docker-volumes
        if [ "${CRYPRQ_TUN_KEEP_IMAGES:-0}" != 1 ]; then
            check "Docker images are the same as before" unchanged docker-images
        fi
    fi
    if find "$E" -type f \( -name '*.key' -o -name 'identity*' \) | grep -q .; then
        fail "no identity key is kept with the evidence"
    else
        pass "no identity key is kept with the evidence"
    fi

    say ""
    say "Evidence: $E"
    if [ "$FAILURES" -eq 0 ] && [ "$status" -eq 0 ]; then
        say "TUN E2E RESULT: PASSED ($CHECKS checks)"
        exit 0
    fi
    say "TUN E2E RESULT: FAILED ($FAILURES of $CHECKS checks)"
    exit 1
}

# ---- preconditions -----------------------------------------------------------

mkdir -p "$E"
say "== preconditions (run $ID)"
command -v timeout >/dev/null || { say "ABORT: timeout(1) is required"; exit 2; }
timeout 30 docker info >/dev/null 2>&1 || { say "ABORT: Docker is not available"; exit 2; }
[ "$(docker info --format '{{.OSType}}')" = linux ] || { say "ABORT: Docker does not run Linux containers"; exit 2; }
docker info --format '{{.OperatingSystem}} {{.ServerVersion}} {{.Architecture}}' >"$E/docker-info.txt"

# No existing Docker network may use the address ranges of this test, and no
# resource may already carry this run's names.
for n in $(docker network ls -q); do
    for subnet in $(docker network inspect "$n" --format '{{range .IPAM.Config}}{{.Subnet}} {{end}}'); do
        case "$subnet" in
        172.31.25[01].* | 172.31.0.0/* | 172.16.0.0/12 | 172.0.0.0/*)
            say "ABORT: Docker network $(docker network inspect "$n" --format '{{.Name}}') uses $subnet"
            exit 2
            ;;
        esac
    done
done
if docker ps -a --format '{{.Names}}' | grep -q "^$P" || docker network ls --format '{{.Name}}' | grep -q "^$P"; then
    say "ABORT: a Docker resource named $P* already exists"
    exit 2
fi

trap cleanup EXIT
if ! host_state before || [ ! -s "$E/host-routes4.before" ] || [ ! -s "$E/host-interfaces.before" ]; then
    rm -f "$E"/host-*.before
    abort "the host's network state cannot be recorded, so it could not be compared afterwards"
fi
say "host and Docker state recorded"

# ---- build: committed files only ---------------------------------------------

say ""
say "== build"
CTX="$(mktemp -d)"
git -C "$ROOT" archive HEAD | tar -x -C "$CTX" || abort "cannot export HEAD"
git -C "$ROOT" rev-parse HEAD >"$E/commit.txt"
[ -z "$(git -C "$ROOT" status --porcelain)" ] || say "note: the working tree has changes; HEAD ($(cut -c1-7 "$E/commit.txt")) is what gets built"
(cd "$CTX" && find . \( -type f -o -type l \) | sed 's|^\./||' | sort) >"$E/build-context-files.txt"
if grep -iE '(^|/)(\.env[^/]*|.*\.(key|pem|p12|pfx)|id_(rsa|ed25519)[^/]*|.*credential.*|\.npmrc|\.netrc)$' "$E/build-context-files.txt"; then
    abort "the build context contains files that look like credentials"
fi
untracked="$(git -C "$ROOT" ls-tree -r --name-only HEAD | sort | comm -13 - "$E/build-context-files.txt")"
check "the build context holds only files committed at HEAD ($(wc -l <"$E/build-context-files.txt" | tr -d ' ') files)" \
    [ -z "$untracked" ]
[ -z "$untracked" ] || abort "the build context holds files that are not committed: $untracked"

# The toolchain the tree asks for, so the image builds with it directly.
RUST_VERSION="$(sed -n 's/^channel *= *"\([0-9.]*\)".*/\1/p' "$CTX/rust-toolchain.toml" 2>/dev/null | head -1)"
[ -n "$RUST_VERSION" ] || abort "cannot read the toolchain version from rust-toolchain.toml"
cat >"$CTX/Dockerfile.tuntest" <<DOCKER
FROM rust:$RUST_VERSION AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p cryprq --locked
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
      iproute2 iputils-ping tcpdump procps netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/cryprq /usr/local/bin/cryprq
DOCKER
for ref in "rust:$RUST_VERSION" debian:bookworm-slim; do
    docker image inspect "$ref" >/dev/null 2>&1 || PULLED+=("$ref")
done
if ! timeout 2400 docker build -q --label "$LABEL" -t "$IMAGE" -f "$CTX/Dockerfile.tuntest" "$CTX" >"$E/build.log" 2>&1; then
    tail -20 "$E/build.log"
    abort "the image could not be built"
fi
IMAGE_ID="$(docker image inspect "$IMAGE" --format '{{.Id}}')"
pass "image built from $(cut -c1-7 "$E/commit.txt")"

check "/dev/net/tun exists in Docker's VM" \
    timeout 60 docker run --rm --label "$LABEL" --network none --device /dev/net/tun "$IMAGE" test -c /dev/net/tun

# ---- network and containers --------------------------------------------------

say ""
say "== containers"
NET_ID="$(timeout 60 docker network create --internal --label "$LABEL" --subnet "$LINK_SUBNET" "$P-net")" ||
    abort "cannot create the internal network"

start_container() {
    local name="$1" ip="$2" id
    id="$(timeout 60 docker run -d --name "$P-$name" --hostname "$name" --label "$LABEL" \
        --network "$P-net" --ip "$ip" --cap-add NET_ADMIN --device /dev/net/tun \
        "$IMAGE" sleep 3600)" || abort "cannot start $name"
    CONTAINERS+=("$id")
}
start_container bob "$BOB_IP"
start_container alice "$ALICE_IP"
start_container mallory "$MALLORY_IP"
BOB="$P-bob"
ALICE="$P-alice"
MALLORY="$P-mallory"

confined() {
    local c="$1" got
    got="$(docker inspect "$c" --format \
        'priv={{.HostConfig.Privileged}} net={{.HostConfig.NetworkMode}} pid={{.HostConfig.PidMode}} capadd={{.HostConfig.CapAdd}} mounts={{len .Mounts}} binds={{len .HostConfig.Binds}} ports={{len .HostConfig.PortBindings}} devices={{range .HostConfig.Devices}}{{.PathOnHost}} {{end}}')"
    echo "$c $got" >>"$E/container-confinement.txt"
    [ "$got" = "priv=false net=$P-net pid= capadd=[CAP_NET_ADMIN] mounts=0 binds=0 ports=0 devices=/dev/net/tun " ]
}
for c in "$BOB" "$ALICE" "$MALLORY"; do
    check "$c: not privileged, no host network, mounts or ports; only NET_ADMIN and /dev/net/tun" confined "$c"
done
check "the container network is internal" \
    [ "$(docker network inspect "$NET_ID" --format '{{.Internal}}')" = true ]
namespaces="$(for c in "$BOB" "$ALICE" "$MALLORY"; do dx 10 "$c" 'readlink /proc/1/ns/net'; done | sort -u | wc -l | tr -d ' ')"
check "each container has its own network namespace" [ "$namespaces" = 3 ]
if [ "$namespaces" != 3 ]; then abort "namespace isolation failed"; fi
for c in "$BOB" "$ALICE"; do
    check "$c has no default route" sh -c "! timeout 10 docker exec $c ip route | grep -q '^default'"
    check "$c cannot reach an outside address" \
        sh -c "! timeout 15 docker exec $c ping -c 1 -W 2 192.0.2.1 >/dev/null 2>&1"
done

# ---- identities --------------------------------------------------------------

for c in "$BOB" "$ALICE" "$MALLORY"; do
    dx 20 "$c" 'cryprq identity init >/dev/null 2>&1' || abort "cannot create an identity in $c"
done
key_of() { dx 20 "$1" 'cryprq identity show 2>/dev/null' | grep -o 'ed25519:[0-9a-f]*' | head -1; }
BOB_KEY="$(key_of "$BOB")"
ALICE_KEY="$(key_of "$ALICE")"
[ -n "$BOB_KEY" ] && [ -n "$ALICE_KEY" ] && [ "$BOB_KEY" != "$ALICE_KEY" ] || abort "identities are missing"
dx 10 "$BOB" "echo $ALICE_KEY > /root/.cryprq/trusted_peers"
dx 10 "$ALICE" "echo $BOB_KEY > /root/.cryprq/trusted_peers"
dx 10 "$MALLORY" "echo $BOB_KEY > /root/.cryprq/trusted_peers" # bob does not pin mallory
for c in bob alice; do dx 10 "$P-$c" 'ip -br addr; echo; ip route' >"$E/$c-network.before" 2>&1; done

start_bob() {
    docker exec -d "$BOB" sh -c "RUST_LOG=info cryprq --vpn \
        --listen /ip4/$BOB_IP/udp/$PORT/quic-v1 \
        --tun-address $TUN_BOB --tun-netmask $MASK >>/bob.log 2>&1; echo \$? >/bob.exit"
}
start_dialer() { # container, log name
    docker exec -d "$1" sh -c "RUST_LOG=info cryprq --vpn \
        --peer /ip4/$BOB_IP/udp/$PORT/quic-v1 \
        --tun-address $TUN_ALICE --tun-netmask $MASK >>/$2.log 2>&1; echo \$? >/$2.exit"
}
sessions() { dx 10 "$1" "grep -c event=vpn_session_established /$2.log 2>/dev/null" || true; }
last_session() {
    dx 10 "$1" "grep event=vpn_session_established /$2.log | tail -1" | grep -o 'session_id=[0-9a-f]*'
}
# replies <container> <address> <count>: number of echo replies received
replies() {
    dx 40 "$1" "ping -c $3 -i 0.3 -W 2 -p $PATTERN $2 2>/dev/null" |
        awk '/packets transmitted/ {print $4}'
}
# hex_hits <container> <file>: occurrences of the ping payload in a capture;
# "missing" if there is no capture, so an absent file never counts as clean.
hex_hits() {
    dx 30 "$1" "[ -s $2 ] || { echo missing; exit 0; }; od -An -v -tx1 $2 | tr -d ' \n' | grep -c $PATTERN" || true
}
# all <test>...: every argument is a shell test that must hold
all() {
    local t
    for t in "$@"; do eval "$t" || return 1; done
}

# ---- 1. a device without a session forwards nothing --------------------------

say ""
say "== device without a session"
start_bob
check "the listener creates its TUN device" waitfor 30 "$BOB" 'grep -q event=vpn_tun_ready /bob.log'
dx 10 "$BOB" 'cat /bob.log' >"$E/bob-startup.log" 2>&1
check "the device has the address and netmask it was given" \
    sh -c "timeout 10 docker exec $BOB ip -br addr show cryprq0 | grep -q '$TUN_BOB/30'"
docker exec -d "$BOB" sh -c 'timeout 12 tcpdump -U -ni eth0 -w /nosession-wire.pcap >/dev/null 2>&1'
sleep 2
got="$(replies "$BOB" "$TUN_ALICE" 3)"
check "without a session a ping into the tunnel gets no reply (got ${got:-none})" [ "${got:-0}" = 0 ]
sleep 11
check "and nothing of it reaches the wire" [ "$(hex_hits "$BOB" /nosession-wire.pcap)" = 0 ]

# ---- 2. session and packets through both devices -----------------------------

say ""
say "== authenticated link between two TUN devices"
start_dialer "$ALICE" alice
check "the dialer creates its TUN device" waitfor 30 "$ALICE" 'grep -q event=vpn_tun_ready /alice.log'
check "the dialer establishes a session" \
    waitfor 40 "$ALICE" 'grep -q event=vpn_session_established /alice.log'
check "the listener establishes a session" \
    waitfor 20 "$BOB" 'grep -q event=vpn_session_established /bob.log'
FIRST="$(last_session "$ALICE" alice)"
FIRST_AT_BOB="$(last_session "$BOB" bob)"
check "both ends report the same session ($FIRST)" all '[ -n "$FIRST" ]' '[ "$FIRST" = "$FIRST_AT_BOB" ]'
for c in bob alice; do dx 10 "$P-$c" 'ip -br addr; echo; ip route; echo; ip -d link show cryprq0' >"$E/$c-network.up" 2>&1; done
check "each namespace has exactly one route to the tunnel subnet, through its device" \
    sh -c "[ \"\$(timeout 10 docker exec $BOB ip route | grep -c '^$TUN_SUBNET dev cryprq0')\" = 1 ] &&
           [ \"\$(timeout 10 docker exec $ALICE ip route | grep -c '^$TUN_SUBNET dev cryprq0')\" = 1 ]"
check "no default route appeared" \
    sh -c "! timeout 10 docker exec $BOB ip route | grep -q '^default' && ! timeout 10 docker exec $ALICE ip route | grep -q '^default'"

docker exec -d "$BOB" sh -c 'timeout 25 tcpdump -U -ni eth0 -w /wire.pcap >/dev/null 2>&1'
docker exec -d "$BOB" sh -c 'timeout 25 tcpdump -U -ni cryprq0 -w /tun.pcap >/dev/null 2>&1'
sleep 2
got="$(replies "$ALICE" "$TUN_BOB" 10)"
check "dialer to listener: 10 of 10 echo replies through the tunnel (got ${got:-none})" [ "${got:-0}" = 10 ]
got="$(replies "$BOB" "$TUN_ALICE" 10)"
check "listener to dialer: 10 of 10 echo replies through the tunnel (got ${got:-none})" [ "${got:-0}" = 10 ]
sleep 22

say ""
say "== what is on the wire"
wire_udp="$(dx 30 "$BOB" "tcpdump -nr /wire.pcap 'udp port $PORT' 2>/dev/null | wc -l" | tr -d ' ')"
wire_icmp="$(dx 30 "$BOB" "tcpdump -nr /wire.pcap 'icmp' 2>/dev/null | wc -l" | tr -d ' ')"
wire_other="$(dx 30 "$BOB" "tcpdump -nr /wire.pcap 'not udp port $PORT and not arp' 2>/dev/null | wc -l" | tr -d ' ')"
tun_icmp="$(dx 30 "$BOB" "tcpdump -nr /tun.pcap 'icmp' 2>/dev/null | wc -l" | tr -d ' ')"
check "the wire carries the link's UDP datagrams ($wire_udp captured)" [ "${wire_udp:-0}" -gt 20 ]
check "no ICMP packet is on the wire ($wire_icmp)" [ "${wire_icmp:-1}" = 0 ]
check "nothing but the link's datagrams and ARP is on the wire ($wire_other other packets)" [ "${wire_other:-1}" = 0 ]
check "the ping payload does not occur in the wire capture" [ "$(hex_hits "$BOB" /wire.pcap)" = 0 ]
wire_tunnel="$(dx 30 "$BOB" "tcpdump -nr /wire.pcap 'net $TUN_SUBNET' 2>/dev/null | wc -l" | tr -d ' ')"
check "no packet with a tunnel address is on the wire ($wire_tunnel)" [ "${wire_tunnel:-1}" = 0 ]
check "control: the same search finds the payload in the capture of the TUN device ($tun_icmp ICMP packets there)" \
    all '[ "$(hex_hits "$BOB" /tun.pcap)" != missing ]' '[ "$(hex_hits "$BOB" /tun.pcap)" -ge 1 ]' '[ "${tun_icmp:-0}" -ge 40 ]'

say ""
say "== TCP through the tunnel"
transfer() { # from, to, address of the receiver
    local from="$1" to="$2" addr="$3" a b
    dx 20 "$to" 'rm -f /recv.bin'
    docker exec -d "$to" sh -c "timeout 60 nc -l $addr 5001 >/recv.bin 2>/dev/null"
    sleep 1
    dx 20 "$from" 'head -c 1048576 /dev/urandom >/send.bin'
    dx 60 "$from" "nc -N -w 20 $addr 5001 </send.bin" || return 1
    sleep 1
    a="$(dx 20 "$from" 'sha256sum </send.bin')"
    b="$(dx 20 "$to" 'sha256sum </recv.bin')"
    [ -n "$a" ] && [ "$a" = "$b" ]
}
check "1 MiB over TCP, dialer to listener, arrives with the same SHA-256" transfer "$ALICE" "$BOB" "$TUN_BOB"
check "1 MiB over TCP, listener to dialer, arrives with the same SHA-256" transfer "$BOB" "$ALICE" "$TUN_ALICE"
mtu="$(dx 10 "$BOB" 'cat /sys/class/net/cryprq0/mtu')"
check "the device MTU is 1420 (is ${mtu:-unknown})" [ "${mtu:-0}" = 1420 ]
check "a 1420-byte packet passes unfragmented, a 1421-byte packet is refused locally" \
    sh -c "timeout 20 docker exec $ALICE ping -c 2 -W 2 -M do -s 1392 $TUN_BOB >/dev/null 2>&1 &&
           ! timeout 20 docker exec $ALICE ping -c 2 -W 2 -M do -s 1393 $TUN_BOB >/dev/null 2>&1"

# ---- 3. an identity that is not pinned ---------------------------------------

say ""
say "== unpinned peer"
before_rejects="$(dx 10 "$BOB" 'grep -c event=vpn_rejected /bob.log' || true)"
before_sessions="$(sessions "$BOB" bob)"
start_dialer "$MALLORY" mallory
check "the listener refuses the unpinned peer" \
    waitfor 30 "$BOB" "[ \"\$(grep -c event=vpn_rejected /bob.log)\" -gt ${before_rejects:-0} ]"
sleep 5
check "the unpinned peer gets no session" [ "$(sessions "$MALLORY" mallory)" = 0 ]
check "the listener established no further session" [ "$(sessions "$BOB" bob)" = "$before_sessions" ]
got="$(replies "$MALLORY" "$TUN_BOB" 3)"
check "the unpinned peer's packets get no reply (got ${got:-none})" [ "${got:-0}" = 0 ]
got="$(replies "$ALICE" "$TUN_BOB" 5)"
check "the pinned link still works: 5 of 5 replies (got ${got:-none})" [ "${got:-0}" = 5 ]
check "and is still the same session" [ "$(last_session "$BOB" bob)" = "$FIRST" ]
dx 10 "$MALLORY" 'pkill -TERM -x cryprq' || true

# ---- 4. the peer stops: fail closed ------------------------------------------

say ""
say "== peer stops"
docker exec -d "$ALICE" sh -c 'timeout 14 tcpdump -U -ni eth0 -w /afterstop-wire.pcap >/dev/null 2>&1'
sleep 2
dx 10 "$BOB" 'pkill -TERM -x cryprq'
check "the listener exits with status 0 on SIGTERM" waitfor 15 "$BOB" '[ "$(cat /bob.exit)" = 0 ]'
check "its TUN device is gone" sh -c "! timeout 10 docker exec $BOB ip link show cryprq0 >/dev/null 2>&1"
check "the dialer drops the session" waitfor 20 "$ALICE" 'grep -q event=vpn_session_closed /alice.log'
got="$(replies "$ALICE" "$TUN_BOB" 3)"
check "without the peer a ping into the tunnel gets no reply (got ${got:-none})" [ "${got:-0}" = 0 ]
sleep 13
check "and its payload does not reach the wire" [ "$(hex_hits "$ALICE" /afterstop-wire.pcap)" = 0 ]
# The stopped listener's kernel answers the dialer's reconnection attempts
# with ICMP "port unreachable" on the link addresses. That is expected and is
# not tunnel traffic; an echo packet or a tunnel address would be.
count_in() { dx 30 "$1" "tcpdump -nr $2 '$3' 2>/dev/null | wc -l" | tr -d ' '; }
check "no ICMP echo is on the dialer's wire" \
    [ "$(count_in "$ALICE" /afterstop-wire.pcap 'icmp[icmptype]=icmp-echo or icmp[icmptype]=icmp-echoreply')" = 0 ]
check "no packet with a tunnel address is on the dialer's wire" \
    [ "$(count_in "$ALICE" /afterstop-wire.pcap "net $TUN_SUBNET")" = 0 ]
check "the dialer's wire carries only reconnection attempts, their refusals and ARP" \
    [ "$(count_in "$ALICE" /afterstop-wire.pcap "not udp port $PORT and not arp and not (icmp[icmptype]=icmp-unreach and src host $BOB_IP)")" = 0 ]

# ---- 5. recovery -------------------------------------------------------------

say ""
say "== recovery"
dx 10 "$BOB" 'rm -f /bob.exit'
start_bob
check "the dialer reconnects and both ends establish a new session" \
    waitfor 75 "$ALICE" "[ \"\$(grep -c event=vpn_session_established /alice.log)\" -ge 2 ]"
SECOND="$(last_session "$ALICE" alice)"
waitfor 20 "$BOB" "[ \"\$(grep -c event=vpn_session_established /bob.log)\" -ge 2 ]" || true
SECOND_AT_BOB="$(last_session "$BOB" bob)"
check "the new session differs from the first and both ends agree on it ($SECOND)" \
    all '[ -n "$SECOND" ]' '[ "$SECOND" != "$FIRST" ]' '[ "$SECOND" = "$SECOND_AT_BOB" ]'
got="$(replies "$ALICE" "$TUN_BOB" 5)"
check "dialer to listener after recovery: 5 of 5 (got ${got:-none})" [ "${got:-0}" = 5 ]
got="$(replies "$BOB" "$TUN_ALICE" 5)"
check "listener to dialer after recovery: 5 of 5 (got ${got:-none})" [ "${got:-0}" = 5 ]

# ---- 6. a process that is killed leaves no device ----------------------------

say ""
say "== killed process"
closed_before="$(dx 10 "$BOB" 'grep -c event=vpn_session_closed /bob.log' || true)"
dx 10 "$ALICE" 'pkill -KILL -x cryprq'
check "the killed dialer's TUN device is gone" \
    waitfor 10 "$ALICE" '! ip link show cryprq0 >/dev/null 2>&1'
check "and so is its route to the tunnel subnet" \
    sh -c "! timeout 10 docker exec $ALICE ip route | grep -q '^$TUN_SUBNET'"
check "the listener notices and drops the session" \
    waitfor 60 "$BOB" "[ \"\$(grep -c event=vpn_session_closed /bob.log)\" -gt ${closed_before:-0} ]"
got="$(replies "$BOB" "$TUN_ALICE" 3)"
check "the listener's packets into the tunnel get no reply (got ${got:-none})" [ "${got:-0}" = 0 ]
dx 10 "$BOB" 'pkill -TERM -x cryprq'
check "the listener exits with status 0 on SIGTERM" waitfor 15 "$BOB" '[ "$(cat /bob.exit)" = 0 ]'

# ---- evidence: logs and captures, never keys ---------------------------------

say ""
say "== evidence"
for f in bob.log nosession-wire.pcap wire.pcap tun.pcap; do
    timeout 30 docker cp "$BOB:/$f" "$E/$f" >/dev/null 2>&1 || say "note: $f was not copied"
done
for f in alice.log afterstop-wire.pcap; do
    timeout 30 docker cp "$ALICE:/$f" "$E/$f" >/dev/null 2>&1 || say "note: $f was not copied"
done
timeout 30 docker cp "$MALLORY:/mallory.log" "$E/mallory.log" >/dev/null 2>&1 || say "note: mallory.log was not copied"
for c in bob alice; do dx 10 "$P-$c" 'ip -br addr; echo; ip route' >"$E/$c-network.after" 2>&1; done
check "the namespaces are back to their state before the tunnel" \
    sh -c "diff '$E/bob-network.before' '$E/bob-network.after' >/dev/null && diff '$E/alice-network.before' '$E/alice-network.after' >/dev/null"
check "the logs contain no private key material" \
    sh -c "! grep -liE 'secret|private key|seed=' '$E'/*.log >/dev/null 2>&1"
