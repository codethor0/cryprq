#!/bin/bash
# Automated Performance Monitoring Script
# Tracks binary size, build time, test time, and startup time

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Performance Monitoring Report"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Date: $(date)"
echo ""

# Create monitoring directory
mkdir -p monitoring

# Binary size
if [ -f "target/release/cryprq" ]; then
    SIZE=$(stat -f%z target/release/cryprq 2>/dev/null || stat -c%s target/release/cryprq 2>/dev/null)
    SIZE_MB=$((SIZE / 1024 / 1024))
    echo "Binary Size: ${SIZE_MB}MB (${SIZE} bytes)"
    echo "${SIZE}" > monitoring/binary-size.txt
else
    echo "⚠️ Binary not found - building..."
    cargo build --release -p cryprq
fi

# Build time
echo ""
echo "Measuring build time..."
START=$(date +%s)
cargo build --release -p cryprq > /dev/null 2>&1
END=$(date +%s)
BUILD_TIME=$((END - START))
echo "Build Time: ${BUILD_TIME}s"
echo "${BUILD_TIME}" > monitoring/build-time.txt

# Test time
echo ""
echo "Measuring test time..."
START=$(date +%s)
cargo test --lib --all --no-fail-fast > /dev/null 2>&1
END=$(date +%s)
TEST_TIME=$((END - START))
echo "Test Time: ${TEST_TIME}s"
echo "${TEST_TIME}" > monitoring/test-time.txt

# Startup time
echo ""
echo "Measuring startup time..."
START=$(date +%s%N)
timeout 1 target/release/cryprq --help > /dev/null 2>&1 || true
END=$(date +%s%N)
STARTUP_TIME=$(( (END - START) / 1000000 ))
echo "Startup Time: ${STARTUP_TIME}ms"
echo "${STARTUP_TIME}" > monitoring/startup-time.txt

# Generate report
cat > "monitoring/report-$(date +%Y%m%d).txt" << REPORT
Performance Monitoring Report
Date: $(date)
Binary Size: ${SIZE_MB}MB
Build Time: ${BUILD_TIME}s
Test Time: ${TEST_TIME}s
Startup Time: ${STARTUP_TIME}ms
REPORT

echo ""
echo "✅ Monitoring complete. Report saved to monitoring/report-$(date +%Y%m%d).txt"
