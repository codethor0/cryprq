// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

// Criterion benchmarks for handshake latency

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cryprq_crypto::HybridHandshake;

fn bench_handshake_creation(c: &mut Criterion) {
    c.bench_function("hybrid_handshake_creation", |b| {
        b.iter(|| {
            black_box(HybridHandshake::new());
        });
    });
}

fn bench_keypair_generation(c: &mut Criterion) {
    use pqcrypto_mlkem::mlkem768::keypair;
    
    c.bench_function("kyber768_keypair", |b| {
        b.iter(|| {
            black_box(keypair());
        });
    });
}

fn bench_encaps_decaps(c: &mut Criterion) {
    use pqcrypto_mlkem::mlkem768::{keypair, encapsulate, decapsulate};
    
    let (pk, sk) = keypair();
    
    c.bench_function("kyber768_encaps", |b| {
        b.iter(|| {
            black_box(encapsulate(&pk));
        });
    });
    
    let (_, ct) = encapsulate(&pk);
    c.bench_function("kyber768_decaps", |b| {
        b.iter(|| {
            black_box(decapsulate(&ct, &sk));
        });
    });
}

criterion_group!(benches, bench_handshake_creation, bench_keypair_generation, bench_encaps_decaps);
criterion_main!(benches);

