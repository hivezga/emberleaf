/**
 * Input Validation Module - SEC-001
 *
 * Centralized validation for all Tauri command inputs
 * Prevents injection attacks, invalid ranges, and malformed data
 *
 * Note: Contains defensive API functions reserved for future use
 */
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use tauri::Emitter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid range: {0}")]
    InvalidRange(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Path traversal detected")]
    PathTraversal,

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Value too long: max {max}, got {actual}")]
    ValueTooLong { max: usize, actual: usize },
}

/// Validate VAD threshold (0.0 to 1.0)
pub fn validate_vad_threshold(threshold: f32) -> Result<f32, ValidationError> {
    if !(0.0..=1.0).contains(&threshold) {
        return Err(ValidationError::InvalidRange(format!(
            "VAD threshold must be between 0.0 and 1.0, got {}",
            threshold
        )));
    }
    Ok(threshold)
}

/// Validate audio duration in milliseconds (10ms to 5000ms)
pub fn validate_duration_ms(duration_ms: u32) -> Result<u32, ValidationError> {
    if !(10..=5000).contains(&duration_ms) {
        return Err(ValidationError::InvalidRange(format!(
            "Duration must be between 10ms and 5000ms, got {}ms",
            duration_ms
        )));
    }
    Ok(duration_ms)
}

/// Validate audio frequency in Hz (50Hz to 4000Hz)
pub fn validate_frequency_hz(frequency: u32) -> Result<u32, ValidationError> {
    if !(50..=4000).contains(&frequency) {
        return Err(ValidationError::InvalidRange(format!(
            "Frequency must be between 50Hz and 4000Hz, got {}Hz",
            frequency
        )));
    }
    Ok(frequency)
}

/// Validate device name (max 256 chars, no control characters)
pub fn validate_device_name(name: &str) -> Result<String, ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::InvalidFormat(
            "Device name cannot be empty".to_string(),
        ));
    }

    if name.len() > 256 {
        return Err(ValidationError::ValueTooLong {
            max: 256,
            actual: name.len(),
        });
    }

    // Check for control characters
    if name.chars().any(|c| c.is_control()) {
        return Err(ValidationError::InvalidFormat(
            "Device name contains invalid control characters".to_string(),
        ));
    }

    Ok(name.to_string())
}

/// Validate file path (must be within allowed directories, no traversal)
pub fn validate_path(path: &str, allowed_base: &Path) -> Result<PathBuf, ValidationError> {
    // Reject empty paths
    if path.is_empty() {
        return Err(ValidationError::InvalidPath(
            "Path cannot be empty".to_string(),
        ));
    }

    // Reject paths with null bytes
    if path.contains('\0') {
        return Err(ValidationError::InvalidPath(
            "Path contains null byte".to_string(),
        ));
    }

    // Parse path
    let path_buf = PathBuf::from(path);

    // Canonicalize to resolve .. and symlinks (if path exists)
    // If path doesn't exist yet, validate parent
    let canonical =
        if path_buf.exists() {
            path_buf
                .canonicalize()
                .map_err(|e| ValidationError::InvalidPath(e.to_string()))?
        } else {
            // For non-existent paths, check parent and validate filename
            if let Some(parent) = path_buf.parent() {
                if parent.as_os_str().is_empty() {
                    // Relative path with no parent - reject
                    return Err(ValidationError::InvalidPath(
                        "Relative paths not allowed".to_string(),
                    ));
                }
                if parent.exists() {
                    let canonical_parent = parent
                        .canonicalize()
                        .map_err(|e| ValidationError::InvalidPath(e.to_string()))?;
                    canonical_parent.join(path_buf.file_name().ok_or_else(|| {
                        ValidationError::InvalidPath("Invalid filename".to_string())
                    })?)
                } else {
                    // Parent doesn't exist - validate it's not trying traversal
                    if path.contains("..") {
                        return Err(ValidationError::PathTraversal);
                    }
                    path_buf
                }
            } else {
                return Err(ValidationError::InvalidPath(
                    "Invalid path structure".to_string(),
                ));
            }
        };

    // Ensure path is within allowed base directory
    if !canonical.starts_with(allowed_base) {
        return Err(ValidationError::PathTraversal);
    }

    Ok(canonical)
}

/// Validate profile name for voice biometrics (alphanumeric + underscore, max 64 chars)
pub fn validate_profile_name(name: &str) -> Result<String, ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::InvalidFormat(
            "Profile name cannot be empty".to_string(),
        ));
    }

    if name.len() > 64 {
        return Err(ValidationError::ValueTooLong {
            max: 64,
            actual: name.len(),
        });
    }

    // Only allow alphanumeric, underscore, and hyphen
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ValidationError::InvalidFormat(
            "Profile name can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }

    Ok(name.to_string())
}

/// Validate KWS sensitivity (0.0 to 1.0)
pub fn validate_sensitivity(sensitivity: f32) -> Result<f32, ValidationError> {
    if !(0.0..=1.0).contains(&sensitivity) {
        return Err(ValidationError::InvalidRange(format!(
            "Sensitivity must be between 0.0 and 1.0, got {}",
            sensitivity
        )));
    }
    Ok(sensitivity)
}

