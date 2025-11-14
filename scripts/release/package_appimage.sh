#!/usr/bin/env bash
set -euo pipefail

# REL-002B: AppImage build via Tauri bundler
# Requires: @tauri-apps/cli, system deps (webkit2gtk, gtk3, etc.)
# Output: dist/Emberleaf-x86_64.AppImage + SHASUMS256.txt

APP_NAME="Emberleaf"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIST_DIR="$ROOT/dist"
BUNDLE_DIR="$ROOT/src-tauri/target/release/bundle/appimage"

mkdir -p "$DIST_DIR"

echo "==> Building AppImage via Tauri bundler"
cd "$ROOT"
npm run tauri build -- --bundles appimage

echo "==> Collecting artifacts"
cp -v "$BUNDLE_DIR"/*.AppImage "$DIST_DIR/"

echo "==> Generating SHA256"
(
  cd "$DIST_DIR"
  sha256sum *.AppImage > SHASUMS256.txt
  echo "==> Done:"
  ls -lh *.AppImage SHASUMS256.txt
)
