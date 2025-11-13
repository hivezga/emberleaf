# Wayland Display Issue - Fixed

## Problem

The app crashed on KDE Plasma Wayland with:
```
Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display.
```

## Root Cause

**webkit2gtk-4.1 (v2.50.1) + GTK4 (v4.20.2) + KDE Plasma Wayland** have protocol incompatibility issues. When creating the Tauri window, GTK sends invalid Wayland protocol messages, causing kwin_wayland to drop the connection.

System logs show:
```
kwin_wayland_wrapper[1214]: error in client communication (pid 110228)
```

## Solution: Force X11 Backend

Use `GDK_BACKEND=x11` to run the app via Xwayland (X11 compatibility layer) instead of native Wayland.

### Quick Start

```bash
# Using the dev script (recommended)
bash scripts/dev.sh

# Or with npm
npm run tauri:dev        # info logging
npm run tauri:debug      # debug logging
npm run tauri:trace      # trace logging (very verbose)

# Manual launch with custom logging
GDK_BACKEND=x11 RUST_LOG=debug npm run tauri dev
```

### Development Script

The `scripts/dev.sh` script automatically configures:
- ✅ X11 backend (`GDK_BACKEND=x11`)
- ✅ Sherpa-ONNX environment variables
- ✅ Rust logging levels
- ✅ Pre-flight checks for models and libraries

**Usage:**
```bash
# Default (info level)
bash scripts/dev.sh

# With specific log level
bash scripts/dev.sh debug    # Debug logging
bash scripts/dev.sh trace    # Trace logging (all modules)
bash scripts/dev.sh warn     # Warnings only
bash scripts/dev.sh error    # Errors only
```

### Rust Logging Levels

Set `RUST_LOG` to control logging verbosity:

| Level   | Usage | Output |
|---------|-------|--------|
| `error` | Production | Only critical errors |
| `warn`  | Staging | Warnings + errors |
| `info`  | **Default** | High-level events (startup, detections) |
| `debug` | Development | Detailed debug info |
| `trace` | Debugging issues | All internal events (very verbose) |

### Module-Specific Logging

For targeted debugging, set `RUST_LOG` to specific modules:

```bash
# KWS debugging only
export RUST_LOG="emberleaf::audio::kws=trace,emberleaf=info"
bash scripts/dev.sh

# Audio pipeline debugging
export RUST_LOG="emberleaf::audio=debug"
npm run tauri dev

# Biometrics debugging
export RUST_LOG="emberleaf::voice::biometrics=trace"
npm run tauri dev

# Multiple modules
export RUST_LOG="emberleaf::audio::kws=trace,emberleaf::audio::vad=debug,tauri=warn"
npm run tauri dev
```

## Alternative Solutions

### 1. Wait for Upstream Fixes (Long-term)

Update webkit2gtk and KDE Plasma when fixes are released:
```bash
sudo pacman -Syu webkit2gtk-4.1 kwin plasma-workspace
```

### 2. Use GNOME/Sway (If Available)

Wayland works better on GNOME Wayland and Sway compositors with webkit2gtk-4.1.

### 3. Permanent X11 Session

Switch to X11 session instead of Wayland session at login.

## Verification

After launching with `scripts/dev.sh`, verify:

1. **No Wayland crash** - App window opens successfully
2. **X11 backend active** - Check with:
   ```bash
   ps aux | grep ember
   # Should show process running without Wayland errors
   ```

3. **Logs visible** - Console shows Rust logs:
   ```
   [INFO emberleaf] Emberleaf starting...
   [INFO emberleaf::paths] Application directories initialized
   [INFO emberleaf] Configuration loaded
   ```

## Files Changed

- **NEW:** `scripts/dev.sh` - Development launch script with X11 + logging
- **UPDATED:** `package.json` - Added `tauri:dev`, `tauri:debug`, `tauri:trace` scripts
- **FIXED:** `src-tauri/src/audio/kws/real.rs` - Model filename corrections

## Related Issues

- Infinite CMake loop: See `FIXED_CMAKE_LOOP.md`
- Model setup: See `docs/DEV_SETUP_SHERPA.md`
- KWS documentation: See `docs/BE-001-kws.md`

## Testing Checklist

- [x] App launches without Wayland crash
- [x] Window displays correctly via X11
- [x] KWS models load successfully
- [x] Rust logging output visible
- [x] No "Error 71" in system logs
- [ ] Wake-word detection functional (requires microphone)
- [ ] Speaker biometrics enrollment (requires microphone)
