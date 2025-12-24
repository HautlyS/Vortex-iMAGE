#!/bin/bash
set -e
cd "$(dirname "$0")/.."

echo "Building Vortex AppImage..."

# Check dependencies
if ! command -v cargo &>/dev/null; then
    echo "Error: cargo not found"
    exit 1
fi

if ! command -v node &>/dev/null; then
    echo "Error: node not found"
    exit 1
fi

# Determine package manager
if [ -f "pnpm-lock.yaml" ]; then
    PKG_MGR="pnpm"
elif [ -f "yarn.lock" ]; then
    PKG_MGR="yarn"
else
    PKG_MGR="npm"
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    $PKG_MGR install
fi

# Install tauri-cli if needed
if ! cargo tauri --version &>/dev/null 2>&1; then
    echo "Installing tauri-cli..."
    cargo install tauri-cli
fi

# Build frontend
echo "Building frontend..."
$PKG_MGR run build

# Build AppImage
echo "Building AppImage..."
cargo tauri build --bundles appimage

echo "✓ Build complete!"
echo "Output: ./src-tauri/target/release/bundle/"
