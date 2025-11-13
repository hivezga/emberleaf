#
# Sherpa-ONNX Bootstrap Script for Emberleaf (Windows)
# Automatically builds and installs Sherpa-ONNX for KWS and speaker recognition
#
# Usage (Admin PowerShell): .\scripts\install_sherpa_onnx.ps1
#

param(
    [string]$Tag = "v1.10.30"
)

$ErrorActionPreference = "Stop"

# Configuration
$SHERPA_REPO = "https://github.com/k2-fsa/sherpa-onnx.git"
$INSTALL_PREFIX = Join-Path $env:LOCALAPPDATA "sherpa-onnx"

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

function Check-Dependencies {
    Write-Info "Checking dependencies..."

    $missing = @()

    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        $missing += "git"
    }

    if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
        $missing += "cmake"
    }

    # Check for Visual Studio or Build Tools
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    $hasMSVC = $false

    if (Test-Path $vsWhere) {
        $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($vsPath) {
            $hasMSVC = $true
        }
    }

    if (-not $hasMSVC) {
        $missing += "Visual Studio 2019/2022 or Build Tools with C++ workload"
    }

    if ($missing.Count -gt 0) {
        Write-Error-Custom "Missing required dependencies: $($missing -join ', ')"
        Write-Host ""
        Write-Host "Please install them first:"
        Write-Host ""
        Write-Host "  1. Git for Windows: https://git-scm.com/download/win"
        Write-Host "  2. CMake: https://cmake.org/download/"
        Write-Host "  3. Visual Studio 2022 Community (free):"
        Write-Host "     https://visualstudio.microsoft.com/downloads/"
        Write-Host "     - Select 'Desktop development with C++' workload"
        Write-Host ""
        Write-Host "  OR install via winget:"
        Write-Host "     winget install Git.Git"
        Write-Host "     winget install Kitware.CMake"
        Write-Host "     winget install Microsoft.VisualStudio.2022.Community"
        Write-Host ""
        exit 1
    }

    Write-Success "All dependencies present"
}

function Clone-Sherpa {
    $workDir = Join-Path $PWD ".sherpa-build"

    Write-Info "Cloning Sherpa-ONNX $Tag..."

    if (Test-Path $workDir) {
        Write-Warning "Build directory exists, removing..."
        Remove-Item -Path $workDir -Recurse -Force
    }

    New-Item -ItemType Directory -Path $workDir | Out-Null
    Set-Location $workDir

    git clone --depth 1 --branch $Tag $SHERPA_REPO sherpa-onnx

    Write-Success "Cloned Sherpa-ONNX"
}

