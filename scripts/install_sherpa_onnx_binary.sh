#!/usr/bin/env bash
#
# Sherpa-ONNX Pre-built Binary Installer for Emberleaf
# Downloads and installs pre-compiled Sherpa-ONNX v1.10.30 binaries
# This avoids CMake 4.x compatibility issues by using official releases
#
# Usage: bash scripts/install_sherpa_onnx_binary.sh
#

set -euo pipefail

# Configuration
SHERPA_TAG="v1.10.30"
SHERPA_RELEASE_URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/${SHERPA_TAG}"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)
            os="linux"
            INSTALL_PREFIX="$HOME/.local/sherpa-onnx"
            LD_VAR="LD_LIBRARY_PATH"
            RC_FILE="$HOME/.bashrc"
            ;;
        Darwin*)
            os="macos"
            INSTALL_PREFIX="$HOME/Library/sherpa-onnx"
            LD_VAR="DYLD_FALLBACK_LIBRARY_PATH"
            RC_FILE="$HOME/.zshrc"
            ;;
        *)
            error "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    OS="$os"
    ARCH="$arch"

    info "Detected platform: $OS-$ARCH"
    info "Install prefix: $INSTALL_PREFIX"
}

# Determine binary package name
get_binary_package() {
    # Sherpa-ONNX release naming: sherpa-onnx-v1.10.30-linux-x64-shared.tar.bz2
    case "$OS-$ARCH" in
        linux-x86_64)
            BINARY_PACKAGE="sherpa-onnx-${SHERPA_TAG}-linux-x64-shared.tar.bz2"
            ;;
        linux-aarch64)
            BINARY_PACKAGE="sherpa-onnx-${SHERPA_TAG}-linux-arm64-shared.tar.bz2"
            ;;
        macos-x86_64)
            BINARY_PACKAGE="sherpa-onnx-${SHERPA_TAG}-osx-x86_64-shared.tar.bz2"
            ;;
        macos-aarch64)
            BINARY_PACKAGE="sherpa-onnx-${SHERPA_TAG}-osx-arm64-shared.tar.bz2"
            ;;
        *)
            error "No pre-built binary available for $OS-$ARCH"
            echo ""
            echo "You'll need to build from source. See: https://k2-fsa.github.io/sherpa/onnx/install/from-source.html"
            exit 1
            ;;
    esac

    DOWNLOAD_URL="${SHERPA_RELEASE_URL}/${BINARY_PACKAGE}"
    info "Binary package: $BINARY_PACKAGE"
}

# Download binary package
download_binary() {
    local work_dir="$PWD/.sherpa-binary"

    info "Downloading Sherpa-ONNX $SHERPA_TAG pre-built binaries..."

    if [ -d "$work_dir" ]; then
        warning "Download directory exists, removing..."
        rm -rf "$work_dir"
    fi

    mkdir -p "$work_dir"
    cd "$work_dir"

    # Download with curl or wget
    if command -v curl &> /dev/null; then
        curl -SL -o "$BINARY_PACKAGE" "$DOWNLOAD_URL"
    elif command -v wget &> /dev/null; then
        wget -O "$BINARY_PACKAGE" "$DOWNLOAD_URL"
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    if [ ! -f "$BINARY_PACKAGE" ]; then
        error "Download failed: $BINARY_PACKAGE not found"
        exit 1
    fi

    success "Downloaded $BINARY_PACKAGE"
}

# Extract and install binary
install_binary() {
    info "Extracting binary package..."

    # Extract tarball
    tar -xf "$BINARY_PACKAGE"

    # Find extracted directory (should be sherpa-onnx-v1.10.30-linux-x64-shared or similar)
    local extracted_dir
    extracted_dir=$(tar -tf "$BINARY_PACKAGE" | head -1 | cut -f1 -d"/")

    if [ ! -d "$extracted_dir" ]; then
        error "Extraction failed: $extracted_dir not found"
        exit 1
    fi

    info "Installing to $INSTALL_PREFIX..."

    # Remove old installation if exists
    if [ -d "$INSTALL_PREFIX" ]; then
        warning "Removing old installation at $INSTALL_PREFIX"
        rm -rf "$INSTALL_PREFIX"
    fi

    # Create parent directory
    mkdir -p "$(dirname "$INSTALL_PREFIX")"

    # Move extracted directory to install location
    mv "$extracted_dir" "$INSTALL_PREFIX"

    success "Installation completed"
}

