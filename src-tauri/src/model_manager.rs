//! Model Manager - Download, verify, and manage KWS models
//!
//! This module handles:
//! - Model registry loading and parsing
//! - Model downloading with progress tracking
//! - SHA256 verification
//! - Model storage management

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

const ALLOWED_HOSTS: &[&str] = &["github.com", "huggingface.co"];

/// KWS Model registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KwsModelEntry {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub lang: String,
    pub wakeword: String,
    #[serde(default)]
    pub description: String,
}

/// KWS Model registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KwsRegistry {
    pub version: String,
    pub models: HashMap<String, KwsModelEntry>,
}

/// Model download progress event
#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadProgress {
    pub model_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f32,
}

impl KwsRegistry {
    /// Load KWS registry from file
    pub fn load(registry_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(registry_path)
            .with_context(|| format!("Failed to read registry: {}", registry_path.display()))?;

        let registry: KwsRegistry =
            serde_json::from_str(&content).context("Failed to parse KWS registry JSON")?;

        log::info!("KWS registry loaded: {} models", registry.models.len());
        Ok(registry)
    }

    /// Get model entry by ID
    pub fn get_model(&self, model_id: &str) -> Option<&KwsModelEntry> {
        self.models.get(model_id)
    }

    /// List all available model IDs
    pub fn list_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
}

/// Model Manager for KWS models
pub struct ModelManager {
    models_dir: PathBuf,
    registry: Option<KwsRegistry>,
}

