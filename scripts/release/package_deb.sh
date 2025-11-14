#!/usr/bin/env bash
set -euo pipefail

# REL-002: .deb build
# Requires: tauri-bundler (or cargo-deb), control metadata in tauri.conf.json/Cargo.toml
# Output: dist/emberleaf_*.deb + SHASUMS256.txt

DIST_DIR="dist"
mkdir -p "$DIST_DIR"

echo "==> Building .deb..."
# If using tauri-bundler (preferred):
#   npm run tauri build -- --bundles deb
# Or cargo-deb flow could be placed here.

# Placeholder: ensure a file exists for CI wiring
DEB_NAME="emberleaf_0.9.0_amd64.deb"
touch "${DIST_DIR}/${DEB_NAME}"

echo "==> Generating SHA256..."
(
  cd "$DIST_DIR"
  sha256sum *.deb > SHASUMS256.txt
  echo "==> Done:"
  ls -lh *.deb SHASUMS256.txt
)
