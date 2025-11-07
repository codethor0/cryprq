#!/usr/bin/env bash
set -euo pipefail
IMG=${IMG:-cryprq-node}
NET=${NET:-vpn-test}

# ---------- 1  network  ---------------------------------------------
echo "== 1.  network ======================================================"
docker network rm "$NET" 2>/dev/null || true
docker network create "$NET" || {
  echo "WARN: network create failed once, retrying after prune"
  docker network prune -f
  docker network create "$NET"
}

# ---------- 2  vpn1 (listener)  -------------------------------------
echo "== 2.  start vpn1 (listener) ========================================"
ID1=$(docker run -d --rm --name vpn1 --network "$NET" \
                 --entrypoint /usr/local/bin/cryprq "$IMG" \
                 --peer /ip4/0.0.0.0/udp/9999/quic-v1/p2p/self)
sleep 3
PEER1=$(docker logs vpn1 2>&1 | awk '/Local identity:/{print $3}')
echo "vpn1 peer-id : $PEER1"

# ---------- 3  vpn2 (dialer)  ----------------------------------------
echo "== 3.  start vpn2 (dialer) =========================================="
ID2=$(docker run -d --rm --name vpn2 --network "$NET" \
                 --entrypoint /usr/local/bin/cryprq "$IMG" \
                 --peer /ip4/vpn1/udp/9999/quic-v1/p2p/$PEER1)
sleep 3
PEER2=$(docker logs vpn2 2>&1 | awk '/Local identity:/{print $3}')
echo "vpn2 peer-id : $PEER2"

# ---------- 4  handshake check  --------------------------------------
echo "== 4.  wait for PQ handshake ========================================"
sleep 5
if docker logs vpn1 2>&1 | grep -q "handshake complete"; then
  echo "PASS: vpn1 handshake OK"
else
  echo "FAIL: vpn1 handshake missing"; exit 1
fi

if docker logs vpn2 2>&1 | grep -q "handshake complete"; then
  echo "PASS: vpn2 handshake OK"
else
  echo "FAIL: vpn2 handshake missing"; exit 1
fi

# ---------- 5  teardown  ---------------------------------------------
echo "== 5.  teardown ====================================================="
docker rm -f vpn1 vpn2
docker network rm "$NET" 2>/dev/null || true
echo "PQ-VPN pair-test passed"
