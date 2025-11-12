#!/bin/bash
# Optimization Tracker - Monitors and adapts optimization strategies

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Optimization Strategy Tracker"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Date: $(date)"
echo ""

# Track optimization metrics
mkdir -p optimization-tracking

# Current binary size
if [ -f "target/release/cryprq" ]; then
    CURRENT_SIZE=$(stat -f%z target/release/cryprq 2>/dev/null || stat -c%s target/release/cryprq 2>/dev/null)
    echo "Current Binary Size: $((CURRENT_SIZE / 1024 / 1024))MB"
    
    # Compare with baseline
    if [ -f "optimization-tracking/baseline-size.txt" ]; then
        BASELINE=$(cat optimization-tracking/baseline-size.txt)
        DIFF=$((CURRENT_SIZE - BASELINE))
        if [ $DIFF -gt 0 ]; then
            echo "⚠️ Binary size increased by $((DIFF / 1024))KB"
        elif [ $DIFF -lt 0 ]; then
            echo "✅ Binary size decreased by $(((-DIFF) / 1024))KB"
        else
            echo "✅ Binary size unchanged"
        fi
    else
        echo "$CURRENT_SIZE" > optimization-tracking/baseline-size.txt
        echo "✅ Baseline established"
    fi
fi

# Build optimization status
echo ""
echo "Build Optimization Status:"
grep -q "opt-level = 3" Cargo.toml && echo "✅ Maximum optimization" || echo "⚠️ Not at maximum"
grep -q "lto = true" Cargo.toml && echo "✅ LTO enabled" || echo "⚠️ LTO disabled"
grep -q "codegen-units = 1" Cargo.toml && echo "✅ Single codegen unit" || echo "⚠️ Multiple codegen units"

# Recommendations
echo ""
echo "Optimization Recommendations:"
if ! grep -q "opt-level = 3" Cargo.toml; then
    echo "  • Enable opt-level = 3 for maximum optimization"
fi
if ! grep -q "lto = true" Cargo.toml; then
    echo "  • Enable LTO for smaller binaries"
fi

echo ""
echo "✅ Optimization tracking complete"
