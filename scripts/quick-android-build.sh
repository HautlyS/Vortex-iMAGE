#!/bin/bash
# Quick Android build script - assumes environment is already set up
# Run ./scripts/android-setup.sh setup first if this fails

set -e

echo "🚀 Building iMAGE for Android..."

# Check if Android is initialized
if [ ! -d "src-tauri/gen/android" ]; then
    echo "📱 Initializing Android project..."
    pnpm tauri android init
fi

# Build
echo "🔨 Building APK..."
pnpm tauri android build

echo ""
echo "✅ Build complete!"
echo "📦 APK location: src-tauri/gen/android/app/build/outputs/apk/universal/release/"
echo ""
echo "To install on connected device:"
echo "  adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk"