function Build-Sherpa {
    Write-Info "Building Sherpa-ONNX (this may take 10-15 minutes)..."

    Set-Location sherpa-onnx
    New-Item -ItemType Directory -Path "build" -Force | Out-Null
    Set-Location build

    # Find Visual Studio
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath

    if (-not $vsPath) {
        Write-Error-Custom "Could not find Visual Studio installation"
        exit 1
    }

    # Import Visual Studio environment
    $vcvarsPath = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"

    if (-not (Test-Path $vcvarsPath)) {
        Write-Error-Custom "Could not find vcvars64.bat"
        exit 1
    }

    Write-Info "Using Visual Studio at: $vsPath"

    # Configure with CMake
    cmake `
        -DCMAKE_BUILD_TYPE=Release `
        -DBUILD_SHARED_LIBS=ON `
        -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF `
        -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF `
        -DSHERPA_ONNX_ENABLE_GPU=OFF `
        -DCMAKE_INSTALL_PREFIX="$INSTALL_PREFIX" `
        -G "Visual Studio 17 2022" `
        -A x64 `
        ..

    # Build
    $numCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
    Write-Info "Building with $numCores parallel jobs..."

    cmake --build . --config Release --parallel $numCores

    Write-Success "Build completed"
}

function Install-Sherpa {
    Write-Info "Installing to $INSTALL_PREFIX..."

    cmake --install . --config Release

    Write-Success "Installation completed"
}

function Setup-Environment {
    $libDir = Join-Path $INSTALL_PREFIX "bin"  # On Windows, DLLs go in bin/
    $includeDir = Join-Path $INSTALL_PREFIX "include"

    Write-Info "Setting up environment variables..."

    # Set for current session
    $env:SHERPA_ONNX_DIR = $INSTALL_PREFIX
    $env:PATH = "$libDir;$env:PATH"

    Write-Success "Environment variables set for current session:"
    Write-Host "  SHERPA_ONNX_DIR=$env:SHERPA_ONNX_DIR"
    Write-Host "  PATH=$libDir;..."

    # Offer to persist to user environment
    Write-Host ""
    Write-Host "To make these permanent, they need to be added to your user environment variables."
    Write-Host ""

    $response = Read-Host "Add to user environment variables now? [y/N]"

    if ($response -match '^[Yy]$') {
        [System.Environment]::SetEnvironmentVariable("SHERPA_ONNX_DIR", $INSTALL_PREFIX, [System.EnvironmentVariableTarget]::User)

        $currentPath = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)
        if ($currentPath -notlike "*$libDir*") {
            $newPath = "$libDir;$currentPath"
            [System.Environment]::SetEnvironmentVariable("PATH", $newPath, [System.EnvironmentVariableTarget]::User)
        }

        Write-Success "Added to user environment variables"
        Write-Info "Restart your terminal for changes to take effect"
    } else {
        Write-Warning "Skipped. You'll need to set these manually or re-run this script."
    }
}

function Remove-BuildDir {
    $workDir = Join-Path $PWD ".sherpa-build"

    Write-Info "Cleaning up build directory..."
    Set-Location $PWD
    Remove-Item -Path $workDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Success "Cleanup completed"
}

function Test-Installation {
    Write-Info "Verifying installation..."

    $libDir = Join-Path $INSTALL_PREFIX "bin"
    $includeDir = Join-Path $INSTALL_PREFIX "include"

    # Check directories exist
    if (-not (Test-Path $libDir)) {
        Write-Error-Custom "Library directory not found: $libDir"
        exit 1
    }

    if (-not (Test-Path $includeDir)) {
        Write-Error-Custom "Include directory not found: $includeDir"
        exit 1
    }

    # Check for key library file
    $libFile = Get-ChildItem -Path $libDir -Filter "sherpa-onnx-c-api.dll" -ErrorAction SilentlyContinue

    if (-not $libFile) {
        Write-Error-Custom "Sherpa-ONNX C API library not found in $libDir"
        exit 1
    }

    # Check for header
    $headerPaths = @(
        (Join-Path $includeDir "sherpa-onnx\c-api\c-api.h"),
        (Join-Path $includeDir "sherpa-onnx-c-api.h")
    )

    $headerFound = $false
    foreach ($header in $headerPaths) {
        if (Test-Path $header) {
            $headerFound = $true
            break
        }
    }

    if (-not $headerFound) {
        Write-Error-Custom "Sherpa-ONNX C API header not found"
        exit 1
    }

    Write-Success "Installation verified!"
    Write-Host ""
    Write-Host "Sherpa-ONNX is ready at: $INSTALL_PREFIX"
    Write-Host "Libraries: $libDir"
    Write-Host "Headers: $includeDir"
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "  1. Run: .\scripts\fetch_models.ps1"
    Write-Host "  2. Build Emberleaf: npm run tauri dev"
}

# Main execution
function Main {
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host "  Sherpa-ONNX Bootstrap for Emberleaf" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host ""

    Check-Dependencies
    Clone-Sherpa
    Build-Sherpa
    Install-Sherpa
    Setup-Environment
    Remove-BuildDir
    Test-Installation

    Write-Host ""
    Write-Success "✓ Sherpa-ONNX installation complete!"
    Write-Host ""
}

# Run main function
Main
