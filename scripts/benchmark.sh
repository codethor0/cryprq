#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Benchmark script for CrypRQ
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚡ Running Benchmarks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ ERROR: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Check for nightly toolchain (for benchmarks)
if ! rustup toolchain list | grep -q nightly; then
    echo "⚠️  Nightly toolchain not found. Installing..."
    rustup toolchain install nightly
fi

# Run benchmarks if available
if [ -d "benches" ] || grep -q "\[\[bench\]\]" Cargo.toml 2>/dev/null; then
    echo "📊 Running Criterion benchmarks..."
    cargo +nightly bench 2>&1 | tee benchmark-results.log
else
    echo "⚠️  No benchmarks found. Creating benchmark infrastructure..."
    
    # Create benches directory
    mkdir -p benches
    
    # Create basic benchmark
    cat > benches/handshake_bench.rs << 'EOF'
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cryprq_crypto::HybridHandshake;

fn bench_handshake(c: &mut Criterion) {
    c.bench_function("hybrid_handshake", |b| {
        b.iter(|| {
            let (alice_pk, alice_sk) = cryprq_crypto::kyber_keypair();
            let (bob_pk, bob_sk) = cryprq_crypto::kyber_keypair();
            
            // Simulate handshake
            let _shared = HybridHandshake::client_handshake(
                black_box(&alice_pk),
                black_box(&bob_sk),
            );
        });
    });
}

criterion_group!(benches, bench_handshake);
criterion_main!(benches);
EOF
    
    # Add criterion to Cargo.toml if not present
    if ! grep -q "criterion" Cargo.toml; then
        echo ""
        echo "[dev-dependencies]" >> Cargo.toml
        echo "criterion = { version = \"0.5\", features = [\"html_reports\"] }" >> Cargo.toml
        echo "" >> Cargo.toml
        echo "[[bench]]" >> Cargo.toml
        echo "name = \"handshake_bench\"" >> Cargo.toml
        echo "harness = false" >> Cargo.toml
    fi
    
    echo "✅ Benchmark infrastructure created"
    echo "📊 Running benchmarks..."
    cargo +nightly bench 2>&1 | tee benchmark-results.log
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Benchmarks completed"
echo "📊 Results: benchmark-results.log"
echo ""

exit 0

