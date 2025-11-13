#!/usr/bin/env bash
#
# Model Fetching Script for Emberleaf
# Downloads and validates Sherpa-ONNX models for KWS and speaker recognition
#
# Usage: bash scripts/fetch_models.sh
#

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Detect OS and set data directory
detect_data_dir() {
    case "$(uname -s)" in
        Linux*)
            DATA_DIR="$HOME/.local/share/Emberleaf"
            ;;
        Darwin*)
            DATA_DIR="$HOME/Library/Application Support/Emberleaf"
            ;;
        *)
            error "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac

    info "Data directory: $DATA_DIR"
}

# Create directory structure
create_directories() {
    info "Creating model directories..."

    mkdir -p "$DATA_DIR/models/kws"
    mkdir -p "$DATA_DIR/models/spk"

    success "Directories created"
}

# Download and extract KWS model
fetch_kws_model() {
    local model_name="sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
    local model_dir="$DATA_DIR/models/kws/$model_name"
    local model_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/kws-models/$model_name.tar.bz2"

    info "Fetching KWS model: $model_name"

    if [ -d "$model_dir" ] && [ -f "$model_dir/encoder.onnx" ]; then
        warning "KWS model already exists at $model_dir"
        read -p "Re-download? [y/N] " -n 1 -r
        echo

        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Skipping KWS model download"
            return 0
        fi

        rm -rf "$model_dir"
    fi

    # Download model archive
    local temp_dir=$(mktemp -d)
    local archive="$temp_dir/$model_name.tar.bz2"

    info "Downloading from $model_url..."
    if command -v curl &> /dev/null; then
        curl -L -o "$archive" "$model_url"
    elif command -v wget &> /dev/null; then
        wget -O "$archive" "$model_url"
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    # Extract
    info "Extracting..."
    mkdir -p "$DATA_DIR/models/kws"
    tar -xjf "$archive" -C "$DATA_DIR/models/kws"

    # Cleanup
    rm -rf "$temp_dir"

    # Verify files
    local required_files=("encoder.onnx" "decoder.onnx" "joiner.onnx" "tokens.txt")
    for file in "${required_files[@]}"; do
        if [ ! -f "$model_dir/$file" ]; then
            error "Missing required file: $file"
            exit 1
        fi
    done

    success "KWS model downloaded and verified"
}

# Download and extract speaker embedding model
fetch_speaker_model() {
    local model_name="3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k"
    local model_dir="$DATA_DIR/models/spk/ecapa-tdnn-16k"
    local model_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recog-models/$model_name.onnx"

    info "Fetching speaker embedding model..."

    if [ -d "$model_dir" ] && [ -f "$model_dir/model.onnx" ]; then
        warning "Speaker model already exists at $model_dir"
        read -p "Re-download? [y/N] " -n 1 -r
        echo

        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Skipping speaker model download"
            return 0
        fi

        rm -rf "$model_dir"
    fi

    # Create directory
    mkdir -p "$model_dir"

    # Download model
    info "Downloading from $model_url..."
    if command -v curl &> /dev/null; then
        curl -L -o "$model_dir/model.onnx" "$model_url"
    elif command -v wget &> /dev/null; then
        wget -O "$model_dir/model.onnx" "$model_url"
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    # Verify file exists and has reasonable size (>1MB)
    if [ ! -f "$model_dir/model.onnx" ]; then
        error "Failed to download speaker model"
        exit 1
    fi

    local file_size=$(stat -f%z "$model_dir/model.onnx" 2>/dev/null || stat -c%s "$model_dir/model.onnx" 2>/dev/null)
    if [ "$file_size" -lt 1048576 ]; then
        error "Speaker model file is too small (possibly corrupted)"
        exit 1
    fi

    success "Speaker embedding model downloaded and verified"
}

# Compute SHA256 hashes
compute_hashes() {
    info "Computing model file hashes..."

    local kws_dir="$DATA_DIR/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
    local spk_dir="$DATA_DIR/models/spk/ecapa-tdnn-16k"

    echo ""
    echo "Model file hashes (for registry):"
    echo "=================================="

    # KWS models
    if [ -d "$kws_dir" ]; then
        echo ""
        echo "KWS Models:"
        for file in encoder.onnx decoder.onnx joiner.onnx tokens.txt; do
            if [ -f "$kws_dir/$file" ]; then
                local hash=$(sha256sum "$kws_dir/$file" 2>/dev/null || shasum -a 256 "$kws_dir/$file" | cut -d' ' -f1)
                echo "  $file: $hash"
            fi
        done
    fi

    # Speaker model
    if [ -f "$spk_dir/model.onnx" ]; then
        echo ""
        echo "Speaker Model:"
        local hash=$(sha256sum "$spk_dir/model.onnx" 2>/dev/null || shasum -a 256 "$spk_dir/model.onnx" | cut -d' ' -f1)
        echo "  model.onnx: $hash"
    fi

    echo ""
}

# Verify installation
verify_models() {
    info "Verifying model installation..."

    local kws_dir="$DATA_DIR/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
    local spk_dir="$DATA_DIR/models/spk/ecapa-tdnn-16k"

    local all_good=true

    # Check KWS models
    local kws_files=("encoder.onnx" "decoder.onnx" "joiner.onnx" "tokens.txt")
    for file in "${kws_files[@]}"; do
        if [ ! -f "$kws_dir/$file" ]; then
            error "Missing KWS file: $file"
            all_good=false
        fi
    done

    # Check speaker model
    if [ ! -f "$spk_dir/model.onnx" ]; then
        error "Missing speaker model file"
        all_good=false
    fi

    if [ "$all_good" = true ]; then
        success "All models verified!"
        echo ""
        echo "Models installed at:"
        echo "  KWS: $kws_dir"
        echo "  Speaker: $spk_dir"
        echo ""
        echo "Next step:"
        echo "  Build Emberleaf: npm run tauri dev"
    else
        error "Some models are missing. Please re-run this script."
        exit 1
    fi
}

# Main execution
main() {
    echo "===================================="
    echo "  Model Fetch for Emberleaf"
    echo "===================================="
    echo ""

    detect_data_dir
    create_directories
    fetch_kws_model
    fetch_speaker_model
    compute_hashes
    verify_models

    echo ""
    success "✓ Model fetch complete!"
    echo ""
}

# Run main function
main "$@"
