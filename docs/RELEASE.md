# Release Process — Emberleaf

**Version:** 0.1.0 (Pre-release)
**Last Updated:** 2025-01-13

---

## Table of Contents

1. [Overview](#overview)
2. [Build Process](#build-process)
3. [Artifact Verification](#artifact-verification)
4. [CI/CD Pipeline](#cicd-pipeline)
5. [Creating a Release](#creating-a-release)
6. [Installation](#installation)

---

## Overview

Emberleaf uses a **tag-based release workflow**:

- **Development**: All commits to `main` run lint/test/check
- **Releases**: Tags matching `v*` trigger artifact builds
- **Distribution**: AppImage (portable) + .deb (Debian/Ubuntu)

All releases include **SHA-256 checksums** for integrity verification.

---

## Build Process

### Prerequisites

**System Dependencies** (Ubuntu/Debian):
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  libasound2-dev \
  libpulse-dev
```

**Build Tools:**
- Node.js 20+
- Rust 1.75+ (stable)
- npm 10+

### Local Build

**1. Install dependencies:**
```bash
npm ci
```

**2. Build frontend:**
```bash
npm run build
```

**3. Package artifacts:**
```bash
npm run package:linux
```

**Output:**
```
dist/
├── Emberleaf-v0.9.0-x86_64.AppImage
└── emberleaf_0.9.0_amd64.deb
```

### Build Variants

| Command | Description | Sherpa-ONNX |
|---------|-------------|-------------|
| `npm run package:linux` | Standard build | Stub (no real KWS) |
| `npm run build:real` | Full AI build | Real Sherpa-ONNX |

---

## Artifact Verification

Every release includes `SHASUMS256.txt` with SHA-256 checksums.

### Verify Downloaded Artifacts

**1. Download files:**
```bash
wget https://github.com/your-org/emberleaf/releases/download/v0.9.0/Emberleaf-v0.9.0-x86_64.AppImage
wget https://github.com/your-org/emberleaf/releases/download/v0.9.0/SHASUMS256.txt
```

**2. Verify integrity:**
```bash
sha256sum --check SHASUMS256.txt
```

**Expected output:**
```
Emberleaf-v0.9.0-x86_64.AppImage: OK
emberleaf_0.9.0_amd64.deb: OK
```

**3. If verification fails:**
```
Emberleaf-v0.9.0-x86_64.AppImage: FAILED
```

**Action:** Do NOT install. File may be corrupted or tampered. Re-download or report issue.

### Manual Checksum Verification

If `SHASUMS256.txt` is unavailable:

```bash
# Generate checksum
sha256sum Emberleaf-v0.9.0-x86_64.AppImage

# Compare with checksum from release notes or GitHub UI
```

---

## CI/CD Pipeline

Automated via **GitHub Actions** (`.github/workflows/ci.yml`).

### Jobs Overview

| Job | Trigger | Purpose |
|-----|---------|---------|
| **lint** | PR + Push | TypeScript, Rust fmt, Clippy |
| **test-web** | PR + Push | Frontend tests (Vitest) |
| **check-rust** | PR + Push | Cargo check (all features) |
| **audit-rust** | PR + Push | Cargo audit (security) |
| **audit-npm** | PR + Push | npm audit (security) |
| **scan-secrets** | PR + Push | Detect leaked secrets |
| **check-xss** | PR + Push | XSS safety (no `dangerouslySetInnerHTML`) |
| **build-linux** | Tags (`v*`) | Build AppImage + .deb + checksums |

### Workflow Steps (build-linux)

1. **Checkout code** (`actions/checkout@v4`)
2. **Setup Node.js** (v20 with npm cache)
3. **Setup Rust** (stable toolchain)
4. **Cache Rust dependencies** (Cargo registry + target/)
5. **Install system dependencies** (WebKit, GTK, PulseAudio)
6. **Install npm dependencies** (`npm ci`)
7. **Build frontend** (`npm run build`)
8. **Package Linux artifacts** (`npm run package:linux`)
9. **Generate SHA-256 checksums**
10. **Verify checksums** (`sha256sum --check`)
11. **Upload artifacts** (30-day retention)
12. **Create draft release** (GitHub Releases)

### Cache Strategy

**npm:**
- Cache key: `${{ runner.os }}-npm-${{ hashFiles('**/package-lock.json') }}`
- Speedup: ~2-3 minutes

**Cargo:**
- Cache key: `${{ runner.os }}-cargo-release-${{ hashFiles('**/Cargo.lock') }}`
- Cached paths: `~/.cargo/registry/`, `src-tauri/target/`
- Speedup: ~5-8 minutes

**Average CI time:**
- **PR (no build):** ~3-5 minutes
- **Tag (with build):** ~8-12 minutes (cached)

---

## Creating a Release

### 1. Prepare Release

**Update version:**
```bash
# package.json
"version": "0.9.0"

# src-tauri/Cargo.toml
version = "0.9.0"

# src-tauri/tauri.conf.json
"version": "0.9.0"
```

**Update CHANGELOG.md:**
```markdown
## [0.9.0] - 2025-01-15

### Added
- First-run onboarding wizard (ONB-001)
- Preflight system check UI (REL-001-UI)

### Security
- Input validation on all Tauri commands (SEC-001)
- Config file permissions hardened to 0600
- CSP tightened (no inline scripts in prod)

### CI/CD
- Automated lint, test, and security audits
- AppImage + .deb packaging with SHA-256 checksums
```

**Commit changes:**
```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "chore: Bump version to 0.9.0"
git push origin main
```

### 2. Create Tag

**Tag the release:**
```bash
git tag v0.9.0
git push origin v0.9.0
```

**CI automatically:**
1. Runs all checks (lint, test, audit)
2. Builds AppImage + .deb
3. Generates checksums
4. Creates **draft release** on GitHub

### 3. Publish Release

1. Go to **GitHub Releases** → **Drafts**
2. Review auto-generated release notes
3. Edit if needed (add highlights, breaking changes)
4. **Publish release** (no longer draft)

### 4. Post-Release

**Announce:**
- Update README.md with latest version
- Announce on Discord/Twitter/blog

**Monitor:**
- GitHub Discussions for issues
- CI status for any failures

---

## Installation

### AppImage (Portable)

**1. Download:**
```bash
wget https://github.com/your-org/emberleaf/releases/latest/download/Emberleaf-x86_64.AppImage
```

**2. Verify checksum** (see [Artifact Verification](#artifact-verification))

**3. Make executable:**
```bash
chmod +x Emberleaf-x86_64.AppImage
```

**4. Run:**
```bash
./Emberleaf-x86_64.AppImage
```

**Optional: Desktop integration**
```bash
# Install to ~/.local/bin
mv Emberleaf-x86_64.AppImage ~/.local/bin/emberleaf

# Add to PATH (if not already)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Debian Package (.deb)

**1. Download:**
```bash
wget https://github.com/your-org/emberleaf/releases/latest/download/emberleaf_0.9.0_amd64.deb
```

**2. Verify checksum** (see [Artifact Verification](#artifact-verification))

**3. Install:**
```bash
sudo dpkg -i emberleaf_0.9.0_amd64.deb

# Fix any missing dependencies
sudo apt-get install -f
```

**4. Run:**
```bash
emberleaf
```

**Uninstall:**
```bash
sudo apt-get remove emberleaf
```

### System Requirements

- **OS:** Ubuntu 22.04+, Debian 12+, Fedora 38+, Arch Linux (current)
- **Display:** X11 or Wayland
- **Audio:** PulseAudio or PipeWire
- **RAM:** 512 MB minimum, 1 GB recommended
- **Disk:** 200 MB for app + models

**Note:** See `docs/INSTALL-Linux.md` for distro-specific setup.

---

## Troubleshooting

### Build Failures

**Error:** `libwebkit2gtk-4.1-dev not found`

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel

# Arch
sudo pacman -S webkit2gtk-4.1
```

**Error:** `cargo audit` fails on known vulnerability

**Solution:**
```bash
# Check advisory details
cargo audit

# Update dependencies
cargo update

# If no fix available, suppress specific advisory (documented reason required)
```

### CI Failures

**Symptom:** Tag push doesn't trigger build

**Check:**
1. Tag format: Must match `v*` (e.g., `v0.9.0`)
2. GitHub Actions enabled in repo settings
3. Workflow file syntax valid (`yamllint .github/workflows/ci.yml`)

**Symptom:** Checksum verification fails

**Cause:** Artifact corruption during build or upload

**Solution:**
1. Re-run workflow (GitHub Actions → Re-run failed jobs)
2. Check build logs for errors
3. Verify file sizes match expected

---

## Future Improvements

**Planned for v1.0:**
- [ ] GPG-signed releases (`.sig` files)
- [ ] Reproducible builds (bit-for-bit identical)
- [ ] SBOM (Software Bill of Materials) generation
- [ ] Flatpak/Snap packaging
- [ ] macOS + Windows builds

**Post-v1.0:**
- [ ] Auto-update mechanism (Tauri updater)
- [ ] Delta updates (AppImage delta patches)
- [ ] Signed commits enforcement

---

**Last Updated:** 2025-01-13
**Next Review:** Before v1.0 release
