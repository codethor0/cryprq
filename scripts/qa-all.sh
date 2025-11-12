#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Comprehensive QA pipeline orchestrator
set -euo pipefail

DATE=$(date +%Y%m%d)
ARTIFACT_DIR="release-${DATE}/qa"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CRYPRQ EXTREME VALIDATION & OPTIMIZATION - Full Pipeline"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Artifacts: $ARTIFACT_DIR"
echo ""

FAILED_STEPS=()

# 1. Environment setup
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Environment Setup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/setup-qa-env.sh 2>&1 | tee "$ARTIFACT_DIR/env-setup.log"; then
    echo "✅ Environment setup complete"
else
    echo "❌ Environment setup failed"
    FAILED_STEPS+=("env-setup")
fi
echo ""

# 2. Format & Clippy
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 2: Format & Clippy"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo fmt --all -- --check 2>&1 | tee "$ARTIFACT_DIR/fmt.log"; then
    echo "✅ Format check passed"
else
    echo "❌ Format check failed"
    FAILED_STEPS+=("fmt")
fi
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee "$ARTIFACT_DIR/clippy.log"; then
    echo "✅ Clippy check passed"
else
    echo "❌ Clippy check failed"
    FAILED_STEPS+=("clippy")
fi
echo ""

# 3. Unit tests
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

# 4. KAT tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 4: KAT (Known Answer Tests)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-kat-tests.sh 2>&1 | tee "$ARTIFACT_DIR/kat-summary.log"; then
    echo "✅ KAT tests passed"
else
    echo "❌ KAT tests failed"
    FAILED_STEPS+=("kat")
fi
echo ""

# 5. Property tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 5: Property Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo test --package cryprq-crypto property_tests 2>&1 | tee "$ARTIFACT_DIR/property-tests.log"; then
    echo "✅ Property tests passed"
else
    echo "❌ Property tests failed"
    FAILED_STEPS+=("property")
fi
echo ""

# 6. Fuzz smoke tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 6: Fuzz Smoke Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-extended-fuzz.sh hybrid_handshake 60 2>&1 | tee "$ARTIFACT_DIR/fuzz-smoke.log"; then
    echo "✅ Fuzz smoke tests passed"
else
    echo "❌ Fuzz smoke tests failed"
    FAILED_STEPS+=("fuzz-smoke")
fi
echo ""

# 7. Miri sweep
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 7: Miri UB Detection"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-miri-sweep.sh 2>&1 | tee "$ARTIFACT_DIR/miri.log"; then
    echo "✅ Miri sweep passed"
else
    echo "❌ Miri sweep failed"
    FAILED_STEPS+=("miri")
fi
echo ""

# 8. Sanitizers
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 8: Sanitizers (ASan/UBSan)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-sanitizers.sh 2>&1 | tee "$ARTIFACT_DIR/sanitizers-summary.log"; then
    echo "✅ Sanitizer tests passed"
else
    echo "❌ Sanitizer tests failed"
    FAILED_STEPS+=("sanitizers")
fi
echo ""

# 9. Coverage
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 9: Coverage Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-coverage.sh 2>&1 | tee "$ARTIFACT_DIR/coverage-summary.log"; then
    echo "✅ Coverage analysis complete"
else
    echo "❌ Coverage analysis failed"
    FAILED_STEPS+=("coverage")
fi
echo ""

# 10. Supply chain
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 10: Supply Chain Security"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-supply-chain.sh 2>&1 | tee "$ARTIFACT_DIR/supply-chain-summary.log"; then
    echo "✅ Supply chain checks passed"
else
    echo "❌ Supply chain checks failed"
    FAILED_STEPS+=("supply-chain")
fi
echo ""

# 11. Reproducible builds
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 11: Reproducible Builds"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-reproducible-build.sh 2>&1 | tee "$ARTIFACT_DIR/reproducible-summary.log"; then
    echo "✅ Reproducible build verification passed"
else
    echo "❌ Reproducible build verification failed"
    FAILED_STEPS+=("reproducible")
fi
echo ""

# 12. Performance benchmarks
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 12: Performance Benchmarks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash scripts/run-performance-regression.sh 2>&1 | tee "$ARTIFACT_DIR/bench-summary.log"; then
    echo "✅ Performance benchmarks passed"
else
    echo "❌ Performance benchmarks failed"
    FAILED_STEPS+=("bench")
fi
echo ""

# Final summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Pipeline Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ ${#FAILED_STEPS[@]} -eq 0 ]; then
    echo "✅ All QA steps passed!"
    echo ""
    echo "Artifacts: $ARTIFACT_DIR"
    exit 0
else
    echo "❌ Failed steps: ${FAILED_STEPS[*]}"
    echo ""
    echo "Check logs in: $ARTIFACT_DIR"
    exit 1
fi
