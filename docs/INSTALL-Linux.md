# Emberleaf Linux Installation Guide

Complete installation guide for running Emberleaf on Linux distributions.

---

## Quick Start

### Method 1: AppImage (Universal)

**Best for:** All distributions, no installation required

```bash
# Download
wget https://github.com/lotusemberlabs/emberleaf/releases/latest/download/Emberleaf-x86_64.AppImage

# Make executable
chmod +x Emberleaf-x86_64.AppImage

# Run
./Emberleaf-x86_64.AppImage
```

**Optional:** Integrate with system menu using [AppImageLauncher](https://github.com/TheAssassin/AppImageLauncher)

### Method 2: .deb Package (Debian/Ubuntu)

**Best for:** Debian, Ubuntu, Linux Mint, Pop!_OS

```bash
# Download
wget https://github.com/lotusemberlabs/emberleaf/releases/latest/download/emberleaf_*_amd64.deb

# Install with dependencies
sudo apt install ./emberleaf_*_amd64.deb
```

---

## System Requirements

### Minimum

- **OS:** Linux kernel 5.10+ (systemd recommended)
- **Architecture:** x86_64 (AMD64)
- **RAM:** 2GB minimum, 4GB recommended
- **Disk:** 500MB for app + 2GB for models (Real KWS mode)

### Required Dependencies

Emberleaf will check these on first run and provide guided fixes if missing:

1. **Audio Server:** PipeWire (recommended) or PulseAudio
2. **WebKit:** WebKit2GTK 2.40+
3. **Desktop Portal:** xdg-desktop-portal (for Wayland)
4. **Microphone Access:** User in `audio` group (some distros)

---

## Per-Distribution Setup

### Arch Linux / Manjaro

```bash
# Install dependencies
sudo pacman -S pipewire pipewire-pulse webkit2gtk xdg-desktop-portal-gtk

# Enable audio services
systemctl --user enable --now pipewire pipewire-pulse

# Download and run AppImage (see Quick Start)
```

**Optional:** Install from AUR (coming soon)

### Ubuntu / Debian / Linux Mint

```bash
# Install dependencies
sudo apt update
sudo apt install pipewire pipewire-pulse libwebkit2gtk-4.0-37 xdg-desktop-portal-gtk

# Enable PipeWire (Ubuntu 22.10+)
systemctl --user enable --now pipewire pipewire-pulse

# Install .deb package (see Quick Start)
```

**Ubuntu 20.04 / Debian 11:** Use PulseAudio instead:

```bash
sudo apt install pulseaudio libwebkit2gtk-4.0-37
```

### Fedora / RHEL / Rocky Linux

```bash
# Install dependencies
sudo dnf install pipewire pipewire-pulseaudio webkit2gtk3 xdg-desktop-portal-gtk

# Enable audio services (usually enabled by default)
systemctl --user enable --now pipewire pipewire-pulse

# Download and run AppImage (see Quick Start)
```

### openSUSE

```bash
# Install dependencies
sudo zypper install pipewire pipewire-pulseaudio webkit2gtk3 xdg-desktop-portal-gtk

# Enable audio services
systemctl --user enable --now pipewire pipewire-pulse

# Download and run AppImage (see Quick Start)
```

### NixOS

Add to `configuration.nix`:

```nix
{
  environment.systemPackages = with pkgs; [
    pipewire
    wireplumber
    webkitgtk
    xdg-desktop-portal-gtk
  ];

  services.pipewire = {
    enable = true;
    pulse.enable = true;
  };
}
```

Then rebuild:

```bash
sudo nixos-rebuild switch
```

**Note:** AppImage may require `appimage-run` wrapper on NixOS.

---

## Troubleshooting

### Preflight Check Failures

Emberleaf runs preflight checks on first launch. If you see errors:

#### "No audio server detected"

**Cause:** Neither PipeWire nor PulseAudio is running.

**Fix:**

```bash
# Check if PipeWire is installed
systemctl --user status pipewire

# If not running, start it
systemctl --user enable --now pipewire pipewire-pulse

# Or use PulseAudio (older distros)
systemctl --user enable --now pulseaudio
```

#### "WebKit2GTK not found"

**Cause:** Missing WebKit library.

**Fix:**

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.0-37

# Arch
sudo pacman -S webkit2gtk

# Fedora
sudo dnf install webkit2gtk3
```

#### "XDG Desktop Portal not running"

**Cause:** Portal service not started (Wayland only; X11 can ignore this warning).

**Fix:**

```bash
# Start portal service
systemctl --user start xdg-desktop-portal

# Auto-start on login
systemctl --user enable xdg-desktop-portal

# Or reboot
```

#### "Cannot access audio devices" / "No microphone detected"

**Cause:** Permissions issue or no mic connected.

**Fix:**

```bash
# Add user to audio group (some distros)
sudo usermod -aG audio $USER

# Log out and back in for group change to apply

# Test microphone
arecord -l   # Should list input devices

# If using Flatpak AppImage, grant permissions:
flatpak override --user --filesystem=xdg-run/pipewire-0
```

### Wayland-Specific Issues

If the app doesn't launch or has rendering issues on Wayland:

```bash
# Force X11 backend (temporary)
GDK_BACKEND=x11 ./Emberleaf-x86_64.AppImage

# Or install compositor-specific portal
# GNOME:
sudo apt install xdg-desktop-portal-gnome

# KDE Plasma:
sudo apt install xdg-desktop-portal-kde

# wlroots (Sway, Hyprland):
sudo apt install xdg-desktop-portal-wlr
```

### PipeWire vs. PulseAudio

**Modern distros (2022+):** Use PipeWire (better latency, Bluetooth support)

**Older distros:** PulseAudio is fine

**Switching to PipeWire:**

```bash
# Debian/Ubuntu
sudo apt install pipewire pipewire-pulse wireplumber
systemctl --user --now enable pipewire pipewire-pulse
systemctl --user --now disable pulseaudio

# Verify
pactl info | grep "Server Name"
# Should show: PulseAudio (on PipeWire)
```

---

## Uninstallation

### AppImage

Simply delete the file:

```bash
rm Emberleaf-x86_64.AppImage
```

User data stored in:

- `~/.local/share/Emberleaf/` (models, profiles)
- `~/.config/emberleaf/` (settings)

### .deb Package

```bash
sudo apt remove emberleaf
```

To also remove user data:

```bash
rm -rf ~/.local/share/Emberleaf ~/.config/emberleaf
```

---

## Advanced: Building from Source

See [DEVELOPMENT.md](./DEVELOPMENT.md) for build instructions.

**Quick build:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js 18+
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install nodejs

# Clone and build
git clone https://github.com/lotusemberlabs/emberleaf.git
cd emberleaf
npm install
npm run tauri build
```

---

## Support

- **GitHub Issues:** https://github.com/lotusemberlabs/emberleaf/issues
- **Preflight Diagnostics:** Run with `--verbose` flag for detailed logs
- **Community:** (Discord/forum link TBD)

---

## FAQ

**Q: Can I use Emberleaf without internet?**
A: Yes! Emberleaf is fully offline after initial download.

**Q: Which is better: AppImage or .deb?**
A: AppImage for portability, .deb for integration and auto-updates.

**Q: Does Emberleaf work on ARM (Raspberry Pi)?**
A: Not yet. x86_64 only for v1.0. ARM support planned for v1.1.

**Q: Why does first launch take long?**
A: Initial preflight checks + model downloads (Real KWS mode). Subsequent launches are instant.

---

Last updated: 2025-11-12
Version: v1.0
