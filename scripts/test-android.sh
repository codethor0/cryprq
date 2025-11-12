#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Android test script for CrypRQ Mobile
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/mobile"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Testing Android App"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for required tools
if [ ! -f "android/gradlew" ]; then
    echo "❌ ERROR: Gradle wrapper not found in android/"
    exit 1
fi

cd android

# Run unit tests
echo "📦 Running unit tests..."
./gradlew test

# Run instrumented tests (if device/emulator available)
if adb devices | grep -q "device$"; then
    echo ""
    echo "📱 Running instrumented tests..."
    ./gradlew connectedAndroidTest
else
    echo ""
    echo "⚠️  No Android device/emulator found. Skipping instrumented tests."
    echo "   Start an emulator or connect a device to run instrumented tests"
fi

# Run Detox E2E tests if available
if [ -f "../package.json" ] && grep -q "detox" ../package.json; then
    echo ""
    echo "🔗 Running Detox E2E tests..."
    cd ..
    if command -v detox &> /dev/null; then
        detox test --configuration android.emu.debug
    else
        echo "⚠️  Detox not found. Install with: npm install -g detox-cli"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Android tests completed"
echo ""

exit 0

