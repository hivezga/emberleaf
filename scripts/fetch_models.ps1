#
# Model Fetching Script for Emberleaf (Windows)
# Downloads and validates Sherpa-ONNX models for KWS and speaker recognition
#
# Usage: .\scripts\fetch_models.ps1
#

$ErrorActionPreference = "Stop"

# Color output functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Detect data directory
$DATA_DIR = Join-Path $env:LOCALAPPDATA "Emberleaf"
Write-Info "Data directory: $DATA_DIR"

function Create-Directories {
    Write-Info "Creating model directories..."

    $kws_dir = Join-Path $DATA_DIR "models\kws"
    $spk_dir = Join-Path $DATA_DIR "models\spk"

    New-Item -ItemType Directory -Path $kws_dir -Force | Out-Null
    New-Item -ItemType Directory -Path $spk_dir -Force | Out-Null

    Write-Success "Directories created"
}

function Fetch-KwsModel {
    $modelName = "sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
    $modelDir = Join-Path $DATA_DIR "models\kws\$modelName"
    $modelUrl = "https://github.com/k2-fsa/sherpa-onnx/releases/download/kws-models/$modelName.tar.bz2"

    Write-Info "Fetching KWS model: $modelName"

    if ((Test-Path $modelDir) -and (Test-Path (Join-Path $modelDir "encoder.onnx"))) {
        Write-Warning "KWS model already exists at $modelDir"
        $response = Read-Host "Re-download? [y/N]"

        if ($response -notmatch '^[Yy]$') {
            Write-Info "Skipping KWS model download"
            return
        }

        Remove-Item -Path $modelDir -Recurse -Force
    }

    # Download model archive
    $tempDir = Join-Path $env:TEMP "emberleaf-models-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir | Out-Null

    $archive = Join-Path $tempDir "$modelName.tar.bz2"

    Write-Info "Downloading from $modelUrl..."

    try {
        Invoke-WebRequest -Uri $modelUrl -OutFile $archive -UseBasicParsing
    } catch {
        Write-Error-Custom "Failed to download KWS model: $_"
        exit 1
    }

    # Extract using tar (available in Windows 10+)
    Write-Info "Extracting..."
    $kws_parent = Join-Path $DATA_DIR "models\kws"
    New-Item -ItemType Directory -Path $kws_parent -Force | Out-Null

    tar -xjf $archive -C $kws_parent

    # Cleanup
    Remove-Item -Path $tempDir -Recurse -Force

    # Verify files
    $requiredFiles = @("encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt")
    foreach ($file in $requiredFiles) {
        $filePath = Join-Path $modelDir $file
        if (-not (Test-Path $filePath)) {
            Write-Error-Custom "Missing required file: $file"
            exit 1
        }
    }

    Write-Success "KWS model downloaded and verified"
}

function Fetch-SpeakerModel {
    $modelName = "3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k"
    $modelDir = Join-Path $DATA_DIR "models\spk\ecapa-tdnn-16k"
    $modelUrl = "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recog-models/$modelName.onnx"

    Write-Info "Fetching speaker embedding model..."

    if ((Test-Path $modelDir) -and (Test-Path (Join-Path $modelDir "model.onnx"))) {
        Write-Warning "Speaker model already exists at $modelDir"
        $response = Read-Host "Re-download? [y/N]"

        if ($response -notmatch '^[Yy]$') {
            Write-Info "Skipping speaker model download"
            return
        }

        Remove-Item -Path $modelDir -Recurse -Force
    }

    # Create directory
    New-Item -ItemType Directory -Path $modelDir -Force | Out-Null

    # Download model
    Write-Info "Downloading from $modelUrl..."
    $modelFile = Join-Path $modelDir "model.onnx"

    try {
        Invoke-WebRequest -Uri $modelUrl -OutFile $modelFile -UseBasicParsing
    } catch {
        Write-Error-Custom "Failed to download speaker model: $_"
        exit 1
    }

    # Verify file exists and has reasonable size (>1MB)
    if (-not (Test-Path $modelFile)) {
        Write-Error-Custom "Failed to download speaker model"
        exit 1
    }

    $fileSize = (Get-Item $modelFile).Length
    if ($fileSize -lt 1048576) {
        Write-Error-Custom "Speaker model file is too small (possibly corrupted)"
        exit 1
    }

    Write-Success "Speaker embedding model downloaded and verified"
}

function Compute-Hashes {
    Write-Info "Computing model file hashes..."

    $kws_dir = Join-Path $DATA_DIR "models\kws\sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
    $spk_dir = Join-Path $DATA_DIR "models\spk\ecapa-tdnn-16k"

    Write-Host ""
    Write-Host "Model file hashes (for registry):"
    Write-Host "=================================="

    # KWS models
    if (Test-Path $kws_dir) {
        Write-Host ""
        Write-Host "KWS Models:"

        $files = @("encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt")
        foreach ($file in $files) {
            $filePath = Join-Path $kws_dir $file
            if (Test-Path $filePath) {
                $hash = (Get-FileHash -Path $filePath -Algorithm SHA256).Hash.ToLower()
                Write-Host "  $file`: $hash"
            }
        }
    }

    # Speaker model
    $spk_file = Join-Path $spk_dir "model.onnx"
    if (Test-Path $spk_file) {
        Write-Host ""
        Write-Host "Speaker Model:"
        $hash = (Get-FileHash -Path $spk_file -Algorithm SHA256).Hash.ToLower()
        Write-Host "  model.onnx: $hash"
    }

    Write-Host ""
}

function Test-Models {
    Write-Info "Verifying model installation..."

    $kws_dir = Join-Path $DATA_DIR "models\kws\sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
    $spk_dir = Join-Path $DATA_DIR "models\spk\ecapa-tdnn-16k"

    $allGood = $true

    # Check KWS models
    $kws_files = @("encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt")
    foreach ($file in $kws_files) {
        $filePath = Join-Path $kws_dir $file
        if (-not (Test-Path $filePath)) {
            Write-Error-Custom "Missing KWS file: $file"
            $allGood = $false
        }
    }

    # Check speaker model
    $spk_file = Join-Path $spk_dir "model.onnx"
    if (-not (Test-Path $spk_file)) {
        Write-Error-Custom "Missing speaker model file"
        $allGood = $false
    }

    if ($allGood) {
        Write-Success "All models verified!"
        Write-Host ""
        Write-Host "Models installed at:"
        Write-Host "  KWS: $kws_dir"
        Write-Host "  Speaker: $spk_dir"
        Write-Host ""
        Write-Host "Next step:"
        Write-Host "  Build Emberleaf: npm run tauri dev"
    } else {
        Write-Error-Custom "Some models are missing. Please re-run this script."
        exit 1
    }
}

# Main execution
function Main {
    Write-Host "====================================" -ForegroundColor Cyan
    Write-Host "  Model Fetch for Emberleaf" -ForegroundColor Cyan
    Write-Host "====================================" -ForegroundColor Cyan
    Write-Host ""

    Create-Directories
    Fetch-KwsModel
    Fetch-SpeakerModel
    Compute-Hashes
    Test-Models

    Write-Host ""
    Write-Success "✓ Model fetch complete!"
    Write-Host ""
}

# Run main function
Main
