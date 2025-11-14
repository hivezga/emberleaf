#!/usr/bin/env bash
set -euo pipefail

# REL-002B: .deb build via Tauri bundler
# Requires: @tauri-apps/cli, system deps, dpkg
# Output: dist/emberleaf_*.deb + SHASUMS256.txt

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIST_DIR="$ROOT/dist"
BUNDLE_DIR="$ROOT/src-tauri/target/release/bundle/deb"

mkdir -p "$DIST_DIR"

echo "==> Building .deb via Tauri bundler"
cd "$ROOT"
npm run tauri build -- --bundles deb

echo "==> Collecting artifacts"
cp -v "$BUNDLE_DIR"/*.deb "$DIST_DIR/"

echo "==> Generating SHA256 (append to same file)"
(
  cd "$DIST_DIR"
  sha256sum *.deb >> SHASUMS256.txt
  echo "==> Done:"
  ls -lh *.deb SHASUMS256.txt
)