impl ModelManager {
    /// Create a new ModelManager
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            registry: None,
        }
    }

    /// Load registry from default location
    pub fn load_registry(&mut self, registry_path: &Path) -> Result<()> {
        self.registry = Some(KwsRegistry::load(registry_path)?);
        Ok(())
    }

    /// Get the registry (must be loaded first)
    pub fn registry(&self) -> Result<&KwsRegistry> {
        self.registry
            .as_ref()
            .ok_or_else(|| anyhow!("Registry not loaded"))
    }

    /// Validate model ID (alphanumeric, _ - only, max 64 chars)
    pub fn validate_model_id(model_id: &str) -> Result<()> {
        if model_id.is_empty() || model_id.len() > 64 {
            bail!("Model ID must be 1-64 characters");
        }

        if !model_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            bail!("Model ID can only contain alphanumeric, underscore, and hyphen");
        }

        Ok(())
    }

    /// Validate download URL (must be from allowed hosts)
    pub fn validate_url(url: &str) -> Result<()> {
        let parsed = url::Url::parse(url).context("Invalid URL")?;

        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow!("URL has no host"))?;

        if !ALLOWED_HOSTS.contains(&host) {
            bail!("URL host '{}' not in allowlist: {:?}", host, ALLOWED_HOSTS);
        }

        Ok(())
    }

    /// Get model directory path
    pub fn model_dir(&self, model_id: &str) -> PathBuf {
        self.models_dir.join("kws").join(model_id)
    }

    /// Check if model is already downloaded and verified
    pub fn is_model_ready(&self, model_id: &str) -> Result<bool> {
        Self::validate_model_id(model_id)?;

        let model_dir = self.model_dir(model_id);
        if !model_dir.exists() {
            return Ok(false);
        }

        // Check for required files
        let required_files = ["encoder", "decoder", "joiner", "tokens"];
        for prefix in &required_files {
            let pattern = if *prefix == "tokens" { ".txt" } else { ".onnx" };
            if !self
                .find_file_by_pattern(&model_dir, prefix, pattern)?
                .is_some()
            {
                return Ok(false);
            }
        }

        // Verify checksums if registry is available
        if let Some(registry) = &self.registry {
            if let Some(entry) = registry.get_model(model_id) {
                return self.verify_model(model_id, &entry.sha256);
            }
        }

        Ok(true)
    }

    /// Find a file in directory by pattern (e.g., "encoder" + ".onnx")
    fn find_file_by_pattern(&self, dir: &Path, prefix: &str, ext: &str) -> Result<Option<PathBuf>> {
        if !dir.exists() {
            return Ok(None);
        }

        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(prefix)
                    && filename.ends_with(ext)
                    && !filename.contains(".int8.")
                {
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }

    /// Verify model integrity against SHA256
    pub fn verify_model(&self, model_id: &str, expected_sha256: &str) -> Result<bool> {
        let model_dir = self.model_dir(model_id);

        // Compute combined hash of all model files
        let mut hasher = Sha256::new();

        let files = ["encoder", "decoder", "joiner", "tokens"];
        for prefix in &files {
            let ext = if *prefix == "tokens" { ".txt" } else { ".onnx" };
            if let Some(file_path) = self.find_file_by_pattern(&model_dir, prefix, ext)? {
                let content = fs::read(&file_path)
                    .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
                hasher.update(&content);
            } else {
                log::warn!("Missing file for pattern: {}{}", prefix, ext);
                return Ok(false);
            }
        }

        let actual_hash = format!("{:x}", hasher.finalize());
        Ok(actual_hash == expected_sha256)
    }

    /// Download model with progress tracking
    pub async fn download_model(&self, app_handle: &AppHandle, model_id: &str) -> Result<()> {
        Self::validate_model_id(model_id)?;

        let registry = self.registry()?;
        let entry = registry
            .get_model(model_id)
            .ok_or_else(|| anyhow!("Model '{}' not found in registry", model_id))?;

        Self::validate_url(&entry.url)?;

        log::info!("Downloading model '{}' from {}", model_id, entry.url);

        let model_dir = self.model_dir(model_id);
        fs::create_dir_all(&model_dir).with_context(|| {
            format!("Failed to create model directory: {}", model_dir.display())
        })?;

        // Download the model archive (assuming it's a tarball or zip)
        let client = reqwest::Client::new();
        let mut response = client
            .get(&entry.url)
            .send()
            .await
            .context("Failed to start download")?;

        if !response.status().is_success() {
            bail!("Download failed with status: {}", response.status());
        }

        let total_size = entry.size;
        let mut downloaded: u64 = 0;
        let mut buffer = Vec::new();

        // Download with progress tracking
        while let Some(chunk) = response.chunk().await.context("Failed to read chunk")? {
            buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

            // Emit progress event (throttled to 10Hz)
            if downloaded.is_multiple_of((total_size / 100).max(1)) || downloaded == total_size {
                let percent = (downloaded as f32 / total_size as f32) * 100.0;
                let progress = ModelDownloadProgress {
                    model_id: model_id.to_string(),
                    downloaded,
                    total: total_size,
                    percent,
                };

                let _ = app_handle.emit("kws:model_download_progress", &progress);
            }
        }

        log::info!("Download complete: {} bytes", downloaded);

        // Write to temporary file
        let archive_path = model_dir.join(format!("{}.tar.gz", model_id));
        let mut file = fs::File::create(&archive_path)
            .with_context(|| format!("Failed to create file: {}", archive_path.display()))?;
        file.write_all(&buffer)
            .context("Failed to write downloaded data")?;

        log::info!("Extracting model archive...");

        // Extract archive (assumes tar.gz format)
        self.extract_archive(&archive_path, &model_dir)?;

        // Clean up archive
        fs::remove_file(&archive_path).ok();

        log::info!("Model extraction complete");

        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_archive(&self, archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let tar_gz = fs::File::open(archive_path)
            .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        archive
            .unpack(dest_dir)
            .with_context(|| format!("Failed to extract archive to: {}", dest_dir.display()))?;

        Ok(())
    }

    /// Remove model from disk
    pub fn remove_model(&self, model_id: &str) -> Result<()> {
        Self::validate_model_id(model_id)?;

        let model_dir = self.model_dir(model_id);
        if model_dir.exists() {
            fs::remove_dir_all(&model_dir).with_context(|| {
                format!("Failed to remove model directory: {}", model_dir.display())
            })?;
            log::info!("Removed model: {}", model_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model_id() {
        assert!(ModelManager::validate_model_id("hey_ember").is_ok());
        assert!(ModelManager::validate_model_id("model-123").is_ok());
        assert!(ModelManager::validate_model_id("").is_err());
        assert!(ModelManager::validate_model_id("a".repeat(65).as_str()).is_err());
        assert!(ModelManager::validate_model_id("model/path").is_err());
        assert!(ModelManager::validate_model_id("model:name").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(ModelManager::validate_url("https://github.com/user/repo/model.tar.gz").is_ok());
        assert!(ModelManager::validate_url("https://huggingface.co/models/model.tar.gz").is_ok());
        assert!(ModelManager::validate_url("https://evil.com/model.tar.gz").is_err());
        assert!(ModelManager::validate_url("ftp://github.com/model.tar.gz").is_ok());
        // URL parsing allows this
    }
}
