# Sherpa-ONNX Development Setup

This guide covers setting up Sherpa-ONNX v1.10.30 for Emberleaf development.

## Quick Start

### Arch Linux One-Liner

```bash
sudo pacman -Syu --needed base-devel cmake ninja git python python-pip pkgconf clang rustup nodejs npm webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg alsa-lib pipewire wireplumber
rustup default stable
cd /mnt/data/dev/Ember && npm run setup:audio:posix && npm install && source ~/.bashrc && npm run tauri dev
```

**What this does:**
1. Installs all system dependencies (build tools, audio, GUI libraries)
2. Sets up Rust stable toolchain
3. Downloads and installs pre-built Sherpa-ONNX v1.10.30 to `$HOME/.local/sherpa-onnx`
4. Downloads KWS and speaker recognition models
5. Installs npm dependencies
6. Sources shell configuration to load Sherpa-ONNX environment variables
7. Launches the app in development mode

**Note:** The installer uses **pre-built binaries** by default to avoid CMake 4.x compatibility issues. If you need to build from source (e.g., for custom architectures), use `npm run setup:audio:build` instead.

**Environment Variables Set:**
- `SHERPA_ONNX_DIR`: `$HOME/.local/sherpa-onnx`
- `LD_LIBRARY_PATH`: `$HOME/.local/sherpa-onnx/lib:...`

### POSIX (Linux/macOS)

```bash
cd Ember
npm run setup:audio:posix  # Runs install_sherpa_onnx.sh && fetch_models.sh
npm run tauri dev
```

### Windows (PowerShell as Administrator)

```powershell
cd Ember
.\scripts\install_sherpa_onnx.ps1
.\scripts\fetch_models.ps1
npm run tauri dev
```

## What Gets Installed

### Sherpa-ONNX Library

**Linux:** `~/.local/sherpa-onnx/`
**macOS:** `~/Library/sherpa-onnx/`
**Windows:** `%LOCALAPPDATA%\sherpa-onnx\`

Contents:
- `lib/` - Shared libraries (libsherpa-onnx-c-api.so/.dylib/.dll)
- `include/` - C API headers
- `bin/` - Command-line tools (optional)

### Models

**Linux:** `~/.local/share/Emberleaf/models/`
**macOS:** `~/Library/Application Support/Emberleaf/models/`
**Windows:** `%LOCALAPPDATA%\Emberleaf\models\`

Structure:
```
models/
├── kws/
│   └── sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/
│       ├── encoder.onnx
│       ├── decoder.onnx
│       ├── joiner.onnx
│       └── tokens.txt
└── spk/
    └── ecapa-tdnn-16k/
        └── model.onnx
```

## Environment Variables

The install script sets these for the current session and offers to persist them:

### Linux

```bash
export SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx"
export LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib:$LD_LIBRARY_PATH"
export PATH="$HOME/.local/sherpa-onnx/bin:$PATH"
```

Add to `~/.bashrc` or `~/.zshrc` to make permanent.

### macOS

```bash
export SHERPA_ONNX_DIR="$HOME/Library/sherpa-onnx"
export DYLD_FALLBACK_LIBRARY_PATH="$HOME/Library/sherpa-onnx/lib:$DYLD_FALLBACK_LIBRARY_PATH"
export PATH="$HOME/Library/sherpa-onnx/bin:$PATH"
```

Add to `~/.zshrc` to make permanent.

### Windows

```powershell
$env:SHERPA_ONNX_DIR = "$env:LOCALAPPDATA\sherpa-onnx"
$env:PATH = "$env:LOCALAPPDATA\sherpa-onnx\bin;$env:PATH"
```

The install script adds these to user environment variables.

## Prerequisites

### Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y git cmake build-essential
```

### Linux (Fedora/RHEL)

```bash
sudo dnf install -y git cmake gcc-c++ make
```

### Linux (Arch)

```bash
sudo pacman -S git cmake base-devel
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install CMake via Homebrew
brew install cmake
```

### Windows

Install:
1. **Git for Windows**: https://git-scm.com/download/win
2. **CMake**: https://cmake.org/download/
3. **Visual Studio 2022 Community** (free): https://visualstudio.microsoft.com/downloads/
   - During installation, select "Desktop development with C++" workload

Or use winget:
```powershell
winget install Git.Git
winget install Kitware.CMake
winget install Microsoft.VisualStudio.2022.Community --override "--add Microsoft.VisualStudio.Workload.NativeDesktop --includeRecommended"
```

## Troubleshooting

### Build Fails: "cmake_minimum_required VERSION 3.3" Error with CMake 4.x

**Symptoms:** Build script loops indefinitely with errors like:
```
CMake Error: Compatibility with CMake < 3.5 has been removed from CMake.
```

**Root Cause:** CMake 4.x no longer supports projects with `cmake_minimum_required` < 3.5. Some Sherpa-ONNX dependencies (kaldi-native-fbank) still use VERSION 3.3.

