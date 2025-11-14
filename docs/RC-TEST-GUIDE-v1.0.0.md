# Emberleaf v1.0.0-rc1 — Release Candidate Test Guide

## Overview

This guide provides step-by-step testing procedures for v1.0.0-rc1 to validate packaging, installation, and core functionality before final v1.0.0 release.

**Test Duration:** ~10-12 minutes per distribution  
**Required:** One Ubuntu LTS + one rolling distro (Arch/Fedora)

---

## Pre-Test Setup

### Download Artifacts

```bash
# Download release artifacts
gh release download v1.0.0-rc1 -D ~/emberleaf-rc1
cd ~/emberleaf-rc1

# Verify checksums
sha256sum --check SHASUMS256.txt
# Expected: all OK
```

---

## 1. AppImage Testing (Portable)

**Platforms:** Arch, Fedora, openSUSE, any modern Linux

### Installation

```bash
# Make executable
chmod +x Emberleaf-*.AppImage

# Run
./Emberleaf-*.AppImage
```

### Verification

- [ ] **App Launch:** Window opens without console errors
- [ ] **Icon:** Appears in task switcher/window list
- [ ] **Dependencies:** No "missing library" errors
- [ ] **Desktop Integration:** Works without system installation

**Expected:** Clean launch, no dependency complaints

---

## 2. Debian Package Testing (.deb)

**Platforms:** Ubuntu 22.04/24.04, Debian 12+

### Installation

```bash
# Install (requires sudo)
sudo apt install ./emberleaf_*_amd64.deb

# Expected output: dependency resolution + successful install
```

### Verification

- [ ] **Desktop Entry:** Check `/usr/share/applications/emberleaf.desktop` exists
- [ ] **Launcher:** Found in application menu (search "Emberleaf")
- [ ] **Icon:** Proper icon displays in launcher
- [ ] **Permissions:** No warnings about missing files

### Uninstallation Test

```bash
# Remove package
sudo apt remove emberleaf

# Verify clean removal
ls /usr/share/applications/ | grep emberleaf
# Expected: no output (clean removal)
```

**Expected:** No debris left behind

---

## 3. Functional Testing (Both Packages)

Run these tests after installing via AppImage OR .deb:

### 3.1 Preflight UI

```bash
# Launch Emberleaf
emberleaf  # (if .deb) or ./Emberleaf-*.AppImage
```

- [ ] **Preflight shows:** PASS/WARN/FAIL icons for each check
- [ ] **Re-run button:** Works (refreshes checks)
- [ ] **Links:** "Fix guide" links open browser/help

### 3.2 Simple Mode - Microphone Test

- [ ] Navigate to **Simple → Check my microphone**
- [ ] **Auto-probe:** Runs and selects default device
- [ ] **Meter:** Moves when speaking (visual feedback)
- [ ] **Next:** Only enables after RMS > 0.05 for ≥1s

**Expected:** Responsive meter, clear visual confirmation

### 3.3 KWS Real (Wake Word)

#### Enable Model

- [ ] Go to **Advanced → Wake Word → KWS Settings**
- [ ] Click **Enable Model** (ES recommended)
- [ ] Progress bar shows download
- [ ] Model status: "Ready" after download

#### Test Wake Word

- [ ] Click **Test wake word**
- [ ] Say "hey ember" clearly
- [ ] **Result:** Green pass chip appears
- [ ] Check browser console: no errors

**Expected:** Detection within ~500ms, no console errors

### 3.4 Device Switch & Restart

- [ ] **Advanced → Audio Device:** Change input to different mic
- [ ] **Banner:** "Requires restart" appears
- [ ] Click **Restart Audio**
- [ ] Meter moves on new device

**Expected:** Clean device switch, no crashes

### 3.5 Device Loss Handling

- [ ] **While audio running:** Physically disconnect microphone
- [ ] **Banner:** "Device lost" notification appears
- [ ] **Fallback:** Auto-switches to available device OR shows selection UI

**Expected:** Graceful fallback, no crash

### 3.6 Mic Monitor Guard

- [ ] Toggle **Mic Monitor** ON
- [ ] If input == output: **Guard UI** shows warning
- [ ] Otherwise: Monitor works without feedback loop warning

**Expected:** Feedback loop protection active

---

## 4. Cross-Platform Matrix

Test at least **one** from each category:

| Distribution | Package Type | Status |
|--------------|--------------|--------|
| Ubuntu 22.04 LTS | .deb | ⬜ |
| Ubuntu 24.04 LTS | .deb | ⬜ |
| Arch Linux | AppImage | ⬜ |
| Fedora | AppImage | ⬜ |
| openSUSE Tumbleweed | AppImage | ⬜ |

---

## 5. Known Issues (Expected)

These are **acceptable** for RC1 (track for v1.0.0):

- Model download slow on first run (no caching yet)
- No wizard polish (ONB-002 pending)
- No speaker biometrics (KWS-REAL-002 pending)

---

## 6. Failure Criteria (Block v1.0.0)

Report these immediately if encountered:

- [ ] **App crashes** on launch
- [ ] **Missing dependencies** not declared in .deb control file
- [ ] **Checksum mismatch** on any artifact
- [ ] **KWS never detects** "hey ember" after model download
- [ ] **Mic permissions** denied without system prompt
- [ ] **Device switch crashes** app

---

## 7. Report Template

Copy this template when reporting test results:

```markdown
## Test Report: v1.0.0-rc1

**Platform:** [OS + version]  
**Package:** [AppImage / .deb]  
**Date:** [YYYY-MM-DD]

### Results

- [ ] Checksum verification: PASS / FAIL
- [ ] Installation: PASS / FAIL
- [ ] Preflight UI: PASS / FAIL
- [ ] Mic test: PASS / FAIL
- [ ] KWS detection: PASS / FAIL
- [ ] Device switch: PASS / FAIL
- [ ] Device loss: PASS / FAIL
- [ ] Mic monitor: PASS / FAIL

### Notes

[Any observations, warnings, or issues]

### Recommendation

- [ ] **SHIP** v1.0.0 (all tests pass)
- [ ] **RC2** needed (critical issues found)
```

---

## 8. Next Steps

### If All Tests Pass

```bash
# Cut v1.0.0 final
git tag v1.0.0
git push origin v1.0.0

# Publish draft release (edit notes)
gh release edit v1.0.0-rc1 --draft=false
```

### If Issues Found

- Document in GitHub issue linking v1.0.0 milestone
- Fix critical blockers
- Cut v1.0.0-rc2 with fixes

---

**Questions?** Check [INSTALL-Linux.md](./INSTALL-Linux.md) or [PACKAGING-REL-002.md](./PACKAGING-REL-002.md)
