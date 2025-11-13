# Decision Log

> Purpose: capture "why" behind choices. Keep entries short; link to PRs.

## [2025-11-12] Packaging: AppImage over Flatpak

**Context:** Linux-first release; minimal dependencies; users outside Flatpak ecosystems.
**Options:** AppImage, Flatpak, Deb-only.
**Decision:** Ship **AppImage + .deb**; revisit Flatpak post-v1.0.
**Why:** Single-file portability; no portal quirks; works on most distros without store.
**Risks:** No sandbox by default → mitigated via CSP & Tauri hardening.
**Links:** REL-001 PR-###.

## [2025-11-12] Default Locale: ES-MX first run

**Context:** Primary early adopters are Spanish (MX).
**Decision:** First-run UI **ES-MX**, easy toggle to EN in Advanced.
**Why:** Reduce setup friction; we already ship i18n.
**Risks:** None (toggle persists).
**Links:** ONB-001 PR-###.

## [2025-11-12] Model Integrity: SHA-256 checksums (no GPG)

**Context:** Distribute ONNX models via HTTPS, no key mgmt overhead.
**Decision:** **SHA-256** hash verification + HTTPS; log and block on mismatch.
**Why:** Simpler pipeline; avoids key distribution complexity for v1.0.
**Risks:** Weaker provenance than signed artifacts → revisit with GPG in SEC-001.
**Links:** KWS-REAL-001 PR-###.
