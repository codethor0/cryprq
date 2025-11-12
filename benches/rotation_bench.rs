// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

// Criterion benchmarks for key rotation overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_ppk_derivation(c: &mut Criterion) {
    use cryprq_crypto::PostQuantumPSK;

    let peer_id = [1u8; 32];
    let kyber_shared = [1u8; 32];
    let salt = [2u8; 16];
    let current_time = 1000u64;

    c.bench_function("ppk_derivation", |b| {
        b.iter(|| {
            black_box(PostQuantumPSK::derive(
                &kyber_shared,
                &peer_id,
                &salt,
                300,
                current_time,
            ));
        });
    });
}

fn bench_rotation_overhead(c: &mut Criterion) {
    use cryprq_crypto::{PPKStore, PostQuantumPSK};

    let mut store = PPKStore::new();
    let peer_id = [1u8; 32];
    let kyber_shared = [1u8; 32];
    let salt = [2u8; 16];
    let mut current_time = 1000u64;

    c.bench_function("rotation_overhead", |b| {
        b.iter(|| {
            let ppk = PostQuantumPSK::derive(&kyber_shared, &peer_id, &salt, 300, current_time);
            store.store(ppk);
            store.cleanup_expired(current_time);
            current_time += 1;
            black_box(store.get(&peer_id, current_time));
        });
    });
}

criterion_group!(benches, bench_ppk_derivation, bench_rotation_overhead);
criterion_main!(benches);
