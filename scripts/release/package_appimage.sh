#!/usr/bin/env bash
set -euo pipefail

# REL-002: AppImage build (Linux-first)
# Requires: linuxdeploy, appimagetool OR tauri-bundler (recommended)
# Output: dist/Emberleaf-x86_64.AppImage + SHASUMS256.txt

APP_NAME="Emberleaf"
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

echo "==> Building AppImage..."
# If using tauri-bundler (preferred):
#   npm run tauri build -- --bundles appimage
# For a pure AppImage toolchain, hook linuxdeploy here.

# Placeholder: ensure a file exists for CI wiring
touch "${DIST_DIR}/${APP_NAME}-x86_64.AppImage"

echo "==> Generating SHA256..."
(
  cd "$DIST_DIR"
  sha256sum *.AppImage > SHASUMS256.txt
  echo "==> Done:"
  ls -lh *.AppImage SHASUMS256.txt
)
