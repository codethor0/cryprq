# CrypRQ Extreme Verify → Optimize → Lock - Implementation Roadmap

**Date**: 2025-11-12  
**Status**: Phase 1 Complete  | Phase 2 In Progress

## Phase 1: Known Issues Fixed 

- [x] Fix KAT API order
- [x] Restore full property tests
- [x] Add Vec import for no_std
- [x] Create KAT loader infrastructure

## Phase 2: Extended Testing (In Progress)

### 2.1 Extended Fuzz Runs
**Status**: Infrastructure Ready  
**Action Required**:
- Run each fuzz target for 30+ minutes
- Targets: `hybrid_handshake`, `protocol_parse`, `key_rotation`, `ppk_derivation`
- Command: `cargo fuzz run <target> -- -max_total_time=1800`
- Save corpus and crashes (should be empty)

**Implementation**:
```bash
for target in hybrid_handshake protocol_parse key_rotation ppk_derivation; do
  cargo fuzz run $target -- -max_total_time=1800 -artifact_prefix=release-20251112/qa/fuzz/
done
```

### 2.2 Full Miri Sweep
**Status**: Quick test passed  
**Action Required**:
- Run `cargo +nightly miri test --all`
- Fix any UB found
- Store logs in artifacts

**Implementation**:
```bash
rustup +nightly component add miri
cargo +nightly miri test --all 2>&1 | tee release-20251112/qa/miri-full.log
```

### 2.3 QUIC/libp2p Interop
**Status**: Not yet integrated  
**Action Required**:
- Add QUIC interop runner Docker endpoint
- Add libp2p test-plans smoke tests
- Archive interop logs

**Implementation**:
- Create `scripts/quic-interop-test.sh`
- Create `scripts/libp2p-interop-test.sh`
- Add Docker endpoint for QUIC interop runner

### 2.4 Criterion Benchmarks
**Status**: Not yet added  
**Action Required**:
- Add Criterion benches for handshake latency, rotation overhead, packets/s
- Generate flamegraphs
- Set performance SLOs

**Implementation**:
- Create `benches/` directory
- Add `handshake_bench.rs`, `rotation_bench.rs`, `throughput_bench.rs`
- Integrate into CI

### 2.5 CI Gate Integration
**Status**: Patch ready  
**Action Required**:
- Apply `ci-gates.patch` to `.github/workflows/ci.yml`
- Add all gates as hard failures
- Upload artifacts

**Implementation**:
- Apply patch from `release-20251112/qa/ci-gates.patch`
- Test CI gates locally
- Verify artifact uploads

## Phase 3: Supply Chain & Reproducibility

### 3.1 SBOM & Grype
**Status**:  Complete
- SBOM generated
- Grype scan complete

### 3.2 Reproducible Builds
**Status**:  Verified
- Checksums match
- Deterministic builds confirmed

## Exit Criteria

- [x] All KATs pass (byte-exact)
- [ ] Fuzz: each target ≥30 min, 0 crashes
- [ ] Miri + Sanitizers: clean
- [ ] Interop: QUIC & libp2p smoke pass
- [ ] Perf: meets/exceeds baselines
- [ ] Reproducible builds: checksums match
- [ ] CI gates: all enforced

---

**Current Status**: Phase 1 Complete  | Phase 2 In Progress

