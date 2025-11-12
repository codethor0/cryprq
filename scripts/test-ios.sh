#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# iOS test script for CrypRQ Mobile
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/mobile"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Testing iOS App"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "❌ ERROR: iOS tests require macOS"
    exit 1
fi

# Check for Xcode
if ! command -v xcodebuild &> /dev/null; then
    echo "❌ ERROR: xcodebuild not found. Please install Xcode."
    exit 1
fi

# Run Jest tests
if [ -f "package.json" ]; then
    echo "📦 Running Jest unit tests..."
    npm test || {
        echo "❌ Jest tests failed"
        exit 1
    }
fi

# Run Xcode tests
if [ -d "ios" ]; then
    cd ios
    
    # Install pods if needed
    if [ ! -d "Pods" ]; then
        echo "📦 Installing CocoaPods dependencies..."
        pod install
    fi
    
    echo ""
    echo "🔨 Running Xcode unit tests..."
    xcodebuild test \
        -workspace CrypRQ.xcworkspace \
        -scheme CrypRQ \
        -sdk iphonesimulator \
        -destination 'platform=iOS Simulator,name=iPhone 15' \
        -only-testing:CrypRQTests || {
        echo "❌ Xcode tests failed"
        exit 1
    }
fi

# Run Detox E2E tests if available
if [ -f "package.json" ] && grep -q "detox" package.json; then
    echo ""
    echo "🔗 Running Detox E2E tests..."
    if command -v detox &> /dev/null; then
        detox test --configuration ios.sim.debug
    else
        echo "⚠️  Detox not found. Install with: npm install -g detox-cli"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ iOS tests completed"
echo ""

exit 0

