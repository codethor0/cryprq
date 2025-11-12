#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "GUI Testing - Cross-Platform Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

PASSED=0
FAILED=0

cd "$(dirname "$0")/../gui" || exit 1

# Test 1: TypeScript compilation
echo "Test 1: TypeScript Compilation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if npx tsc --noEmit > /tmp/gui-tsc.log 2>&1; then
    echo "PASSED: TypeScript compilation successful"
    PASSED=$((PASSED + 1))
else
    echo "FAILED: TypeScript compilation errors"
    cat /tmp/gui-tsc.log | head -20
    FAILED=$((FAILED + 1))
fi
echo ""

# Test 2: Linting
echo "Test 2: Code Linting"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if npm run lint > /tmp/gui-lint.log 2>&1; then
    echo "PASSED: Linting successful"
    PASSED=$((PASSED + 1))
else
    echo "WARNING: Linting issues found (non-blocking)"
    cat /tmp/gui-lint.log | tail -10
fi
echo ""

# Test 3: Build verification
echo "Test 3: Build Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if npm run build > /tmp/gui-build.log 2>&1; then
    echo "PASSED: Build successful"
    PASSED=$((PASSED + 1))
    if [ -d "dist" ] || [ -d "dist-electron" ]; then
        echo "PASSED: Build artifacts found"
        PASSED=$((PASSED + 1))
    else
        echo "WARNING: Build artifacts not found in expected location"
    fi
else
    echo "FAILED: Build failed"
    cat /tmp/gui-build.log | tail -20
    FAILED=$((FAILED + 1))
fi
echo ""

# Test 4: IPC handlers verification
echo "Test 4: IPC Handlers Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
REQUIRED_HANDLERS=("session:start" "session:stop" "session:restart" "metrics:get" "settings:load" "settings:save")
MISSING=0
for handler in "${REQUIRED_HANDLERS[@]}"; do
    if grep -r "ipcMain.handle.*$handler" electron/main/ > /dev/null 2>&1; then
        echo "PASSED: Handler '$handler' found"
    else
        echo "FAILED: Handler '$handler' not found"
        MISSING=$((MISSING + 1))
    fi
done
if [ $MISSING -eq 0 ]; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi
echo ""

# Test 5: Component structure
echo "Test 5: Component Structure"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
REQUIRED_COMPONENTS=("Dashboard" "Peers" "Settings" "Logs")
MISSING=0
for component in "${REQUIRED_COMPONENTS[@]}"; do
    if find src/components -name "*${component}*" -type f | grep -q .; then
        echo "PASSED: Component '$component' found"
    else
        echo "FAILED: Component '$component' not found"
        MISSING=$((MISSING + 1))
    fi
done
if [ $MISSING -eq 0 ]; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi
echo ""

# Test 6: Dependencies check
echo "Test 6: Dependencies Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f "node_modules/.bin/electron" ] || command -v electron > /dev/null 2>&1; then
    echo "PASSED: Electron found"
    PASSED=$((PASSED + 1))
else
    echo "WARNING: Electron not found (may need npm install)"
fi
if [ -f "node_modules/.bin/react-scripts" ] || [ -d "node_modules/react-scripts" ]; then
    echo "PASSED: React dependencies found"
    PASSED=$((PASSED + 1))
else
    echo "WARNING: React dependencies not found (may need npm install)"
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "GUI Testing Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total: $((PASSED + FAILED))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "All GUI tests passed!"
    exit 0
else
    echo "Some GUI tests failed. Please review the output above."
    exit 1
fi

