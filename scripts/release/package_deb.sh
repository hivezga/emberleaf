#!/usr/bin/env bash
#
# Debian package (.deb) packaging script for Emberleaf
# Produces: dist/emberleaf_*.deb
#
# Usage:
#   ./scripts/release/package_deb.sh
#   or
#   npm run package:deb
#

set -euo pipefail

cd "$(dirname "$0")/../.."

echo "🔧 Building Emberleaf .deb package..."

# Ensure dependencies are installed
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🎨 Building frontend..."
npm run build

# Build backend + bundle .deb
echo "🦀 Building Rust backend and bundling .deb..."
cd src-tauri

# Use Tauri CLI to build with deb bundler
cargo tauri build --bundles deb

cd ..

# Move artifacts to dist/
echo "📦 Collecting artifacts..."
mkdir -p dist

# Find the generated .deb
DEB_PATH=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -n 1)

if [ -z "$DEB_PATH" ]; then
    echo "❌ Error: .deb package not found in target/release/bundle/deb"
    exit 1
fi

# Copy to dist
DEB_NAME=$(basename "$DEB_PATH")
cp "$DEB_PATH" "dist/$DEB_NAME"

echo "✅ .deb package built successfully:"
echo "   dist/$DEB_NAME"

# Generate SHA256 checksum
cd dist
sha256sum "$DEB_NAME" > "${DEB_NAME}.sha256"
echo "   dist/${DEB_NAME}.sha256"

echo ""
echo "🚀 Ready to distribute!"
echo "   Install with: sudo dpkg -i dist/$DEB_NAME"
echo "   Or: sudo apt install ./dist/$DEB_NAME"
echo ""
echo "📋 Package dependencies (will be installed automatically):"
echo "   - pipewire | pulseaudio"
echo "   - libwebkit2gtk-4.0-37"
echo "   - xdg-desktop-portal (recommended)"
