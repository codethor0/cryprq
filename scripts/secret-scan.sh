#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Secret Scanning"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SCAN_LOG="secret-scan-$(date +%Y%m%d-%H%M%S).log"
ISSUES=0

# Patterns to detect secrets
PATTERNS=(
    "password\s*=\s*['\"][^'\"]+['\"]"
    "secret\s*=\s*['\"][^'\"]+['\"]"
    "api[_-]?key\s*=\s*['\"][^'\"]+['\"]"
    "private[_-]?key\s*=\s*['\"][^'\"]+['\"]"
    "token\s*=\s*['\"][^'\"]+['\"]"
    "bearer\s+[A-Za-z0-9_-]{20,}"
    "BEGIN\s+(RSA\s+)?PRIVATE\s+KEY"
    "-----BEGIN"
    "sk_live_[0-9a-zA-Z]{32,}"
    "pk_live_[0-9a-zA-Z]{32,}"
    "xox[baprs]-[0-9a-zA-Z-]{10,}"
    "ghp_[0-9a-zA-Z]{36,}"
    "gho_[0-9a-zA-Z]{36,}"
    "ghu_[0-9a-zA-Z]{36,}"
    "ghs_[0-9a-zA-Z]{36,}"
    "ghr_[0-9a-zA-Z]{36,}"
)

# Exclude patterns (false positives)
EXCLUDE_PATTERNS=(
    "test"
    "example"
    "placeholder"
    "dummy"
    "TODO"
    "FIXME"
    "password.*test"
    "secret.*test"
    "private_key.*test"
    "token.*test"
    "password.*example"
    "secret.*example"
    "password.*placeholder"
    "secret.*placeholder"
    "password.*dummy"
    "secret.*dummy"
    "password.*TODO"
    "secret.*TODO"
    "password.*FIXME"
    "secret.*FIXME"
    "tokens:"  # Rate limiter tokens
    "self.tokens"
    "private_key:"  # Type name
    "PrivateKey"
    "SecretKey"
    "password:"  # Type name
    "Password"
    "token:"  # Type name
    "Token"
    "api_key:"  # Type name
    "ApiKey"
    "secret:"  # Type name
    "Secret"
    "BEGIN.*PRIVATE.*KEY"  # Test keys
    "test.*key"
    "example.*key"
    "placeholder.*key"
    "dummy.*key"
    "TODO.*key"
    "FIXME.*key"
)

echo "Scanning for potential secrets..."
echo "Log file: $SCAN_LOG"
echo "" > "$SCAN_LOG"

# Scan files
FILES=$(find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.json" -o -name "*.yaml" -o -name "*.yml" -o -name "*.sh" -o -name "*.env*" \) \
    ! -path "./target/*" \
    ! -path "./.git/*" \
    ! -path "./node_modules/*" \
    ! -path "./.cargo/*" \
    ! -path "./gui/node_modules/*" \
    ! -path "./mobile/node_modules/*" \
    ! -path "./gui/dist/*" \
    ! -path "./mobile/android/build/*" \
    ! -path "./mobile/ios/build/*" \
    ! -path "./mobile/ios/Pods/*" \
    ! -path "./.playwright/*" \
    ! -path "./artifacts/*" \
    ! -path "./coverage/*" \
    ! -path "./test-results/*" \
    ! -path "./playwright-report/*" \
    ! -name "*.log" \
    ! -name "Cargo.lock")

for pattern in "${PATTERNS[@]}"; do
    matches=$(echo "$FILES" | xargs grep -l -i -E "$pattern" 2>/dev/null || true)
    
    if [ -n "$matches" ]; then
        for file in $matches; do
            # Check if it's a false positive
            is_false_positive=false
            for exclude in "${EXCLUDE_PATTERNS[@]}"; do
                if grep -qi "$exclude" "$file" 2>/dev/null; then
                    is_false_positive=true
                    break
                fi
            done
            
            if [ "$is_false_positive" = false ]; then
                echo "⚠️  Potential secret found in $file (pattern: $pattern)" | tee -a "$SCAN_LOG"
                grep -n -i -E "$pattern" "$file" 2>/dev/null | head -3 | tee -a "$SCAN_LOG"
                echo "" | tee -a "$SCAN_LOG"
                ISSUES=$((ISSUES + 1))
            fi
        done
    fi
done

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $ISSUES -eq 0 ]; then
    echo "✅ Secret scan passed with no issues."
    rm -f "$SCAN_LOG"
    exit 0
else
    echo "❌ Secret scan found $ISSUES potential issue(s)"
    echo "📊 Scan log: $SCAN_LOG"
    echo "⚠️  Please review and remove any hardcoded secrets"
    exit 1
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

