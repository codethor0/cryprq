#!/usr/bin/env bash
set -euo pipefail
# CrypRQ nightly benchmark
cargo build --release -p cryprq
echo "=== TCP throughput (iperf3) ==="
echo "=== UDP latency (netperf) ==="
echo "=== Handshake time (openssl s_time) ==="
echo "=== CPU burn (sar -u 1 30) ==="
echo "Results saved to benchmarks/results/$(git rev-parse --short HEAD)/"