**Solution:** Use pre-built binaries (default):
```bash
npm run setup:audio:posix  # Uses install_sherpa_onnx_binary.sh
```

This downloads official v1.10.30 release binaries and avoids CMake entirely.

**Alternative:** If you must build from source:
```bash
# Downgrade CMake temporarily
conda install cmake=3.28  # or use CMake 3.x from another source
npm run setup:audio:build
```

### Build Fails on Linux: "cmake: command not found"

**Solution:** Install CMake (only needed if building from source):
```bash
sudo apt-get install cmake  # Debian/Ubuntu
sudo dnf install cmake       # Fedora/RHEL
sudo pacman -S cmake         # Arch
```

### Build Fails on macOS: "No CMAKE_C_COMPILER could be found"

**Solution:** Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Build Fails on Windows: "Could not find Visual Studio"

**Solution:** Install Visual Studio 2022 with C++ workload (see Prerequisites above).

### Runtime Error: "libsherpa-onnx-c-api.so: cannot open shared object file"

**Solution:** Environment variables not set. Run:
```bash
# Linux
export LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib:$LD_LIBRARY_PATH"

# macOS
export DYLD_FALLBACK_LIBRARY_PATH="$HOME/Library/sherpa-onnx/lib:$DYLD_FALLBACK_LIBRARY_PATH"
```

Or re-run the install script and accept the offer to persist to your shell RC file.

### Model Download Fails: "curl: command not found"

**Solution:** Install curl or wget:
```bash
sudo apt-get install curl    # Debian/Ubuntu
sudo dnf install curl        # Fedora/RHEL
brew install curl            # macOS
```

### Models Downloaded but Not Found by Emberleaf

**Symptoms:** Error message "KWS model directory not found" or "Missing KWS model files".

**Solution:** Verify model paths:

```bash
# Linux
ls ~/.local/share/Emberleaf/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/
# Should show: encoder.onnx  decoder.onnx  joiner.onnx  tokens.txt

# macOS
ls ~/Library/Application\ Support/Emberleaf/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/

# Windows (PowerShell)
Get-ChildItem "$env:LOCALAPPDATA\Emberleaf\models\kws\sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
```

If files are missing, re-run:
```bash
bash scripts/fetch_models.sh    # POSIX
.\scripts\fetch_models.ps1       # Windows
```

### Ember Build Fails: "SHERPA_ONNX_DIR not set"

**Solution:** Set the environment variable in your current shell:

```bash
# Linux
export SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx"

# macOS
export SHERPA_ONNX_DIR="$HOME/Library/sherpa-onnx"

# Windows (PowerShell)
$env:SHERPA_ONNX_DIR = "$env:LOCALAPPDATA\sherpa-onnx"
```

Then rebuild:
```bash
npm run tauri dev
```

## Manual Cleanup

To remove Sherpa-ONNX and models:

### Linux

```bash
rm -rf ~/.local/sherpa-onnx
rm -rf ~/.local/share/Emberleaf/models
# Remove from ~/.bashrc if added
```

### macOS

```bash
rm -rf ~/Library/sherpa-onnx
rm -rf ~/Library/Application\ Support/Emberleaf/models
# Remove from ~/.zshrc if added
```

### Windows

```powershell
Remove-Item -Path "$env:LOCALAPPDATA\sherpa-onnx" -Recurse -Force
Remove-Item -Path "$env:LOCALAPPDATA\Emberleaf\models" -Recurse -Force
# Remove from user environment variables if added
```

## Advanced: Building Sherpa-ONNX Manually

If the automated script fails, you can build manually:

```bash
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx
git checkout v1.10.30

mkdir build && cd build
cmake \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=ON \
  -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF \
  -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF \
  -DSHERPA_ONNX_ENABLE_GPU=OFF \
  -DCMAKE_INSTALL_PREFIX="$HOME/.local/sherpa-onnx" \
  ..

make -j$(nproc)
make install
```

Then set `SHERPA_ONNX_DIR` and rebuild Emberleaf.

## Verification

After installation, verify everything is set up:

```bash
# Check Sherpa-ONNX installation
ls "$SHERPA_ONNX_DIR/lib"
ls "$SHERPA_ONNX_DIR/include"

# Check models
# Linux
ls ~/.local/share/Emberleaf/models/kws/
ls ~/.local/share/Emberleaf/models/spk/

# macOS
ls ~/Library/Application\ Support/Emberleaf/models/kws/
ls ~/Library/Application\ Support/Emberleaf/models/spk/

# Check environment
echo $SHERPA_ONNX_DIR
echo $LD_LIBRARY_PATH  # Linux
echo $DYLD_FALLBACK_LIBRARY_PATH  # macOS

# Build Emberleaf
npm run tauri dev
```

If Emberleaf builds and starts without errors, setup is complete!

## Next Steps

- See [BE-001-kws.md](BE-001-kws.md) for KWS usage
- See [BE-002-biometrics.md](BE-002-biometrics.md) for speaker recognition usage
