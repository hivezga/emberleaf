# Command Input Validation Matrix

**Last Updated:** 2025-01-13
**Related:** SEC-001, SEC-001B

This document provides a comprehensive overview of input validation applied to all Tauri commands in Emberleaf.

---

## Overview

All user-facing Tauri commands implement **centralized input validation** via `src-tauri/src/validation.rs`. Validation failures:

1. Return early with descriptive error messages
2. Emit `audio:error` events to the frontend
3. Display localized toast notifications (ES/EN)

---

## Command Validation Matrix

| Command | Input Fields | Validators Applied | Error Codes |
|---------|-------------|-------------------|-------------|
| **set_input_device** | `name: String` | `validate_device_name` | `invalid_device_name` |
| **set_output_device** | `name: String` | `validate_device_name` | `invalid_device_name` |
| **play_test_tone** | `frequency_hz: Option<f32>`<br>`duration_ms: Option<u32>`<br>`device_name: Option<String>` | `validate_frequency_hz_f32`<br>`validate_duration_ms`<br>`validate_opt_device_name` | `invalid_frequency`<br>`invalid_duration`<br>`invalid_device_name` |
| **start_mic_monitor** | `gain: Option<f32>` | `validate_gain` | `invalid_gain` |
| **vad_set_threshold** | `threshold: f32` | `validate_vad_threshold` | `invalid_threshold` |
| **kws_set_sensitivity** | `sensitivity: f32` | `validate_sensitivity` | `invalid_sensitivity` |
| **list_input_devices** | — | — (read-only) | — |
| **list_output_devices** | — | — (read-only) | — |
| **current_input_device** | — | — (read-only) | — |
| **current_output_device** | — | — (read-only) | — |
| **restart_audio_capture** | — | — (no inputs) | — |
| **stop_mic_monitor** | — | — (no inputs) | — |
| **get_config** | — | — (read-only) | — |
| **get_audio_debug** | — | — (read-only) | — |
| **get_audio_snapshot** | — | — (read-only) | — |
| **run_preflight_checks** | — | — (no inputs) | — |
| **suggest_input_device** | — | — (no inputs) | — |

---

## Validation Rules

### Device Name (`validate_device_name`)
- **Range:** 1–256 characters
- **Constraints:**
  - No empty strings
  - No control characters (`\x00-\x1F`, `\x7F`)
  - Alphanumeric, spaces, underscores, hyphens allowed
- **Error Code:** `invalid_device_name`
- **Localized Messages:**
  - **EN:** "Device name is invalid."
  - **ES:** "El nombre del dispositivo no es válido."

### Frequency (`validate_frequency_hz_f32`)
- **Range:** 50.0 Hz – 4000.0 Hz
- **Rationale:** Voice frequency range
- **Error Code:** `invalid_frequency`
- **Localized Messages:**
  - **EN:** "Frequency out of range (50–4000 Hz)."
  - **ES:** "Frecuencia fuera de rango (50–4000 Hz)."

### Duration (`validate_duration_ms`)
- **Range:** 10 ms – 5000 ms
- **Rationale:** Prevent DoS via excessive processing
- **Error Code:** `invalid_duration`
- **Localized Messages:**
  - **EN:** "Duration out of range (10–5000 ms)."
  - **ES:** "Duración fuera de rango (10–5000 ms)."

### Gain (`validate_gain`)
- **Range:** 0.0 – 0.5
- **Rationale:** Prevent audio feedback (safety cap at 50%)
- **Error Code:** `invalid_gain`
- **Localized Messages:**
  - **EN:** "Gain out of range (0.0–0.5)."
  - **ES:** "Ganancia fuera de rango (0.0–0.5)."

### VAD Threshold (`validate_vad_threshold`)
- **Range:** 0.0 – 1.0
- **Rationale:** Probability-based sensitivity
- **Error Code:** `invalid_threshold`
- **Localized Messages:**
  - **EN:** "VAD threshold out of range (0.0–1.0)."
  - **ES:** "Umbral VAD fuera de rango (0.0–1.0)."