/// Validate monitor gain (0.0 to 0.5 for safety)
pub fn validate_gain(gain: f32) -> Result<f32, ValidationError> {
    if !(0.0..=0.5).contains(&gain) {
        return Err(ValidationError::InvalidRange(format!(
            "Gain must be between 0.0 and 0.5, got {}",
            gain
        )));
    }
    Ok(gain)
}

/// Validate optional device name
pub fn validate_opt_device_name(name: &Option<String>) -> Result<(), ValidationError> {
    if let Some(n) = name {
        validate_device_name(n)?;
    }
    Ok(())
}

/// Validate device ID components (host_api, index, name)
pub fn validate_device_id(host_api: &str, index: i32, name: &str) -> Result<(), ValidationError> {
    // Validate host API identifier (simple alphanumeric token)
    if host_api.is_empty() || host_api.len() > 64 {
        return Err(ValidationError::InvalidFormat(
            "Host API identifier must be 1-64 characters".to_string(),
        ));
    }

    if !host_api
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(ValidationError::InvalidFormat(
            "Host API identifier contains invalid characters".to_string(),
        ));
    }

    // Validate index is non-negative
    if index < 0 {
        return Err(ValidationError::InvalidRange(format!(
            "Device index must be >= 0, got {}",
            index
        )));
    }

    // Validate device name
    validate_device_name(name)?;

    Ok(())
}

/// Validate frequency in Hz (accepts f32 for test tone)
pub fn validate_frequency_hz_f32(frequency: f32) -> Result<f32, ValidationError> {
    if !(50.0..=4000.0).contains(&frequency) {
        return Err(ValidationError::InvalidRange(format!(
            "Frequency must be between 50Hz and 4000Hz, got {}Hz",
            frequency
        )));
    }
    Ok(frequency)
}

/// Validate VAD mode string
pub fn validate_vad_mode(mode: &str) -> Result<String, ValidationError> {
    match mode {
        "aggressive" | "balanced" | "sensitive" => Ok(mode.to_string()),
        _ => Err(ValidationError::InvalidFormat(format!(
            "Invalid VAD mode '{}', must be 'aggressive', 'balanced', or 'sensitive'",
            mode
        ))),
    }
}

// ========== Validation Error Emission ==========

#[derive(serde::Serialize, Debug, Clone)]
pub struct ValidationErrorPayload<'a> {
    pub code: &'a str,    // e.g., "invalid_gain", "invalid_device_name"
    pub message: &'a str, // short, user-friendly
    pub field: &'a str,   // e.g., "gain", "device_name"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// Emit a validation error event to the frontend