# Set up environment
setup_environment() {
    local lib_dir="$INSTALL_PREFIX/lib"

    info "Setting up environment variables..."

    # Set for current session
    export SHERPA_ONNX_DIR="$INSTALL_PREFIX"
    export ${LD_VAR}="$lib_dir:${!LD_VAR:-}"
    export PATH="$INSTALL_PREFIX/bin:$PATH"

    success "Environment variables set for current session:"
    echo "  SHERPA_ONNX_DIR=$SHERPA_ONNX_DIR"
    echo "  $LD_VAR=$lib_dir:..."
    echo "  PATH=$INSTALL_PREFIX/bin:..."

    # Offer to persist to shell RC
    echo ""
    echo "To make these permanent, add to your shell RC file ($RC_FILE):"
    echo ""
    echo "  export SHERPA_ONNX_DIR=\"$INSTALL_PREFIX\""
    echo "  export $LD_VAR=\"$lib_dir:\${$LD_VAR}\""
    echo "  export PATH=\"$INSTALL_PREFIX/bin:\$PATH\""
    echo ""

    read -p "Add to $RC_FILE now? [y/N] " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        {
            echo ""
            echo "# Sherpa-ONNX for Emberleaf (added by install_sherpa_onnx_binary.sh)"
            echo "export SHERPA_ONNX_DIR=\"$INSTALL_PREFIX\""
            echo "export $LD_VAR=\"$lib_dir:\${$LD_VAR}\""
            echo "export PATH=\"$INSTALL_PREFIX/bin:\$PATH\""
        } >> "$RC_FILE"

        success "Added to $RC_FILE"
        info "Restart your shell or run: source $RC_FILE"
    else
        warning "Skipped. You'll need to set these manually or re-run this script."
    fi
}

# Cleanup download directory
cleanup() {
    local work_dir="$PWD/.sherpa-binary"

    if [ -d "$work_dir" ]; then
        info "Cleaning up download directory..."
        cd "$PWD"
        rm -rf "$work_dir"
        success "Cleanup completed"
    fi
}

# Verify installation
verify_installation() {
    info "Verifying installation..."

    local lib_dir="$INSTALL_PREFIX/lib"
    local include_dir="$INSTALL_PREFIX/include"

    # Check directories exist
    if [ ! -d "$lib_dir" ]; then
        error "Library directory not found: $lib_dir"
        exit 1
    fi

    if [ ! -d "$include_dir" ]; then
        error "Include directory not found: $include_dir"
        exit 1
    fi

    # Check for key library file
    local lib_pattern="libsherpa-onnx-c-api.*"
    if ! ls "$lib_dir"/$lib_pattern 1> /dev/null 2>&1; then
        error "Sherpa-ONNX C API library not found in $lib_dir"
        exit 1
    fi

    # Check for header
    local header_paths=(
        "$include_dir/sherpa-onnx/c-api/c-api.h"
        "$include_dir/sherpa-onnx-c-api.h"
    )

    local header_found=false
    for header in "${header_paths[@]}"; do
        if [ -f "$header" ]; then
            header_found=true
            break
        fi
    done

    if [ "$header_found" = false ]; then
        error "Sherpa-ONNX C API header not found"
        exit 1
    fi

    success "Installation verified!"
    echo ""
    echo "Sherpa-ONNX $SHERPA_TAG is ready at: $INSTALL_PREFIX"
    echo "Libraries: $lib_dir"
    echo "Headers: $include_dir"
    echo ""

    # Export for current shell session
    export SHERPA_ONNX_DIR="$INSTALL_PREFIX"
    export ${LD_VAR}="$lib_dir:${!LD_VAR:-}"
    info "Using SHERPA_ONNX_DIR=$SHERPA_ONNX_DIR"
    info "Using ${LD_VAR}=$lib_dir:..."

    echo ""
    echo "Next steps:"
    echo "  1. Run: bash scripts/fetch_models.sh"
    echo "  2. Build Emberleaf: npm run tauri dev"
}

# Main execution
main() {
    echo "=========================================="
    echo "  Sherpa-ONNX Binary Installer"
    echo "  Version: $SHERPA_TAG (pre-built)"
    echo "=========================================="
    echo ""
    echo "This script downloads pre-compiled Sherpa-ONNX binaries"
    echo "to avoid CMake build issues on systems with CMake 4.x."
    echo ""

    detect_platform
    get_binary_package
    download_binary
    install_binary
    setup_environment
    cleanup
    verify_installation

    echo ""
    success "✓ Sherpa-ONNX binary installation complete!"
    echo ""
}

# Run main function
main "$@"
