#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# iOS build script for CrypRQ Mobile
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/mobile"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🍎 Building iOS App"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "❌ ERROR: iOS builds require macOS"
    exit 1
fi

# Check for Xcode
if ! command -v xcodebuild &> /dev/null; then
    echo "❌ ERROR: xcodebuild not found. Please install Xcode."
    exit 1
fi

# Check for CocoaPods
if ! command -v pod &> /dev/null; then
    echo "⚠️  WARNING: CocoaPods not found. Installing..."
    sudo gem install cocoapods || {
        echo "❌ Failed to install CocoaPods"
        exit 1
    }
fi

# Install pods
if [ -d "ios" ]; then
    cd ios
    echo "📦 Installing CocoaPods dependencies..."
    pod install
    
    # Build for simulator
    echo ""
    echo "🔨 Building for iOS Simulator..."
    xcodebuild -workspace CrypRQ.xcworkspace \
        -scheme CrypRQ \
        -configuration Debug \
        -sdk iphonesimulator \
        -destination 'platform=iOS Simulator,name=iPhone 15' \
        clean build
    
    # Build for device (requires signing)
    echo ""
    echo "🔨 Building for iOS Device..."
    if [ -n "${IOS_TEAM_ID:-}" ]; then
        xcodebuild -workspace CrypRQ.xcworkspace \
            -scheme CrypRQ \
            -configuration Release \
            -sdk iphoneos \
            CODE_SIGN_IDENTITY="iPhone Developer" \
            DEVELOPMENT_TEAM="$IOS_TEAM_ID" \
            clean build
    else
        echo "⚠️  IOS_TEAM_ID not set. Skipping device build."
        echo "   Set IOS_TEAM_ID to build for device"
    fi
    
    # Archive (for App Store)
    echo ""
    echo "📦 Creating archive..."
    if [ -n "${IOS_TEAM_ID:-}" ]; then
        xcodebuild -workspace CrypRQ.xcworkspace \
            -scheme CrypRQ \
            -configuration Release \
            archive \
            -archivePath build/CrypRQ.xcarchive \
            CODE_SIGN_IDENTITY="iPhone Developer" \
            DEVELOPMENT_TEAM="$IOS_TEAM_ID"
    else
        echo "⚠️  IOS_TEAM_ID not set. Skipping archive."
    fi
else
    echo "❌ ERROR: ios/ directory not found"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ iOS build completed"
echo ""

exit 0

