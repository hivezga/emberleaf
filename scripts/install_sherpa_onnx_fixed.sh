#!/usr/bin/env bash
#
# Sherpa-ONNX Bootstrap Script for Emberleaf (Fixed for CMake 4.x)
# Automatically builds and installs Sherpa-ONNX for KWS and speaker recognition
#
# Usage: bash scripts/install_sherpa_onnx_fixed.sh
#

set -euo pipefail

# Configuration
SHERPA_TAG="v1.10.30"  # Pin to stable release
SHERPA_REPO="https://github.com/k2-fsa/sherpa-onnx.git"

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

# Detect OS and set install path
detect_os() {
    case "$(uname -s)" in
        Linux*)
            OS="Linux"
            INSTALL_PREFIX="$HOME/.local/sherpa-onnx"
            LD_VAR="LD_LIBRARY_PATH"
            RC_FILE="$HOME/.bashrc"
            ;;
        Darwin*)
            OS="macOS"
            INSTALL_PREFIX="$HOME/Library/sherpa-onnx"
            LD_VAR="DYLD_FALLBACK_LIBRARY_PATH"
            RC_FILE="$HOME/.zshrc"
            ;;
        *)
            error "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac

    info "Detected OS: $OS"
    info "Install prefix: $INSTALL_PREFIX"
}

# Check for required tools
check_dependencies() {
    info "Checking dependencies..."

    local missing=()

    if ! command -v git &> /dev/null; then
        missing+=("git")
    fi

    if ! command -v cmake &> /dev/null; then
        missing+=("cmake")
    fi

    if ! command -v make &> /dev/null; then
        missing+=("make")
    fi

    # Check for C++ compiler
    if ! command -v g++ &> /dev/null && ! command -v clang++ &> /dev/null; then
        missing+=("C++ compiler (g++ or clang++)")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        error "Missing required dependencies: ${missing[*]}"
        echo ""
        echo "Please install them first:"
        echo ""

        if [ "$OS" = "Linux" ]; then
            echo "  # Debian/Ubuntu:"
            echo "  sudo apt-get update"
            echo "  sudo apt-get install -y git cmake build-essential"
            echo ""
            echo "  # Fedora/RHEL:"
            echo "  sudo dnf install -y git cmake gcc-c++ make"
            echo ""
            echo "  # Arch:"
            echo "  sudo pacman -S git cmake base-devel"
        else
            echo "  # macOS:"
            echo "  xcode-select --install"
            echo "  brew install cmake"
        fi
        echo ""
        exit 1
    fi

    success "All dependencies present"

    # Show cmake version
    local cmake_version=$(cmake --version | head -n1 | awk '{print $3}')
    info "CMake version: $cmake_version"
}

# Clone Sherpa-ONNX repository
clone_sherpa() {
    local work_dir="$PWD/.sherpa-build"

    info "Cloning Sherpa-ONNX $SHERPA_TAG..."

    if [ -d "$work_dir/sherpa-onnx" ]; then
        warning "Build directory exists, removing..."
        rm -rf "$work_dir"
    fi

    mkdir -p "$work_dir"
    cd "$work_dir"

    git clone --depth 1 --branch "$SHERPA_TAG" "$SHERPA_REPO" sherpa-onnx

    success "Cloned Sherpa-ONNX"
}

# Patch CMake requirements for compatibility with CMake 4.x
patch_cmake_requirements() {
    info "Patching CMake requirements for modern CMake compatibility..."

    local build_dir="$PWD"

    # Wait for kaldi-native-fbank to be downloaded by CMake
    # We'll patch it after the first cmake run fails

    success "Patch preparation complete"
}

# Build Sherpa-ONNX
build_sherpa() {
    info "Building Sherpa-ONNX (this may take 5-10 minutes)..."

    cd sherpa-onnx
    mkdir -p build
    cd build

    # First cmake run (will download dependencies but may fail)
    info "Running initial cmake configuration (may fail - this is expected)..."

    set +e  # Temporarily disable exit on error
    cmake \
        -DCMAKE_BUILD_TYPE=Release \
        -DBUILD_SHARED_LIBS=ON \
        -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF \
        -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF \
        -DSHERPA_ONNX_ENABLE_GPU=OFF \
        -DCMAKE_INSTALL_PREFIX="$INSTALL_PREFIX" \
        .. 2>&1 | tee cmake_output.log

    local cmake_exit=$?
    set -e  # Re-enable exit on error

    # Check if kaldi-native-fbank was downloaded
    if [ -d "_deps/kaldi_native_fbank-src" ]; then
        info "Patching kaldi-native-fbank CMakeLists.txt..."

        # Patch the CMakeLists.txt to use CMake 3.8 minimum instead of old version
        local cmake_file="_deps/kaldi_native_fbank-src/CMakeLists.txt"

        if [ -f "$cmake_file" ]; then
            # Backup original
            cp "$cmake_file" "${cmake_file}.backup"

            # Replace cmake_minimum_required with modern version
            sed -i 's/cmake_minimum_required(VERSION 2\.[0-9]\+)/cmake_minimum_required(VERSION 3.8)/g' "$cmake_file"
            sed -i 's/cmake_minimum_required(VERSION 3\.[0-4])/cmake_minimum_required(VERSION 3.8)/g' "$cmake_file"

            success "Patched kaldi-native-fbank CMakeLists.txt"

            # Show what was changed
            info "CMake requirement changed to:"
            grep "cmake_minimum_required" "$cmake_file" | head -1
        fi
    fi

    # Run cmake again with the patched dependencies
    info "Re-running cmake configuration with patches..."
    cmake \
        -DCMAKE_BUILD_TYPE=Release \
        -DBUILD_SHARED_LIBS=ON \
        -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF \
        -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF \
        -DSHERPA_ONNX_ENABLE_GPU=OFF \
        -DCMAKE_INSTALL_PREFIX="$INSTALL_PREFIX" \
        ..

    # Build (use all available cores)
    local num_cores=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
    info "Building with $num_cores parallel jobs..."

    make -j"$num_cores"

    success "Build completed"
}

# Install Sherpa-ONNX
install_sherpa() {
    info "Installing to $INSTALL_PREFIX..."

    make install

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
            echo "# Sherpa-ONNX for Emberleaf (added by install_sherpa_onnx.sh)"
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

# Cleanup build directory
cleanup() {
    local work_dir="$(dirname $PWD)"

    info "Cleaning up build directory..."
    cd "$work_dir"
    rm -rf ".sherpa-build"
    success "Cleanup completed"
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
    echo "Sherpa-ONNX is ready at: $INSTALL_PREFIX"
    echo "Libraries: $lib_dir"
    echo "Headers: $include_dir"
    echo ""
    echo "Next steps:"
    echo "  1. Run: bash scripts/fetch_models.sh"
    echo "  2. Export: export SHERPA_ONNX_DIR=\"$INSTALL_PREFIX\""
    echo "  3. Build Emberleaf: npm run tauri dev"
}

# Main execution
main() {
    echo "======================================"
    echo "  Sherpa-ONNX Bootstrap for Emberleaf"
    echo "  (Fixed for CMake 4.x)"
    echo "======================================"
    echo ""

    detect_os
    check_dependencies
    clone_sherpa
    patch_cmake_requirements
    build_sherpa
    install_sherpa
    setup_environment
    cleanup
    verify_installation

    echo ""
    success "✓ Sherpa-ONNX installation complete!"
    echo ""
}

# Run main function
main "$@"
