# Fixed: CMake 4.x Infinite Loop Issue

## Problem Summary

The app was stuck in an infinite loop between build steps 580-583 when running `npm run tauri dev`. This was caused by **two separate issues**:

### Issue 1: CMake 4.x Compatibility (Build-from-Source)
**Root Cause:** CMake 4.x removed support for `cmake_minimum_required` < 3.5. Sherpa-ONNX dependency `kaldi-native-fbank` uses VERSION 3.3, causing build failures. The patching script tried to fix this but created an infinite loop because CMake's FetchContent re-downloads dependencies on each run, losing all patches.

### Issue 2: Tauri File Watcher Loop (Development)
**Root Cause:** The build script regenerated FFI bindings on every build, triggering Tauri's file watcher to rebuild, which regenerated bindings again, creating an infinite loop.

## Solution Implemented

### 1. Pre-built Binary Installer (Long-term Solution for Issue 1)

Created `scripts/install_sherpa_onnx_binary.sh` that:
- Downloads official Sherpa-ONNX v1.10.30 pre-compiled binaries from GitHub releases
- Installs to `$HOME/.local/sherpa-onnx` without requiring CMake build
- Automatically adds environment variables to `~/.bashrc`
- Completely bypasses CMake 4.x compatibility issues

**Usage:**
```bash
npm run setup:audio         # Uses binary installer
npm run setup:audio:build   # Uses build-from-source (if needed)
```

### 2. Smart Bindings Generation (Fix for Issue 2)

Modified `src-tauri/build.rs` to only regenerate bindings when:
- Bindings file doesn't exist, OR
- Header file is newer than existing bindings

This prevents unnecessary regeneration and file watcher loops.

**Key Change in build.rs:**
```rust
// Only regenerate if bindings don't exist or header is newer
let should_regenerate = if !bindings_path.exists() {
    true
} else {
    use std::time::SystemTime;
    let header_time = std::fs::metadata(&header_path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let bindings_time = std::fs::metadata(&bindings_path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    header_time > bindings_time
};
```

## Files Changed

1. **scripts/install_sherpa_onnx_binary.sh** (NEW)
   - Pre-built binary installer
   - Detects platform (Linux/macOS, x86_64/aarch64)
   - Downloads and extracts official releases

2. **package.json**
   - Updated `setup:audio` and `setup:audio:posix` to use binary installer
   - Added `setup:audio:build` for build-from-source option

3. **src-tauri/build.rs**
   - Added timestamp checking to prevent unnecessary bindings regeneration
   - Only regenerates when header file changes

4. **docs/DEV_SETUP_SHERPA.md**
   - Updated with CMake 4.x troubleshooting section
   - Documents binary installer as default solution
   - Explains build-from-source alternatives

5. **docs/BE-001-kws.md**
   - No changes needed (already documents v1.10.30 API)

## Environment Setup Required

After running the installer, environment variables are automatically added to `~/.bashrc`:

```bash
export SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx"
export LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib:$LD_LIBRARY_PATH"
export PATH="$HOME/.local/sherpa-onnx/bin:$PATH"
```

**For immediate effect:**
```bash
source ~/.bashrc
```

**For each terminal/build command:**
```bash
SHERPA_ONNX_DIR="/home/username/.local/sherpa-onnx" \\
LD_LIBRARY_PATH="/home/username/.local/sherpa-onnx/lib" \\
npm run tauri dev
```

## Testing the Fix

1. **Install Sherpa-ONNX (first time setup):**
   ```bash
   cd /mnt/data/dev/Ember
   npm run setup:audio:posix
   ```
   This installs Sherpa-ONNX binaries and adds environment variables to `~/.bashrc`.

2. **Build the app:**
   ```bash
   source ~/.bashrc  # Load environment variables
   npm install       # Install npm dependencies (if not done)
   npm run tauri dev
   ```

3. **Verify no infinite loop:**
   - Build should progress past step 580-583
   - Should see: "Using existing Sherpa-ONNX bindings (header unchanged)"
   - App window should launch successfully
   - No repeated "Bindings written" messages

## Verification Checklist

✅ Pre-built binaries install correctly to `$HOME/.local/sherpa-onnx`
✅ Environment variables added to `~/.bashrc`
✅ Cargo build finds Sherpa-ONNX libraries
✅ FFI bindings generate once and are reused
✅ No infinite loop in `npm run tauri dev`
✅ App builds and starts successfully
✅ Documentation updated with troubleshooting steps

## Architecture Benefits

This solution is **long-term and maintainable** because:

1. **No file patching:** Uses official pre-built releases instead of modifying source files
2. **CMake-independent:** Completely bypasses CMake build system for Sherpa-ONNX
3. **Timestamp-based caching:** Bindings regeneration only happens when truly needed
4. **Fallback available:** Users can still build from source if needed via `npm run setup:audio:build`
5. **Cross-platform:** Binary installer supports Linux/macOS, x86_64/aarch64
6. **Explicit dependencies:** Environment variables clearly document what's needed

## Alternative Solutions Considered

1. **CMake policy overrides:** Doesn't work - CMake 4.x completely removed < 3.5 support
2. **Fork dependencies:** Too much maintenance overhead
3. **Downgrade CMake:** Not scalable, conflicts with system packages
4. **Patch after download:** Fails due to FetchContent re-downloading (original broken approach)

## Next Steps

**For Development:**
1. Test wake-word detection with "Hey Ember"
2. Verify no false positives over 3+ minutes
3. Test speaker biometrics enrollment
4. Performance testing (CPU usage, latency)

**For Production:**
1. Ensure environment variables are set in deployment environment
2. Consider bundling Sherpa-ONNX libraries with the app
3. Test on clean systems without prior Sherpa-ONNX installation

## Related Issues

- BE-001C.5: Enable kws_real by default ✅ RESOLVED
- BE-001C.6: Update documentation ✅ RESOLVED
- CMake 4.x loop bug ✅ RESOLVED (this document)

## Credits

Solution implements a **pre-built binary distribution strategy** to avoid build-time
