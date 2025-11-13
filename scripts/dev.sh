#!/usr/bin/env bash
#
# Emberleaf Development Launch Script
# Runs the app with X11 backend (Wayland workaround) and Rust logging enabled
#
# Usage: bash scripts/dev.sh [log_level]
#   log_level: trace, debug, info (default), warn, error
#

set -euo pipefail

# Color output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

# Set log level (default: info)
LOG_LEVEL="${1:-info}"

# Validate log level
case "$LOG_LEVEL" in
    trace|debug|info|warn|error)
        ;;
    *)
        warning "Invalid log level '$LOG_LEVEL', using 'info'"
        LOG_LEVEL="info"
        ;;
esac

echo "======================================="
echo "  Emberleaf Development Environment"
echo "======================================="
echo ""

# Check Sherpa-ONNX installation
if [ ! -d "$HOME/.local/sherpa-onnx" ]; then
    warning "Sherpa-ONNX not found at $HOME/.local/sherpa-onnx"
    echo "Run: npm run setup:audio:posix"
    exit 1
fi

info "Sherpa-ONNX found: $HOME/.local/sherpa-onnx"

# Check models
if [ ! -d "$HOME/.local/share/Emberleaf/models/kws" ]; then
    warning "KWS models not found"
    echo "Run: bash scripts/fetch_models.sh"
    exit 1
fi

info "Models found: $HOME/.local/share/Emberleaf/models"

# Set environment variables
export SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx"
export LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib:${LD_LIBRARY_PATH:-}"
export PATH="$HOME/.local/sherpa-onnx/bin:${PATH:-}"

# Backend selection: Respects EMB_DISPLAY_BACKEND or legacy DEV_WAYLAND/DEV_X11
# Priority: EMB_DISPLAY_BACKEND > DEV_WAYLAND/DEV_X11 > auto
if [ -n "${EMB_DISPLAY_BACKEND}" ]; then
    info "Display backend: EMB_DISPLAY_BACKEND=${EMB_DISPLAY_BACKEND}"
elif [ "${DEV_WAYLAND:-0}" = "1" ]; then
    export EMB_DISPLAY_BACKEND=wayland
    info "Display backend: wayland (via DEV_WAYLAND=1)"
elif [ "${DEV_X11:-0}" = "1" ]; then
    export EMB_DISPLAY_BACKEND=x11
    info "Display backend: x11 (via DEV_X11=1)"
else
    info "Display backend: auto-detect (set EMB_DISPLAY_BACKEND to override)"
fi

# Note: Actual backend configuration happens in Rust (display_backend.rs)

# Enable Rust logging
export RUST_LOG="$LOG_LEVEL"

# Optional: Enable detailed logging for specific modules
# Uncomment and customize as needed:
# export RUST_LOG="emberleaf=debug,tauri=info"
# export RUST_LOG="emberleaf::audio::kws=trace,emberleaf::audio::vad=debug"
# export RUST_LOG="emberleaf::voice::biometrics=debug"

info "Environment configured:"
echo "  SHERPA_ONNX_DIR=$SHERPA_ONNX_DIR"
echo "  LD_LIBRARY_PATH=$HOME/.local/sherpa-onnx/lib:..."
echo "  GDK_BACKEND=$GDK_BACKEND (X11/Xwayland)"
echo "  RUST_LOG=$RUST_LOG"
echo ""

success "Starting Emberleaf in development mode..."
echo ""

# Launch the app
npm run tauri dev
