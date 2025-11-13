#!/usr/bin/env bash
#
# AppImage packaging script for Emberleaf
# Produces: dist/Emberleaf-x86_64.AppImage
#
# Usage:
#   ./scripts/release/package_appimage.sh
#   or
#   npm run package:appimage
#

set -euo pipefail

cd "$(dirname "$0")/../.."

echo "🔧 Building Emberleaf AppImage..."

# Ensure dependencies are installed
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🎨 Building frontend..."
npm run build

# Build backend + bundle AppImage
echo "🦀 Building Rust backend and bundling AppImage..."
cd src-tauri

# Use Tauri CLI to build with AppImage bundler
# The appimage bundler is configured in tauri.conf.json
cargo tauri build --bundles appimage

cd ..

# Move artifacts to dist/
echo "📦 Collecting artifacts..."
mkdir -p dist

# Find the generated AppImage
APPIMAGE_PATH=$(find src-tauri/target/release/bundle/appimage -name "*.AppImage" | head -n 1)

if [ -z "$APPIMAGE_PATH" ]; then
    echo "❌ Error: AppImage not found in target/release/bundle/appimage"
    exit 1
fi

# Copy to dist with canonical name
cp "$APPIMAGE_PATH" dist/Emberleaf-x86_64.AppImage

echo "✅ AppImage built successfully:"
echo "   dist/Emberleaf-x86_64.AppImage"

# Generate SHA256 checksum
cd dist
sha256sum Emberleaf-x86_64.AppImage > Emberleaf-x86_64.AppImage.sha256
echo "   dist/Emberleaf-x86_64.AppImage.sha256"

echo ""
echo "🚀 Ready to distribute!"
echo "   Users can download and run: chmod +x Emberleaf-x86_64.AppImage && ./Emberleaf-x86_64.AppImage"
