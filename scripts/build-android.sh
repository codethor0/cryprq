#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Android build script for CrypRQ Mobile
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/mobile"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🤖 Building Android App"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for required tools
if ! command -v java &> /dev/null; then
    echo "❌ ERROR: Java not found. Please install JDK."
    exit 1
fi

if [ ! -f "android/gradlew" ]; then
    echo "❌ ERROR: Gradle wrapper not found in android/"
    exit 1
fi

# Check for Android SDK
if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ]; then
    echo "⚠️  WARNING: ANDROID_HOME or ANDROID_SDK_ROOT not set"
    echo "   Set ANDROID_HOME to your Android SDK path"
fi

cd android

# Clean previous builds
echo "🧹 Cleaning previous builds..."
./gradlew clean

# Build debug APK
echo ""
echo "🔨 Building debug APK..."
./gradlew assembleDebug

# Build release AAB (if keystore exists)
if [ -f "app/upload.keystore" ] || [ -f "upload.keystore" ]; then
    echo ""
    echo "🔨 Building release AAB..."
    ./gradlew bundleRelease
else
    echo ""
    echo "⚠️  Keystore not found. Skipping release build."
    echo "   Create upload.keystore to build release AAB"
fi

# Find output files
APK_PATH=$(find app/build/outputs/apk -name "*.apk" | head -1)
AAB_PATH=$(find app/build/outputs/bundle -name "*.aab" | head -1)

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Android build completed"
echo ""
if [ -n "$APK_PATH" ]; then
    echo "📦 Debug APK: $APK_PATH"
    ls -lh "$APK_PATH"
fi
if [ -n "$AAB_PATH" ]; then
    echo "📦 Release AAB: $AAB_PATH"
    ls -lh "$AAB_PATH"
fi
echo ""

exit 0