pub fn emit_validation_error(
    app: &tauri::AppHandle,
    code: &str,
    field: &str,
    message: &str,
    value: Option<serde_json::Value>,
) {
    let payload = ValidationErrorPayload {
        code,
        field,
        message,
        value,
    };
    let _ = app.emit("audio:error", &payload);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_vad_threshold_valid() {
        assert!(validate_vad_threshold(0.0).is_ok());
        assert!(validate_vad_threshold(0.5).is_ok());
        assert!(validate_vad_threshold(1.0).is_ok());
    }

    #[test]
    fn test_vad_threshold_invalid() {
        assert!(validate_vad_threshold(-0.1).is_err());
        assert!(validate_vad_threshold(1.1).is_err());
        assert!(validate_vad_threshold(999.0).is_err());
    }

    #[test]
    fn test_duration_valid() {
        assert!(validate_duration_ms(10).is_ok());
        assert!(validate_duration_ms(100).is_ok());
        assert!(validate_duration_ms(5000).is_ok());
    }

    #[test]
    fn test_duration_invalid() {
        assert!(validate_duration_ms(9).is_err());
        assert!(validate_duration_ms(5001).is_err());
        assert!(validate_duration_ms(0).is_err());
    }

    #[test]
    fn test_device_name_valid() {
        assert!(validate_device_name("USB Microphone").is_ok());
        assert!(validate_device_name("Built-in Audio").is_ok());
    }

    #[test]
    fn test_device_name_invalid() {
        assert!(validate_device_name("").is_err()); // Empty
        assert!(validate_device_name(&"a".repeat(257)).is_err()); // Too long
        assert!(validate_device_name("test\x00name").is_err()); // Null byte
        assert!(validate_device_name("test\nname").is_err()); // Newline
    }

    #[test]
    fn test_path_traversal() {
        let temp_dir = env::temp_dir();
        assert!(validate_path("../../../etc/passwd", &temp_dir).is_err());
        assert!(validate_path("./config/../../../etc/passwd", &temp_dir).is_err());
    }

    #[test]
    fn test_profile_name_valid() {
        assert!(validate_profile_name("user123").is_ok());
        assert!(validate_profile_name("john_doe").is_ok());
        assert!(validate_profile_name("profile-2024").is_ok());
    }

    #[test]
    fn test_profile_name_invalid() {
        assert!(validate_profile_name("").is_err()); // Empty
        assert!(validate_profile_name(&"a".repeat(65)).is_err()); // Too long
        assert!(validate_profile_name("user@domain").is_err()); // Special char
        assert!(validate_profile_name("user name").is_err()); // Space
        assert!(validate_profile_name("user/path").is_err()); // Slash
    }

    #[test]
    fn test_sensitivity_valid() {
        assert!(validate_sensitivity(0.0).is_ok());
        assert!(validate_sensitivity(0.5).is_ok());
        assert!(validate_sensitivity(1.0).is_ok());
    }

    #[test]
    fn test_sensitivity_invalid() {
        assert!(validate_sensitivity(-0.01).is_err());
        assert!(validate_sensitivity(1.01).is_err());
        assert!(validate_sensitivity(-1.0).is_err());
    }

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

    #[test]
    fn test_device_id_valid() {
        assert!(validate_device_id("pulse", 0, "USB Mic").is_ok());
        assert!(validate_device_id("alsa", 2, "Built-in Audio").is_ok());
        assert!(validate_device_id("jack_v2.0", 10, "System Audio").is_ok());
    }

    #[test]
    fn test_device_id_invalid() {
        assert!(validate_device_id("bad host", 0, "USB Mic").is_err()); // Space in host_api
        assert!(validate_device_id("alsa", -1, "USB Mic").is_err()); // Negative index
        assert!(validate_device_id("pulse", 0, "").is_err()); // Empty name
        assert!(validate_device_id("", 0, "USB Mic").is_err()); // Empty host_api
    }

    #[test]
    fn test_opt_device_name_valid() {
        assert!(validate_opt_device_name(&None).is_ok());
        assert!(validate_opt_device_name(&Some("USB Mic".to_string())).is_ok());
    }

    #[test]
    fn test_opt_device_name_invalid() {
        assert!(validate_opt_device_name(&Some("".to_string())).is_err());
        assert!(validate_opt_device_name(&Some("test\x00name".to_string())).is_err());
    }

    #[test]
    fn test_frequency_hz_f32_valid() {
        assert!(validate_frequency_hz_f32(50.0).is_ok());
        assert!(validate_frequency_hz_f32(440.0).is_ok());
        assert!(validate_frequency_hz_f32(4000.0).is_ok());
    }

    #[test]
    fn test_frequency_hz_f32_invalid() {
        assert!(validate_frequency_hz_f32(49.9).is_err());
        assert!(validate_frequency_hz_f32(4000.1).is_err());
        assert!(validate_frequency_hz_f32(20.0).is_err());
    }

    #[test]
    fn test_vad_mode_valid() {
        assert!(validate_vad_mode("aggressive").is_ok());
        assert!(validate_vad_mode("balanced").is_ok());
        assert!(validate_vad_mode("sensitive").is_ok());
    }

    #[test]
    fn test_vad_mode_invalid() {
        assert!(validate_vad_mode("invalid").is_err());
        assert!(validate_vad_mode("Aggressive").is_err()); // Case sensitive
        assert!(validate_vad_mode("").is_err());
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn device_name_rejects_controls(s in r"[\x00-\x1F\x7F]{1,16}") {
            assert!(validate_device_name(&s).is_err());
        }

        #[test]
        fn device_name_accepts_reasonable_ascii(s in r"[A-Za-z0-9 _\-]{1,64}") {
            // Allowed set should pass
            assert!(validate_device_name(&s).is_ok());
        }

        #[test]
        fn threshold_in_unit_interval(x in 0.0f32..1.0) {
            assert!(validate_vad_threshold(x).is_ok());
        }

        #[test]
        fn threshold_outside_unit_interval(x in any::<f32>().prop_filter("out of [0,1]", |v| *v < 0.0 || *v > 1.0)) {
            assert!(validate_vad_threshold(x).is_err());
        }

        #[test]
        fn gain_in_safe_range(x in 0.0f32..0.5) {
            assert!(validate_gain(x).is_ok());
        }

        #[test]
        fn gain_outside_safe_range(x in any::<f32>().prop_filter("out of [0,0.5]", |v| *v < 0.0 || *v > 0.5)) {
            assert!(validate_gain(x).is_err());
        }

        #[test]
        fn frequency_in_voice_range(x in 50.0f32..4000.0) {
            assert!(validate_frequency_hz_f32(x).is_ok());
        }

        #[test]
        fn frequency_outside_voice_range(x in any::<f32>().prop_filter("out of [50,4000]", |v| *v < 50.0 || *v > 4000.0)) {
            assert!(validate_frequency_hz_f32(x).is_err());
        }

        #[test]
        fn duration_in_valid_range(x in 10u32..5000) {
            assert!(validate_duration_ms(x).is_ok());
        }

        #[test]
        fn duration_outside_valid_range(x in any::<u32>().prop_filter("out of [10,5000]", |v| *v < 10 || *v > 5000)) {
            assert!(validate_duration_ms(x).is_err());
        }
    }
}
