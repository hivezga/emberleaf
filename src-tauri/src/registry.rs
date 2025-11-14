//! Model registry with SHA-256 verification and Ed25519 signatures
//! Contains defensive API functions reserved for future verification features
#![allow(dead_code)]

use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Ed25519 public key for verifying registry signatures (baked into binary)
/// This is a placeholder - replace with your actual public key in production
const REGISTRY_PUBLIC_KEY: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Model integrity verification state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationState {
    /// Hash matches signed registry
    Verified,
    /// File not in registry (allowed only in dev mode or with env var)
    Unknown,
    /// Hash mismatch - file modified or corrupted
    #[allow(dead_code)]
    Mismatch { expected: String, actual: String },
}

impl VerificationState {
    pub fn is_verified(&self) -> bool {
        matches!(self, VerificationState::Verified)
    }

    pub fn is_safe(&self) -> bool {
        match self {
            VerificationState::Verified => true,
            VerificationState::Unknown => {
                // Allow unknown models if explicitly enabled
                std::env::var("EMVER_ALLOW_UNKNOWN_MODELS")
                    .map(|v| v == "1")
                    .unwrap_or(false)
                    || cfg!(debug_assertions) // Allow in dev builds
            }
            VerificationState::Mismatch { .. } => false,
        }
    }
}

/// Model registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub path: String,
    pub sha256: String,
    #[serde(default)]
    pub description: String,
}

/// Model registry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub version: String,
    pub models: HashMap<String, ModelEntry>,
}

impl ModelRegistry {
    /// Load and verify registry from disk
    pub fn load_and_verify(registry_path: &Path, signature_path: &Path) -> Result<Self> {
        // Read registry JSON
        let registry_json = fs::read(registry_path).context("Failed to read registry.json")?;

        // Read detached signature
        let signature_bytes = fs::read(signature_path).context("Failed to read registry.sig")?;

        // Verify signature
        Self::verify_signature(&registry_json, &signature_bytes)?;

        // Parse registry
        let registry: ModelRegistry =
            serde_json::from_slice(&registry_json).context("Failed to parse registry.json")?;

        log::info!("Model registry loaded: {} models", registry.models.len());
        Ok(registry)
    }

    fn verify_signature(data: &[u8], signature_bytes: &[u8]) -> Result<()> {
        // Decode public key from hex
        let public_key_bytes = hex::decode(REGISTRY_PUBLIC_KEY)
            .map_err(|e| anyhow::anyhow!("Failed to decode public key: {}", e))?;

        let verifying_key = VerifyingKey::from_bytes(
            &public_key_bytes
                .try_into()
                .map_err(|_| anyhow!("Invalid public key length"))?,
        )
        .context("Failed to create verifying key")?;

        // Parse signature
        let signature = Signature::from_bytes(
            signature_bytes
                .try_into()
                .map_err(|_| anyhow!("Invalid signature length"))?,
        );

        // Verify
        verifying_key
            .verify(data, &signature)
            .context("Registry signature verification failed")?;

        log::info!("Registry signature verified successfully");
        Ok(())
    }

    /// Verify a single file against the registry
    pub fn verify_file(&self, path: &Path) -> Result<VerificationState> {
        let path_str = path.to_string_lossy().to_string();

        // Compute actual hash
        let actual_hash = compute_sha256(path)?;

        // Check if file is in registry
        if let Some(entry) = self.models.get(&path_str) {
            if entry.sha256 == actual_hash {
                Ok(VerificationState::Verified)
            } else {
                Ok(VerificationState::Mismatch {
                    expected: entry.sha256.clone(),
                    actual: actual_hash,
                })
            }
        } else {
            Ok(VerificationState::Unknown)
        }
    }
}

/// Compute SHA256 hash of a file
pub fn compute_sha256(path: &Path) -> Result<String> {
    let bytes =
        fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();

    Ok(hex::encode(hash))
}

/// Find a model file by pattern (e.g., "encoder*.onnx"), excluding int8 quantized versions
fn find_model_file(model_dir: &Path, pattern_prefix: &str, extension: &str) -> Result<PathBuf> {
    let entries = std::fs::read_dir(model_dir)
        .with_context(|| format!("Failed to read model directory: {}", model_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Match pattern and exclude int8 quantized versions
            if filename.starts_with(pattern_prefix)
                && filename.ends_with(extension)
                && !filename.contains(".int8.")
            {
                return Ok(path);
            }
        }
    }

    bail!(
        "Model file matching pattern '{}*{}' not found in {}",
        pattern_prefix,
        extension,
        model_dir.display()
    )
}

/// Verify an ONNX model set (encoder, decoder, joiner, tokens) - auto-detects filenames
pub fn verify_onnx_set(model_dir: &Path) -> Result<HashMap<String, VerificationState>> {
    let mut results = HashMap::new();

    // Auto-detect model files by pattern (exclude int8 quantized versions)
    let patterns = [
        ("encoder", ".onnx"),
        ("decoder", ".onnx"),
        ("joiner", ".onnx"),
        ("tokens", ".txt"),
    ];

    for (prefix, ext) in &patterns {
        let file_path = find_model_file(model_dir, prefix, ext)?;

        // For now, just compute hash (registry verification will be added when registry exists)
        let hash = compute_sha256(&file_path)?;
        let filename = file_path.file_name().unwrap().to_str().unwrap();
        log::debug!("{}: {}", filename, hash);

        // In production, load registry and verify
        // For now, mark as Unknown (will be allowed in dev mode)
        results.insert(filename.to_string(), VerificationState::Unknown);
    }

    Ok(results)
}

// Add hex dependency to Cargo.toml
// For now, use a simple hex encoding implementation
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("Invalid hex: {}", e))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_state() {
        assert!(VerificationState::Verified.is_verified());
        assert!(!VerificationState::Unknown.is_verified());
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"hello";
        let encoded = hex::encode(data);
        assert_eq!(encoded, "68656c6c6f");

        let decoded = hex::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
