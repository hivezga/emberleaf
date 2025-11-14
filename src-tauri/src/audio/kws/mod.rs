//! Keyword Spotting (KWS) - Wake-word detection
//!
//! This module provides wake-word detection using either:
//! - Stub implementation (default): Energy-based heuristic for development
//! - Real implementation (kws_real feature): Sherpa-ONNX Zipformer KWS
//!
//! Thread Safety:
//! All FFI pointers and non-Send types are confined to a worker thread.
//! Communication happens via crossbeam channels.

use serde::{Deserialize, Serialize};

// Always compile stub for fallback support
pub mod stub;

// Conditional compilation: expose real implementation if feature enabled
cfg_if::cfg_if! {
    if #[cfg(feature = "kws_real")] {
        pub mod real;
    }
}

/// Unified KWS worker that can hold either real or stub implementation
pub enum KwsWorker {
    #[cfg(feature = "kws_real")]
    Real(real::KwsWorker),
    Stub(stub::KwsWorker),
}

impl KwsWorker {
    /// Start the appropriate KWS worker based on configuration
    pub fn start(
        app_handle: tauri::AppHandle,
        paths: crate::paths::AppPaths,
        config: KwsConfig,
        vad_config: crate::audio::vad::VadConfig,
        audio_config: crate::audio::AudioConfig,
    ) -> anyhow::Result<Self> {
        #[cfg(feature = "kws_real")]
        {
            // Extract model_id from config, default to "default" if not set
            let model_id = config
                .model_id
                .clone()
                .unwrap_or_else(|| "default".to_string());
            real::KwsWorker::start(
                app_handle,
                paths,
                config,
                vad_config,
                audio_config,
                model_id,
            )
            .map(KwsWorker::Real)
        }
        #[cfg(not(feature = "kws_real"))]
        {
            stub::KwsWorker::start(app_handle, paths, config, vad_config, audio_config)
                .map(KwsWorker::Stub)
        }
    }

    /// Start stub worker directly (for fallback when real fails)
    pub fn start_stub(
        app_handle: tauri::AppHandle,
        paths: crate::paths::AppPaths,
        config: KwsConfig,
        vad_config: crate::audio::vad::VadConfig,
        audio_config: crate::audio::AudioConfig,
    ) -> anyhow::Result<Self> {
        stub::KwsWorker::start(app_handle, paths, config, vad_config, audio_config)
            .map(KwsWorker::Stub)
    }
}

/// KWS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KwsConfig {
    pub keyword: String,
    pub score_threshold: f32,
    pub refractory_ms: u64,
    pub endpoint_ms: u64,
    pub provider: String,
    pub max_active_paths: usize,
    pub enabled: bool,
    /// Model ID for real KWS (None = use stub)
    #[serde(default)]
    pub model_id: Option<String>,
    /// Current mode: "stub" or "real"
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "stub".to_string()
}

impl Default for KwsConfig {
    fn default() -> Self {
        Self {
            keyword: "hey ember".to_string(),
            score_threshold: 0.60,
            refractory_ms: 1200,
            endpoint_ms: 300,
            provider: "cpu".to_string(),
            max_active_paths: 4,
            enabled: true,
            model_id: None,
            mode: "stub".to_string(),
        }
    }
}

/// Sensitivity presets
#[derive(Debug, Clone)]
pub enum Sensitivity {
    Low,
    Balanced,
    High,
}

impl Sensitivity {
    pub fn threshold(&self) -> f32 {
        match self {
            Sensitivity::Low => 0.70,
            Sensitivity::Balanced => 0.60,
            Sensitivity::High => 0.50,
        }
    }

    pub fn endpoint_ms(&self) -> u64 {
        match self {
            Sensitivity::Low => 350,
            Sensitivity::Balanced => 300,
            Sensitivity::High => 250,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Sensitivity::Low),
            "balanced" => Some(Sensitivity::Balanced),
            "high" => Some(Sensitivity::High),
            _ => None,
        }
    }
}

/// Wake-word detection event
#[derive(Debug, Clone, Serialize)]
pub struct WakeWordEvent {
    pub keyword: String,
    pub score: f32,
}