### KWS Sensitivity (`validate_sensitivity`)
- **Range:** 0.0 – 1.0
- **Rationale:** Probability-based sensitivity
- **Error Code:** `invalid_sensitivity`
- **Localized Messages:**
  - **EN:** "Sensitivity out of range (0.0–1.0)."
  - **ES:** "Sensibilidad fuera de rango (0.0–1.0)."

### Optional Device Name (`validate_opt_device_name`)
- **Delegates to:** `validate_device_name` if `Some(name)` is provided
- **Allows:** `None` (uses default device)

---

## Event Emission

All validation failures emit the `audio:error` event with the following payload:

```typescript
{
  code: string,        // e.g., "invalid_gain"
  message: string,     // User-friendly error description
  field: string,       // e.g., "gain", "device_name"
  value?: unknown      // The invalid value (for debugging)
}
```

**Frontend Handling:**
- Global listener in `src/lib/tauriSafe.ts` (`subscribeGlobalErrors()`)
- Maps error codes to localized messages via `src/lib/errors.ts` (`toUserMessage()`)
- Displays toast notification using `sonner` (5-second duration)

---

## Testing

### Unit Tests
All validators have comprehensive unit tests in `src-tauri/src/validation.rs`:

```bash
cargo test --bin ember validation
```

**Test Coverage:**
- Valid boundary values (min, mid, max)
- Invalid values (below min, above max, edge cases)
- Empty strings, control characters, null bytes
- Path traversal attempts (for file path validators)

**Example:**
```rust
#[test]
fn test_gain_valid() {
    assert!(validate_gain(0.0).is_ok());
    assert!(validate_gain(0.25).is_ok());
    assert!(validate_gain(0.5).is_ok());
}

#[test]
fn test_gain_invalid() {
    assert!(validate_gain(-0.01).is_err());
    assert!(validate_gain(0.51).is_err());
    assert!(validate_gain(1.0).is_err());
}
```

### Property-Based Tests
Fuzz-like testing using `proptest` (runs 256 cases per test by default):

```bash
cargo test --bin ember prop_tests
```

**Examples:**
- Device names with control characters (should reject)
- Frequencies within/outside voice range
- Gain values within/outside safe range

---

## CI Integration

The `quick-validation` job runs on every PR:

```yaml
quick-validation:
  name: Quick Validation (Rust unit subset)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          src-tauri/target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    - name: Unit tests (validation module)
      run: cargo test -q --manifest-path src-tauri/Cargo.toml validation
```

**Result:** Fast early failure (~30 seconds) before heavier jobs run.

---

## Future Validators (Placeholders)

The following validators are defined but not yet wired to commands:

| Validator | Purpose | Status |
|-----------|---------|--------|
| `validate_path` | File path security (no traversal) | Reserved for future file ops |
| `validate_profile_name` | Voice profile identifiers | Reserved for BE-002 (voice biometrics) |
| `validate_device_id` | Device ID components | Reserved for advanced device management |
| `validate_vad_mode` | VAD mode string validation | Reserved for configurable VAD modes |

---

## Security Notes

- **Path Traversal:** `validate_path` canonicalizes paths and rejects `..` sequences
- **Control Character Injection:** All string validators reject `\x00-\x1F` and `\x7F`
- **DoS Prevention:** Duration and frequency ranges prevent resource exhaustion
- **Feedback Prevention:** Gain cap at 0.5 prevents monitor feedback loops

---

## References

- **SEC-001:** Input Validation Framework
- **SEC-001B:** Apply Validation to All Commands
- **SECURITY.md:** Comprehensive security policy
- **CONTRIBUTING.md:** Validation checklist for new commands

---

**Last Updated:** 2025-01-13
**Next Review:** Before v1.0 release
