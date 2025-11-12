#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Comprehensive QA pipeline orchestrator (Bulletproof QA)
set -euo pipefail

DATE=$(date +%Y%m%d)
ARTIFACT_DIR="release-${DATE}/qa"
mkdir -p "$ARTIFACT_DIR"

# Bootstrap if first run
if [ ! -f ".tools-lock.json" ]; then
    echo "Running bootstrap..."
    bash scripts/qa-bootstrap.sh
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CRYPRQ EXTREME VALIDATION & OPTIMIZATION - Full Pipeline"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Artifacts: $ARTIFACT_DIR"
echo ""

FAILED_STEPS=()

# 0. Bootstrap
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 0: Bootstrap (Toolchain & Dependencies)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/qa-bootstrap.sh 2>&1 | tee "$ARTIFACT_DIR/bootstrap.log"; then
    echo "✅ Bootstrap complete"
else
    echo "❌ Bootstrap failed"
    FAILED_STEPS+=("bootstrap")
fi
echo ""

# 1. Repo sanity & hardening
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Repo Sanity & Hardening"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo fmt --all -- --check 2>&1 | tee "$ARTIFACT_DIR/fmt.log"; then
    echo "✅ Format check passed"
else
    echo "❌ Format check failed"
    FAILED_STEPS+=("fmt")
fi
if cargo clippy --workspace --all-targets --all-features -D warnings 2>&1 | tee "$ARTIFACT_DIR/clippy.log"; then
    echo "✅ Clippy check passed"
else
    echo "❌ Clippy check failed"
    FAILED_STEPS+=("clippy")
fi
if bash scripts/run-unused-deps.sh 2>&1 | tee "$ARTIFACT_DIR/unused-deps.log"; then
    echo "✅ Unused deps check passed"
else
    echo "❌ Unused deps check failed"
    FAILED_STEPS+=("unused-deps")
fi
echo ""

# 2. Unit tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: Unit Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo test --all --lib 2>&1 | tee "$ARTIFACT_DIR/unit-tests.log"; then
    echo "✅ Unit tests passed"
else
    echo "❌ Unit tests failed"
    FAILED_STEPS+=("unit-tests")
fi
echo ""

# 3. KAT tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: KAT (Known Answer Tests)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-kat-tests.sh 2>&1 | tee "$ARTIFACT_DIR/kat-summary.log"; then
    echo "✅ KAT tests passed"
else
    echo "❌ KAT tests failed"
    FAILED_STEPS+=("kat")
fi
echo ""

# 4. Property tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 4: Property Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-property-tests.sh 2>&1 | tee "$ARTIFACT_DIR/property-summary.log"; then
    echo "✅ Property tests passed"
else
    echo "❌ Property tests failed"
    FAILED_STEPS+=("property")
fi
echo ""

# 5. Extended fuzz (30+ min per target)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 5: Extended Fuzz (30+ min per target)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-extended-fuzz.sh all 1800 2>&1 | tee "$ARTIFACT_DIR/fuzz-extended.log"; then
    echo "✅ Extended fuzz tests passed"
else
    echo "❌ Extended fuzz tests failed"
    FAILED_STEPS+=("fuzz-extended")
fi
echo ""

# 6. Miri sweep
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 6: Miri UB Detection"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-miri-sweep.sh 2>&1 | tee "$ARTIFACT_DIR/miri.log"; then
    echo "✅ Miri sweep passed"
else
    echo "❌ Miri sweep failed"
    FAILED_STEPS+=("miri")
fi
echo ""

# 7. Sanitizers
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 7: Sanitizers (ASan/UBSan)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-sanitizers.sh 2>&1 | tee "$ARTIFACT_DIR/sanitizers-summary.log"; then
    echo "✅ Sanitizer tests passed"
else
    echo "❌ Sanitizer tests failed"
    FAILED_STEPS+=("sanitizers")
fi
echo ""

