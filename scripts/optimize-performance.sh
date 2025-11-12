#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚡ Performance Optimization Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

OPT_LOG="optimization-$(date +%Y%m%d-%H%M%S).log"

# Analysis 1: Check for optimization opportunities
echo "Analysis 1: Code Optimization Opportunities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for common performance issues
echo "Checking for performance anti-patterns..."

# Check for unnecessary clones
CLONE_COUNT=$(grep -r "\.clone()" --include="*.rs" . | grep -v "test" | wc -l || echo "0")
echo "  • Clone operations: $CLONE_COUNT (review for optimization)"

# Check for allocations in hot paths
ALLOC_COUNT=$(grep -r "Vec::new\|String::new\|Box::new" --include="*.rs" . | grep -v "test" | wc -l || echo "0")
echo "  • Allocation operations: $ALLOC_COUNT (consider pre-allocation)"

# Check for async/await usage
ASYNC_COUNT=$(grep -r "async\|await" --include="*.rs" . | grep -v "test" | wc -l || echo "0")
echo "  • Async operations: $ASYNC_COUNT (verify efficient usage)"
echo ""

# Analysis 2: Build optimizations
echo "Analysis 2: Build Optimizations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check Cargo.toml for optimization settings
if grep -q "opt-level = \"3\"" Cargo.toml 2>/dev/null || grep -q "opt-level = 3" Cargo.toml 2>/dev/null; then
    echo "✅ Maximum optimization enabled (opt-level = 3)"
else
    echo "⚠️  Consider enabling opt-level = 3 for release builds"
fi

# Check for LTO
if grep -q "lto = true" Cargo.toml 2>/dev/null; then
    echo "✅ Link-time optimization (LTO) enabled"
else
    echo "⚠️  Consider enabling LTO for smaller binaries"
fi

# Check for codegen-units
if grep -q "codegen-units = 1" Cargo.toml 2>/dev/null; then
    echo "✅ Single codegen unit (better optimization)"
else
    echo "⚠️  Consider codegen-units = 1 for better optimization"
fi
echo ""

# Analysis 3: Memory optimizations
echo "Analysis 3: Memory Optimizations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check binary size
BINARY_SIZE=$(stat -f%z target/release/cryprq 2>/dev/null || stat -c%s target/release/cryprq 2>/dev/null || echo "0")
if [ "$BINARY_SIZE" -lt 15000000 ]; then
    echo "✅ Binary size optimized: ${BINARY_SIZE} bytes"
else
    echo "⚠️  Binary size large: ${BINARY_SIZE} bytes"
    echo "   Consider: strip symbols, enable LTO, reduce dependencies"
fi
echo ""

# Analysis 4: Runtime optimizations
echo "Analysis 4: Runtime Performance"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for profiling data
if [ -f "perf.data" ] || [ -f "flamegraph.svg" ]; then
    echo "✅ Profiling data available"
else
    echo "ℹ️  Run profiling for detailed analysis:"
    echo "   cargo install flamegraph"
    echo "   cargo flamegraph --bin cryprq"
fi
echo ""

# Recommendations
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💡 Optimization Recommendations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Enable LTO in Cargo.toml:"
echo "   [profile.release]"
echo "   lto = true"
echo "   codegen-units = 1"
echo ""
echo "2. Use cargo bench for detailed benchmarks:"
echo "   cargo +nightly bench"
echo ""
echo "3. Profile with flamegraph:"
echo "   cargo install flamegraph"
echo "   cargo flamegraph --bin cryprq"
echo ""
echo "4. Check for memory leaks:"
echo "   valgrind --leak-check=full ./target/release/cryprq"
echo ""
echo "📊 Optimization log: $OPT_LOG"
echo "✅ Performance optimization analysis complete"

