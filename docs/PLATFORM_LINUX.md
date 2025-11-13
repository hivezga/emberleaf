# Linux Platform Guide (Emberleaf)

This guide covers Linux-specific setup, troubleshooting, and optimizations for Emberleaf, with special attention to **Arch Linux** and **Wayland** environments.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Display Backend Selection](#display-backend-selection)
- [Troubleshooting](#troubleshooting)
- [Arch Linux Notes](#arch-linux-notes)
- [Performance Tuning](#performance-tuning)

---

## System Requirements

### Required Packages

Emberleaf requires the following system libraries:

```bash
# Arch Linux
sudo pacman -S webkit2gtk-4.1 gtk3 pipewire xdg-desktop-portal

# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev pipewire

# Fedora
sudo dnf install webkit2gtk4.1-devel gtk3-devel pipewire
```

### Wayland-Specific Dependencies

For Wayland compositors (sway, Hyprland, KDE Wayland, GNOME Wayland):

```bash
# For wlroots-based compositors (sway, Hyprland, river, etc.)
sudo pacman -S xdg-desktop-portal-wlr xorg-xwayland

# For KDE Plasma (Wayland)
sudo pacman -S xdg-desktop-portal-kde xorg-xwayland

# For GNOME (Wayland)
sudo pacman -S xdg-desktop-portal-gnome xorg-xwayland
```

**Why XWayland?** Even if you prefer native Wayland, XWayland provides fallback support for apps that don't yet fully support Wayland.

---

## Installation

### 1. Clone and Setup

```bash
git clone https://github.com/lotusemberlabs/emberleaf.git
cd emberleaf
npm install
```

### 2. Install Audio Dependencies (Optional - Real KWS Mode)

If you want real wake-word detection with Sherpa-ONNX:

```bash
npm run setup:audio:posix
```

For stub mode (development without audio models):

```bash
# No setup needed, just run
npm run dev:stub
```

---

## Display Backend Selection

Emberleaf automatically detects your display environment but provides explicit control for stability.

### Environment Variable: `EMB_DISPLAY_BACKEND`

Controls which display backend to use:

- `auto` (default) - Auto-detect based on `WAYLAND_DISPLAY`
- `wayland` - Force native Wayland
- `x11` - Force X11/XWayland

### Usage

**Quick scripts:**

```bash
# Force X11 (most stable)
npm run dev:x11

# Force Wayland (experimental)
npm run dev:wayland

# Auto-detect (default)
npm run dev:stub
```

**Manual override:**

```bash
# Explicit Wayland
EMB_DISPLAY_BACKEND=wayland npm run dev:stub

# Explicit X11
EMB_DISPLAY_BACKEND=x11 npm run dev:stub
```

**Legacy variables (still supported):**

```bash
# Old style (still works)
DEV_WAYLAND=1 npm run dev:stub
DEV_X11=1 npm run dev:stub
```

### What Happens Under the Hood

When you start Emberleaf, the `display_backend` module:

1. Reads `EMB_DISPLAY_BACKEND` (or auto-detects)
2. Sets appropriate environment variables:
   - **Wayland mode:**
     - `WINIT_UNIX_BACKEND=wayland`
     - `GDK_BACKEND=wayland`
     - `WEBKIT_DISABLE_DMABUF_RENDERER=1` (GBM workaround)
     - `WEBKIT_DISABLE_COMPOSITING_MODE=1` (stability)
   - **X11 mode:**
     - `WINIT_UNIX_BACKEND=x11`
     - `GDK_BACKEND=x11`
     - Optionally keeps DMABUF disabled if under XWayland

---

## Troubleshooting

### Issue: App Opens Then Immediately Closes

**Symptoms:**
- `npm run dev:stub` opens a window that closes within 1-2 seconds
- Logs show: `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`

**Cause:** Wayland compositor incompatibility with WebKitGTK/GBM/DMABUF renderer.

**Solution:**

1. **Force X11 mode:**
   ```bash
   npm run dev:x11
   ```

2. **Check compositor logs:**
   ```bash
   # For sway
   journalctl --user -u sway -f

   # For Hyprland
   cat ~/.cache/hyprland/hyprland.log
   ```

3. **Verify XWayland is running:**
   ```bash
   pgrep -a Xwayland
   ```

---

### Issue: `Failed to load module "appmenu-gtk-module"`

**Symptoms:**
- Warning in console: `Failed to load module "appmenu-gtk-module"`

**Cause:** Optional GTK module for app menu integration (not required).

**Solution:** Safe to ignore. If you want to suppress it:

```bash
# Arch Linux
sudo pacman -S appmenu-gtk-module

# Ubuntu/Debian
sudo apt install appmenu-gtk-module
```

---

### Issue: GBM/DMABUF Buffer Errors

**Symptoms:**
- `Failed to create GBM buffer of size 800x600: Invalid argument`
- Rendering glitches or black window

**Cause:** GPU driver issues with DMA-BUF on some systems.

**Solution:** Emberleaf already disables DMABUF in Wayland mode, but you can force it globally:

```bash
export WEBKIT_DISABLE_DMABUF_RENDERER=1
npm run dev:stub
```

---

### Issue: Screen Sharing/Portals Not Working

**Symptoms:**
- Cannot share screen/windows
- Portal prompts don't appear

**Cause:** Missing or misconfigured XDG Desktop Portal.

**Solution:**

1. **Install portal for your compositor:**
   ```bash
   # wlroots (sway, Hyprland, river)
   sudo pacman -S xdg-desktop-portal-wlr

   # KDE
   sudo pacman -S xdg-desktop-portal-kde

   # GNOME
   sudo pacman -S xdg-desktop-portal-gnome
   ```

2. **Restart portal daemon:**
   ```bash
   systemctl --user restart xdg-desktop-portal
   ```

3. **Verify portals:**
   ```bash
   systemctl --user status xdg-desktop-portal
   ```

---

## Arch Linux Notes

### Package Installation (Complete)

```bash
# Core requirements
sudo pacman -S webkit2gtk-4.1 gtk3 base-devel

# Audio (PipeWire recommended)
sudo pacman -S pipewire pipewire-alsa pipewire-pulse wireplumber

# Wayland support (if using Wayland compositor)
sudo pacman -S xdg-desktop-portal-wlr xorg-xwayland

# Development tools
sudo pacman -S rust nodejs npm git
```

### Compositor-Specific Tips

#### Sway

Add to `~/.config/sway/config`:

```
# Enable XWayland for Tauri apps
xwayland enable

# Optional: Force specific apps to X11
for_window [app_id="emberleaf"] xwayland force
```

#### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
# Portal config for screen sharing
exec-once = dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP

# Force XWayland for specific apps (if needed)
windowrulev2 = xray on,class:^(emberleaf)$
```

---

## Performance Tuning

### CPU Provider for ONNX Models

Emberleaf uses CPU-based ONNX inference by default. For better performance:

```toml
# ~/.config/Emberleaf/config.toml
[kws]
provider = "cpu"  # Or "cuda" if you have NVIDIA GPU support
```

### Single Web Process (Lower RAM)

If experiencing high memory usage, edit `display_backend.rs` and uncomment:

```rust
env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");
```

**Trade-off:** Less isolation between tabs/webviews, but ~200MB lower RAM usage.

### Wayland Native vs XWayland

**Wayland Native:**
- Pros: Better Wayland integration, fractional scaling
- Cons: Less stable with WebKit, GPU issues

**XWayland (X11 on Wayland):**
- Pros: Most stable, best WebKit compatibility
- Cons: No native Wayland fractional scaling

**Recommendation:** Use `npm run dev:x11` for daily use; test `npm run dev:wayland` periodically as WebKitGTK improves.

---

## Debugging

### Enable Verbose Logs

```bash
# Rust logging
RUST_LOG=trace npm run dev:stub

# WebKit logging
G_MESSAGES_DEBUG=all npm run dev:stub

# Combined
RUST_LOG=debug G_MESSAGES_DEBUG=all npm run dev:stub
```

### Check Display Backend at Runtime

Look for these log lines on startup:

```
=== Configuring Wayland Display Backend ===
  WINIT_UNIX_BACKEND=wayland
  GDK_BACKEND=wayland
  WEBKIT_DISABLE_DMABUF_RENDERER=1
========================================
```

---

## Known Issues

### Arch + Wayland + NVIDIA

**Issue:** Black window or immediate crash.

**Workaround:**
```bash
# Force X11 mode
export EMB_DISPLAY_BACKEND=x11
npm run dev:stub
```

**Long-term:** NVIDIA Wayland support is improving with newer drivers (>= 545.x).

### KDE Plasma 6 + WebKitGTK

**Issue:** Compositor crashes or protocol errors.

**Workaround:**
```bash
# Use X11 session instead of Wayland session
# OR
npm run dev:x11
```

---

## Contributing

Found a Linux-specific issue? Please report it with:

- Distro and version (`cat /etc/os-release`)
- Compositor (`echo $XDG_SESSION_TYPE` and `$XDG_CURRENT_DESKTOP`)
- GPU (`lspci | grep VGA`)
- WebKitGTK version (`pacman -Q webkit2gtk-4.1`)
- Logs (`RUST_LOG=debug npm run dev:stub 2>&1 | tee ember.log`)

---

## See Also

- [BE-001: KWS Implementation](BE-001-kws.md)
- [BE-009: Linux WebView Stability](https://github.com/lotusemberlabs/emberleaf/issues/9)
- [Main README](../README.md)
