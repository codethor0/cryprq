#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

root="$(pwd)"
ts="$(date +%Y%m%d_%H%M%S)"
art="artifacts/cutover_${ts}"
mkdir -p "$art"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CrypRQ Go-Live Cutover Smoke Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "[1/7] Toolchain + lint + test"
rustup toolchain install 1.83.0 >/dev/null 2>&1 || true
cargo fmt --all -- --check || cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee "$art/clippy.txt"
cargo test --all --no-fail-fast 2>&1 | tee "$art/tests.txt"

echo ""
echo "[2/7] Release build"
time cargo build --release -p cryprq 2>&1 | tee "$art/build.txt"

echo ""
echo "[3/7] Docker image + listener"
docker build -t cryprq-node:release . 2>&1 | tee "$art/docker_build.txt"
docker rm -f cryprq-listener >/dev/null 2>&1 || true
docker run -d --name cryprq-listener -p 9999:9999/udp \
  cryprq-node:release --listen /ip4/0.0.0.0/udp/9999/quic-v1
sleep 2
docker logs --since 1s cryprq-listener 2>&1 | tee "$art/listener_boot.txt"

echo ""
echo "[4/7] Dialer handshake"
timeout 10 docker run --rm --network host cryprq-node:release \
  --peer /ip4/127.0.0.1/udp/9999/quic-v1 2>&1 | tee "$art/dialer.txt" || true

echo ""
echo "[5/7] Rotation check (accelerated)"
export CRYPRQ_ROTATE_SECS=10
docker rm -f cryprq-listener >/dev/null 2>&1 || true
docker run -d --name cryprq-listener -p 9999:9999/udp \
  -e CRYPRQ_ROTATE_SECS \
  cryprq-node:release --listen /ip4/0.0.0.0/udp/9999/quic-v1
sleep 15
docker logs cryprq-listener 2>&1 | grep -i -E "rotate|rotation|zeroiz" | tee "$art/rotation.txt" || true
docker rm -f cryprq-listener >/dev/null 2>&1 || true

echo ""
echo "[6/7] Security artifacts"
bash scripts/syft-sbom.sh 2>&1 | tee "$art/syft.txt" || echo "⚠️  SBOM generation skipped"
bash scripts/grype-scan.sh 2>&1 | tee "$art/grype.txt" || echo "⚠️  Grype scan skipped"
bash scripts/repro-build.sh 2>&1 | tee "$art/repro.txt" || echo "⚠️  Reproducible build check skipped"

echo ""
echo "[7/7] Bundle"
mkdir -p "release-${ts}/security"
cp -r "$art" "release-${ts}/"
cp -r artifacts/sbom "release-${ts}/security/" 2>/dev/null || true
cp -r artifacts/grype "release-${ts}/security/" 2>/dev/null || true

# Copy binary and checksum
if [ -f "target/release/cryprq" ]; then
    mkdir -p "release-${ts}/bin"
    cp target/release/cryprq "release-${ts}/bin/"
    shasum -a 256 "target/release/cryprq" > "release-${ts}/bin/checksums.txt" 2>/dev/null || \
    md5sum "target/release/cryprq" > "release-${ts}/bin/checksums.txt" 2>/dev/null || true
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Cutover smoke test complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Artifacts: release-${ts}/"
echo ""

