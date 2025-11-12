#!/bin/bash
# Weekly Dependency Check Script
# Run every Monday to check for updates and vulnerabilities

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Weekly Dependency Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Date: $(date)"
echo ""

# Security audit
echo "1. Running security audit..."
cargo audit > "dependency-reports/audit-$(date +%Y%m%d).log" 2>&1
if [ $? -eq 0 ]; then
    echo "✅ No vulnerabilities found"
else
    echo "⚠️ Vulnerabilities found - check dependency-reports/audit-$(date +%Y%m%d).log"
fi

# Outdated dependencies
echo ""
echo "2. Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated > "dependency-reports/outdated-$(date +%Y%m%d).log" 2>&1
    echo "✅ Outdated dependencies report saved"
else
    echo "⚠️ cargo-outdated not installed (install with: cargo install cargo-outdated)"
fi

# Dependency count
echo ""
echo "3. Dependency statistics..."
echo "Total dependencies: $(grep -c '^name = ' Cargo.lock)"
echo ""

echo "✅ Dependency check complete. Reports saved to dependency-reports/"
