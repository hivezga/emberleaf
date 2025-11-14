use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Application paths following OS conventions
#[derive(Clone, Debug)]
pub struct AppPaths {
    /// Configuration directory (settings, thresholds)
    pub config: PathBuf,
    /// Data directory (models, voiceprints, sync, state.toml)
    pub data: PathBuf,
    /// Cache directory (temp audio, logs)
    pub cache: PathBuf,
}

impl AppPaths {
    /// Resolve OS-specific paths for Emberleaf
    ///
    /// # Platform Paths
    ///
    /// ## Linux
    /// - Config: `~/.config/Emberleaf/`
    /// - Data: `~/.local/share/Emberleaf/` → models/, voiceprints/, sync/, state.toml
    /// - Cache: `~/.cache/Emberleaf/` → tmp_audio/, logs/
    ///
    /// ## macOS
    /// - Config: `~/Library/Preferences/Emberleaf/`
    /// - Data: `~/Library/Application Support/Emberleaf/`
    /// - Cache: `~/Library/Caches/Emberleaf/`
    ///
    /// ## Windows
    /// - Config: `%APPDATA%\Emberleaf\config\`
    /// - Data: `%LOCALAPPDATA%\Emberleaf\`
    /// - Cache: `%LOCALAPPDATA%\Emberleaf\Cache\`
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "LotusEmberLabs", "Emberleaf")
            .context("Failed to determine project directories")?;

        let paths = Self {
            config: proj_dirs.config_dir().to_path_buf(),
            data: proj_dirs.data_dir().to_path_buf(),
            cache: proj_dirs.cache_dir().to_path_buf(),
        };

        Ok(paths)
    }

    /// Create all necessary directories with subdirectories
    pub fn ensure_directories(&self) -> Result<()> {
        // Config directory
        fs::create_dir_all(&self.config).context("Failed to create config directory")?;

        // Data directory and subdirectories
        fs::create_dir_all(&self.data).context("Failed to create data directory")?;
        fs::create_dir_all(self.data.join("models"))
            .context("Failed to create models directory")?;
        fs::create_dir_all(self.data.join("voiceprints"))
            .context("Failed to create voiceprints directory")?;
        fs::create_dir_all(self.data.join("sync")).context("Failed to create sync directory")?;

        // Cache directory and subdirectories
        fs::create_dir_all(&self.cache).context("Failed to create cache directory")?;
        fs::create_dir_all(self.cache.join("tmp_audio"))
            .context("Failed to create tmp_audio directory")?;
        fs::create_dir_all(self.cache.join("logs")).context("Failed to create logs directory")?;

        log::info!("Application directories initialized");
        log::debug!("  Config: {}", self.config.display());
        log::debug!("  Data:   {}", self.data.display());
        log::debug!("  Cache:  {}", self.cache.display());

        Ok(())
    }

    /// Get path to config file
    pub fn config_file(&self) -> PathBuf {
        self.config.join("config.toml")
    }

    /// Get path to state file
    #[allow(dead_code)]
    pub fn state_file(&self) -> PathBuf {
        self.data.join("state.toml")
    }

    /// Get path to models directory
    pub fn models_dir(&self) -> PathBuf {
        self.data.join("models")
    }

    /// Get path to model registry
    #[allow(dead_code)]
    pub fn model_registry(&self) -> PathBuf {
        self.models_dir().join("registry.json")
    }

    /// Get path to model registry signature
    #[allow(dead_code)]
    pub fn model_registry_sig(&self) -> PathBuf {
        self.models_dir().join("registry.sig")
    }

    /// Get path to KWS models root directory
    pub fn kws_models_root(&self) -> PathBuf {
        self.models_dir().join("kws")
    }

    /// Get path to a specific KWS model directory by model_id
    pub fn kws_model_dir(&self, model_id: &str) -> PathBuf {
        self.kws_models_root().join(model_id)
    }

    /// Get path to KWS registry file
    pub fn kws_registry(&self) -> PathBuf {
        self.models_dir().join("kws_registry.json")
    }

    /// Get path to voiceprints directory
    pub fn voiceprints_dir(&self) -> PathBuf {
        self.data.join("voiceprints")
    }

    /// Get path to speaker embedding model directory
    pub fn speaker_model_file(&self) -> PathBuf {
        self.models_dir()
            .join("spk")
            .join("ecapa-tdnn-16k")
            .join("model.onnx")
    }

    /// Get path to profiles directory (alias for voiceprints)
    pub fn profiles_dir(&self) -> PathBuf {
        self.voiceprints_dir()
    }

    /// Get path to logs directory
    #[allow(dead_code)]
    pub fn logs_dir(&self) -> PathBuf {
        self.cache.join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_creation() {
        let paths = AppPaths::new().expect("Failed to create paths");

        // Verify paths are not empty
        assert!(!paths.config.as_os_str().is_empty());
        assert!(!paths.data.as_os_str().is_empty());
        assert!(!paths.cache.as_os_str().is_empty());
    }
}