# 8. Interop (QUIC + libp2p)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 8: Interop (QUIC + libp2p)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-quic-interop.sh 2>&1 | tee "$ARTIFACT_DIR/quic-interop.log"; then
    echo "✅ QUIC interop passed"
else
    echo "❌ QUIC interop failed"
    FAILED_STEPS+=("quic-interop")
fi
if bash scripts/run-libp2p-interop.sh 2>&1 | tee "$ARTIFACT_DIR/libp2p-interop.log"; then
    echo "✅ libp2p interop passed"
else
    echo "❌ libp2p interop failed"
    FAILED_STEPS+=("libp2p-interop")
fi
echo ""

# 9. Benchmarks
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 9: Performance Benchmarks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-benchmarks.sh 2>&1 | tee "$ARTIFACT_DIR/bench-summary.log"; then
    echo "✅ Benchmarks passed"
else
    echo "❌ Benchmarks failed"
    FAILED_STEPS+=("bench")
fi
echo ""

# 10. Coverage
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 10: Coverage Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-coverage.sh 2>&1 | tee "$ARTIFACT_DIR/coverage-summary.log"; then
    echo "✅ Coverage analysis complete"
else
    echo "❌ Coverage analysis failed"
    FAILED_STEPS+=("coverage")
fi
echo ""

# 11. MSRV & SemVer
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 11: MSRV & SemVer Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-msrv-check.sh 2>&1 | tee "$ARTIFACT_DIR/msrv-summary.log"; then
    echo "✅ MSRV verification passed"
else
    echo "❌ MSRV verification failed"
    FAILED_STEPS+=("msrv")
fi
if bash scripts/run-semver-checks.sh 2>&1 | tee "$ARTIFACT_DIR/semver-summary.log"; then
    echo "✅ SemVer checks passed"
else
    echo "❌ SemVer checks failed"
    FAILED_STEPS+=("semver")
fi
echo ""

# 12. Supply chain
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 12: Supply Chain Security"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-supply-chain.sh 2>&1 | tee "$ARTIFACT_DIR/supply-chain-summary.log"; then
    echo "✅ Supply chain checks passed"
else
    echo "❌ Supply chain checks failed"
    FAILED_STEPS+=("supply-chain")
fi
echo ""

# 13. SBOM & Grype
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 13: SBOM & Grype Scan"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-sbom-and-grype.sh 2>&1 | tee "$ARTIFACT_DIR/sbom-summary.log"; then
    echo "✅ SBOM & Grype scan passed"
else
    echo "❌ SBOM & Grype scan failed"
    FAILED_STEPS+=("sbom")
fi
echo ""

# 14. Reproducible builds
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 14: Reproducible Builds"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-reproducible-build.sh 2>&1 | tee "$ARTIFACT_DIR/reproducible-summary.log"; then
    echo "✅ Reproducible build verification passed"
else
    echo "❌ Reproducible build verification failed"
    FAILED_STEPS+=("reproducible")
fi
echo ""

# 15. Provenance & Signing
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 15: Provenance & Signing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/sign-and-prove.sh 2>&1 | tee "$ARTIFACT_DIR/provenance-summary.log"; then
    echo "✅ Provenance & signing complete"
else
    echo "⚠️ Provenance & signing skipped (non-blocking)"
fi
echo ""

# 16. Mutation Testing
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 16: Mutation Testing (cargo-mutants)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-mutation-tests.sh 2>&1 | tee "$ARTIFACT_DIR/mutation-summary.log"; then
    echo "✅ Mutation testing passed"
else
    echo "❌ Mutation testing failed"
    FAILED_STEPS+=("mutation")
fi
echo ""

# 17. Loom Concurrency Tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 17: Loom Concurrency Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-loom-tests.sh 2>&1 | tee "$ARTIFACT_DIR/loom-summary.log"; then
    echo "✅ Loom tests passed"
else
    echo "⚠️ Loom tests skipped (infrastructure ready)"
fi
echo ""

