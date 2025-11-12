// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

// Criterion benchmarks for key rotation overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cryprq_crypto::PostQuantumPSK;

fn bench_ppk_derivation(c: &mut Criterion) {
    use cryprq_crypto::PPKStore;
    
    let mut store = PPKStore::new();
    let peer_id = [1u8; 32];
    
    c.bench_function("ppk_derivation", |b| {
        b.iter(|| {
            black_box(store.derive_ppk(&peer_id, 300));
        });
    });
}

fn bench_rotation_overhead(c: &mut Criterion) {
    use cryprq_crypto::PPKStore;
    
    let mut store = PPKStore::new();
    let peer_id = [1u8; 32];
    
    c.bench_function("rotation_overhead", |b| {
        b.iter(|| {
            let _ppk = store.derive_ppk(&peer_id, 300);
            store.rotate_if_needed(300);
        });
    });
}

criterion_group!(benches, bench_ppk_derivation, bench_rotation_overhead);
criterion_main!(benches);

