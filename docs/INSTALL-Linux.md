# Emberleaf — Linux Installation (AppImage & .deb)

## Quick Start

- **AppImage (portable):**
  ```bash
  chmod +x Emberleaf-x86_64.AppImage
  ./Emberleaf-x86_64.AppImage
  ```

- **Debian/Ubuntu (.deb):**
  ```bash
  sudo apt install ./emberleaf_VERSION_amd64.deb
  emberleaf
  ```

## Verify Integrity

```bash
sha256sum --check SHASUMS256.txt
```

## Requirements

* WebKit2GTK, GTK3, PipeWire/PulseAudio, portals (Wayland)
* See `docs/DECISIONS.md` for rationale (AppImage vs Flatpak).

## Troubleshooting

* Missing deps → run preflight (first-run) and see remediation
* Wayland/X11 notes
* Audio device permissions