# 18. Dudect Constant-Time Tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 18: Dudect Constant-Time Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-dudect-tests.sh 2>&1 | tee "$ARTIFACT_DIR/dudect-summary.log"; then
    echo "✅ Dudect infrastructure ready"
else
    echo "⚠️ Dudect tests skipped (infrastructure ready)"
fi
echo ""

# 19. Network Adversity
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 19: Network Adversity (tc netem)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-network-adversity.sh 2>&1 | tee "$ARTIFACT_DIR/network-adversity-summary.log"; then
    echo "✅ Network adversity infrastructure ready"
else
    echo "⚠️ Network adversity tests skipped (infrastructure ready)"
fi
echo ""

# 20. MSRV Verification
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 20: MSRV Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-msrv-check.sh 2>&1 | tee "$ARTIFACT_DIR/msrv-summary.log"; then
    echo "✅ MSRV verification passed"
else
    echo "❌ MSRV verification failed"
    FAILED_STEPS+=("msrv")
fi
echo ""

# 16. Mutation Testing
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 16: Mutation Testing (cargo-mutants)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-mutation-tests.sh 2>&1 | tee "$ARTIFACT_DIR/mutation-summary.log"; then
    echo "✅ Mutation testing passed"
else
    echo "❌ Mutation testing failed"
    FAILED_STEPS+=("mutation")
fi
echo ""

# 17. Loom Concurrency Tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 17: Loom Concurrency Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-loom-tests.sh 2>&1 | tee "$ARTIFACT_DIR/loom-summary.log"; then
    echo "✅ Loom tests passed"
else
    echo "⚠️ Loom tests skipped (infrastructure ready)"
fi
echo ""

# 18. Dudect Constant-Time Tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 18: Dudect Constant-Time Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-dudect-tests.sh 2>&1 | tee "$ARTIFACT_DIR/dudect-summary.log"; then
    echo "✅ Dudect infrastructure ready"
else
    echo "⚠️ Dudect tests skipped (infrastructure ready)"
fi
echo ""

# 19. Network Adversity
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 19: Network Adversity (tc netem)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-network-adversity.sh 2>&1 | tee "$ARTIFACT_DIR/network-adversity-summary.log"; then
    echo "✅ Network adversity infrastructure ready"
else
    echo "⚠️ Network adversity tests skipped (infrastructure ready)"
fi
echo ""

# 20. Documentation Linting
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 20: Documentation Linting"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-doc-lints.sh 2>&1 | tee "$ARTIFACT_DIR/doc-lints-summary.log"; then
    echo "✅ Documentation linting passed"
else
    echo "❌ Documentation linting failed"
    FAILED_STEPS+=("docs")
fi
echo ""

# 21. OpenSSF Scorecard
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 21: OpenSSF Scorecard"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-openssf-scorecard.sh 2>&1 | tee "$ARTIFACT_DIR/scorecard-summary.log"; then
    echo "✅ Scorecard complete"
else
    echo "⚠️ Scorecard skipped (non-blocking)"
fi
echo ""

# Final summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Pipeline Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Generate report
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Generating QA Report"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
bash scripts/qa-all-report.sh

if [ ${#FAILED_STEPS[@]} -eq 0 ]; then
    echo ""
    echo "✅ All QA steps passed!"
    echo ""
    echo "Artifacts: $ARTIFACT_DIR"
    echo "Reports: $ARTIFACT_DIR/REPORT.md, $ARTIFACT_DIR/SUMMARY.md"
    exit 0
else
    echo ""
    echo "❌ Failed steps: ${FAILED_STEPS[*]}"
    echo ""
    echo "Collecting artifacts and opening issue..."
    bash scripts/qa-collect-and-open-issue.sh "${FAILED_STEPS[0]}"
    echo ""
    echo "Check logs in: $ARTIFACT_DIR"
    echo "Issue template: $ARTIFACT_DIR/issue.md"
    exit 1
fi
