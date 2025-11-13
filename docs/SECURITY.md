# Security Policy — Emberleaf

**Version:** 0.1.0 (Pre-release)
**Last Updated:** 2025-01-13
**Scope:** Linux desktop builds (AppImage, .deb)

---

## Table of Contents

1. [Overview](#overview)
2. [Content Security Policy (CSP)](#content-security-policy-csp)
3. [Tauri Allowlist & Protocol Hardening](#tauri-allowlist--protocol-hardening)
4. [Command Input Validation](#command-input-validation)
5. [Dependency & Build Integrity](#dependency--build-integrity)
6. [Config & Model Integrity](#config--model-integrity)
7. [Renderer Safety (XSS Prevention)](#renderer-safety-xss-prevention)
8. [Known Non-Goals](#known-non-goals)
9. [Reporting Security Issues](#reporting-security-issues)

---

## Overview

Emberleaf is a **privacy-first, fully offline voice assistant**. Security is foundational:

- **No telemetry**: Zero data leaves your device
- **No cloud services**: All processing happens locally
- **Open source**: Auditable codebase
- **Desktop-only**: No web deployment surface

This document describes the security measures implemented for v1.0 readiness (**SEC-001**).

---

## Content Security Policy (CSP)

### Production CSP

```
default-src 'self';
img-src 'self' blob: data:;
media-src 'self' blob:;
script-src 'self';
style-src 'self' 'unsafe-inline';
connect-src 'self';
font-src 'self' data:;
object-src 'none';
base-uri 'self';
form-action 'none';
frame-ancestors 'none'
```

**Rationale:**
- **`script-src 'self'`**: Blocks inline `<script>` tags and `eval()` in production
- **`style-src 'self' 'unsafe-inline'`**: Allows Tailwind CSS utility classes (required for Vite/React)
- **`connect-src 'self'`**: No external API calls (offline-first)
- **`object-src 'none'`**: Blocks Flash, Java applets
- **`form-action 'none'`**: No form submissions to external URLs
- **`frame-ancestors 'none'`**: Prevents embedding in iframes (clickjacking protection)

### Development CSP

Development mode (`npm run dev`) uses a relaxed CSP to support Vite HMR:

```
script-src 'self' 'unsafe-inline' 'unsafe-eval';
connect-src 'self' http://localhost:1420 ws://localhost:1420
```

**Note:** Devtools are **disabled** in production builds via Cargo feature flags.

---

## Tauri Allowlist & Protocol Hardening

### Permissions

**Restricted APIs** (configured in `src-tauri/tauri.conf.json`):

- **Shell**: `open: false`, `scope: []` — No arbitrary command execution
- **Dialog**: Limited to file picker (scoped to user dirs)
- **FS**: Read/write restricted to app config directory
- **HTTP**: Disabled (no network calls)

### Protocol Isolation

- **`withGlobalTauri: false`**: Tauri API not exposed to global scope
- **`enableRemoteUrlIpcProxy: false`**: Prevents IPC proxying to remote URLs
- **`freezePrototype: true`**: Prevents prototype pollution attacks

### Devtools

- **Development**: Enabled via `devtools` feature flag
- **Production**: **Disabled** (compiled out)

```toml
# Cargo.toml
[profile.release]
# devtools feature NOT included in release builds
```

---

## Command Input Validation

All Tauri commands implement **centralized validation** via `src-tauri/src/validation.rs`.

### Validation Rules

| Input Type | Constraint | Rationale |
|------------|-----------|-----------|
| **VAD Threshold** | `0.0 ≤ threshold ≤ 1.0` | Prevent invalid sensitivity values |
| **Audio Duration** | `10ms ≤ duration ≤ 5000ms` | Prevent DoS via excessive processing |
| **Frequency** | `50Hz ≤ freq ≤ 4000Hz` | Voice range bounds |
| **Device Name** | Max 256 chars, no control chars | Prevent injection/overflow |
| **File Paths** | Canonicalized, within allowed dirs | Prevent path traversal |
| **Profile Names** | Alphanumeric + `_-`, max 64 chars | Filesystem-safe identifiers |

### Example: VAD Threshold Validation

```rust
// SEC-001: Validate input range
let validated_threshold = validation::validate_vad_threshold(threshold)
    .map_err(|e| e.to_string())?;
```

### Path Traversal Prevention

File paths are:
1. Canonicalized (resolve `..` and symlinks)
2. Checked against allowed base directory (app config dir)
3. Rejected if outside scope

```rust
validate_path("/etc/passwd", &app_config_dir) // ❌ Rejected
validate_path("../../../etc/passwd", &app_config_dir) // ❌ Rejected
validate_path("config/settings.toml", &app_config_dir) // ✅ Allowed
```

### Unit Tests

All validators have comprehensive test coverage:

```bash
cargo test --package ember --lib validation
```

---

## Dependency & Build Integrity

### Automated Auditing (CI)

**Rust Dependencies:**
```bash
cargo audit --deny warnings
```

**npm Dependencies:**
```bash
npm audit --audit-level=high
```

**CI Jobs:**
- `audit-rust` — Runs `cargo audit` on every PR
- `audit-npm` — Runs `npm audit` on every PR
- **Non-fatal**: Currently informational; will become blocking in v1.0

### Release Artifact Verification

All tagged releases (`v*`) include:

1. **AppImage** (`.AppImage`)
2. **Debian Package** (`.deb`)
3. **SHA-256 Checksums** (`SHASUMS256.txt`)

**Verification Steps:**

1. Download artifacts and checksums
2. Verify integrity:

```bash
sha256sum --check SHASUMS256.txt
```

3. Expected output:
```
Emberleaf-v0.9.0-x86_64.AppImage: OK
emberleaf_0.9.0_amd64.deb: OK
```

**Future:** GPG signatures (`.sig` files) planned for v1.0.

---

## Config & Model Integrity

### Configuration File Permissions

Config files (`config.toml`, voice profiles) are written with **owner-only permissions**:

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&config_path)?.permissions();
    perms.set_mode(0o600); // Owner read/write only
    fs::set_permissions(&config_path, perms)?;
}
```

**Result:**
```bash
-rw------- 1 user user 1234 config.toml
```

### Model Checksum Verification

ONNX models (Sherpa-ONNX KWS, voice embeddings) are:
1. Downloaded from trusted sources
2. Verified against SHA-256 checksums in `registry.rs`
3. **Rejected** if checksum mismatch

**Example:**
```rust
let expected_hash = "abc123...";
let actual_hash = sha2::Sha256::digest(&model_bytes);
if expected_hash != format!("{:x}", actual_hash) {
    return Err("Model integrity check failed");
}
```

### No Secrets in Repository

**CI Check:**
```bash
# Scans for common secret patterns
grep -rE "(api[_-]?key|password|secret|token|bearer)" src/
grep -rE "AKIA[0-9A-Z]{16}" .  # AWS keys
grep -rE "ghp_[0-9a-zA-Z]{36}" .  # GitHub tokens
```

**Result:** Non-fatal warning in CI; manual review required.

---

## Renderer Safety (XSS Prevention)

### React Security

- **All user content rendered via React** (automatic escaping)
- **No `dangerouslySetInnerHTML`** (enforced by CI check)
- **Toast notifications**: Literal strings only (no user-provided HTML)

**CI Enforcement:**
```bash
# Fails build if dangerouslySetInnerHTML found
grep -r "dangerouslySetInnerHTML" src/ --include="*.tsx"
```

### External Links

All external links use:
```tsx
<a href="..." target="_blank" rel="noopener noreferrer">
```

**Purpose:** Prevents `window.opener` hijacking.

---

## Known Non-Goals

Emberleaf **does not** address the following (out of scope for desktop offline app):

1. **Network security**: No server, no TLS, no API keys
2. **Telemetry/analytics**: Zero data collection (privacy-first)
3. **Remote code execution**: No plugin system, no script loading
4. **Multi-user isolation**: Desktop app runs as single user
5. **Hardware-based attestation**: No TPM/Secure Enclave integration (may be added in future)

---

## Reporting Security Issues

**Do NOT open public GitHub issues for security vulnerabilities.**

**Contact:** security@lotusemberlabs.com (if available) or DM maintainers privately.

**Response Time:**
- **Critical** (RCE, data exfiltration): 24-48 hours
- **High** (privilege escalation, DoS): 3-5 days
- **Medium/Low**: Best effort

**Disclosure Policy:**
- Coordinated disclosure after patch release
- Security advisories published on GitHub

**Bug Bounty:** Not currently available (pre-v1.0).

---

## Security Roadmap

**v1.0 Release Gates:**
- [x] CSP hardened (prod + dev)
- [x] Input validation on all commands
- [x] Config file permissions (0600)
- [x] Dependency auditing in CI
- [x] XSS prevention verified
- [ ] Third-party security audit (external)
- [ ] GPG-signed releases

**Post-v1.0:**
- Voice profile encryption (ChaCha20-Poly1305 already integrated)
- AppArmor/SELinux profiles
- Reproducible builds
- SBOM (Software Bill of Materials) generation

---

**Last Reviewed:** 2025-01-13
**Next Review:** Before v1.0 release
