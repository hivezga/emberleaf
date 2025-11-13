// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod display_backend;
mod ffi;
mod model_manager;
mod paths;
mod preflight;
mod registry;
mod validation;
mod voice;

use audio::kws::{KwsConfig, Sensitivity};
use audio::monitor::MicMonitor;
use audio::runtime::AudioRuntime;
use audio::vad::VadConfig;
use audio::AudioConfig;
use paths::AppPaths;
#[cfg(feature = "kws_real")]
use registry::verify_onnx_set;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use voice::{
    BiometricsConfig, EnrollmentProgress, ProfileInfo, SpeakerBiometrics, VerificationResult,
};

use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

// SEC-001B: Import validators for command input validation
use validation::{
    emit_validation_error, validate_device_name, validate_duration_ms, validate_frequency_hz_f32,
    validate_gain,
};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub kws: KwsConfig,
    pub vad: VadConfig,
    pub biometrics: BiometricsConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub focus_ring_contrast_min: f32,
    pub min_touch_target_px: u32,
    /// Remember mic monitor state across app restarts (default: false)
    #[serde(default)]
    pub persist_monitor_state: bool,
    /// Last monitor state (used when persist_monitor_state is true)
    #[serde(default)]
    pub monitor_was_on: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            kws: KwsConfig::default(),
            vad: VadConfig::default(),
            biometrics: BiometricsConfig::default(),
            ui: UiConfig {
                focus_ring_contrast_min: 3.0,
                min_touch_target_px: 32,
                persist_monitor_state: false,
                monitor_was_on: false,
            },
        }
    }
}

impl AppConfig {
    /// Load config from file or create default
    pub fn load_or_create(path: &std::path::Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            log::info!("Config loaded from: {}", path.display());
            Ok(config)
        } else {
            let config = Self::default();
            let toml_str = toml::to_string_pretty(&config)?;
            fs::write(path, toml_str)?;
            log::info!("Default config created at: {}", path.display());
            Ok(config)
        }
    }
}

/// Application state
struct AppState {
    paths: AppPaths,
    config: Arc<Mutex<AppConfig>>,
    audio_runtime: Arc<Mutex<Option<AudioRuntime>>>,
    speaker_biometrics: Arc<Mutex<Option<SpeakerBiometrics>>>,
    mic_monitor: Arc<Mutex<Option<MicMonitor>>>,
    model_manager: Arc<tokio::sync::Mutex<model_manager::ModelManager>>,
    /// Reentrancy guard for restart_audio_capture
    restart_in_progress: AtomicBool,
    /// Remember if monitor was active before restart (for resume)
    monitor_was_active: Arc<Mutex<bool>>,
    /// Last restart timestamp (milliseconds since UNIX epoch)
    last_restart_ms: Arc<Mutex<u64>>,
}

/// Tauri command: Set KWS sensitivity (runtime only, not persisted)
#[tauri::command]
async fn kws_set_sensitivity(level: String, state: State<'_, AppState>) -> Result<String, String> {
    let sensitivity = Sensitivity::from_str(&level)
        .ok_or_else(|| format!("Invalid sensitivity level: {}", level))?;

    // Update config in memory (not saved to disk)
    {
        let mut config = state.config.lock().unwrap();
        config.kws.score_threshold = sensitivity.threshold();
        config.kws.endpoint_ms = sensitivity.endpoint_ms();
    }

    log::info!("Sensitivity set to: {} (not persisted)", level);
    Ok(format!(
        "Sensitivity set to {}: threshold={:.2}, endpoint={}ms (call save_preferences to persist)",
        level,
        sensitivity.threshold(),
        sensitivity.endpoint_ms()
    ))
}

/// Tauri command: Set VAD threshold (runtime only, not persisted)
#[tauri::command]
async fn vad_set_threshold(threshold: f32, state: State<'_, AppState>) -> Result<String, String> {
    // SEC-001: Validate input range
    let validated_threshold = validation::validate_vad_threshold(threshold)
        .map_err(|e| e.to_string())?;

    // Update config in memory
    {
        let mut config = state.config.lock().unwrap();
        config.vad.threshold = validated_threshold;
    }

    log::info!("VAD threshold set to: {} (not persisted)", validated_threshold);
    Ok(format!(
        "VAD threshold set to: {} (call save_preferences to persist)",
        validated_threshold
    ))
}

/// Tauri command: Save current configuration to disk
#[tauri::command]
async fn save_preferences(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().unwrap().clone();
    let config_path = state.paths.config_file();
    let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;

    // SEC-001: Write config file with secure permissions (owner-only: 0600)
    fs::write(&config_path, toml_str).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o600); // Owner read/write only
        fs::set_permissions(&config_path, perms).map_err(|e| e.to_string())?;
    }

    log::info!("Preferences saved to: {} (secure perms)", config_path.display());
    Ok(format!("Preferences saved to: {}", config_path.display()))
}

/// Tauri command: Check if KWS is enabled
#[tauri::command]
async fn kws_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.lock().unwrap();
    Ok(config.kws.enabled)
}

// ===== KWS MODEL MANAGEMENT COMMANDS =====

/// Helper function to restart audio capture (internal use)
async fn restart_audio_capture_internal(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Reuse existing restart logic but return simpler Result
    restart_audio_capture(state, app_handle)
        .await
        .map(|_| ())
}

/// KWS status response
#[derive(Debug, Clone, Serialize)]
struct KwsStatus {
    mode: String,
    model_id: Option<String>,
    keyword: String,
    lang: Option<String>,
    enabled: bool,
}

/// Tauri command: Get KWS status
#[tauri::command]
async fn kws_status(state: State<'_, AppState>) -> Result<KwsStatus, String> {
    // Clone config data first (don't hold lock across await)
    let (mode, model_id_opt, keyword, enabled) = {
        let config = state.config.lock().unwrap();
        (
            config.kws.mode.clone(),
            config.kws.model_id.clone(),
            config.kws.keyword.clone(),
            config.kws.enabled,
        )
    };

    // Get model info if available (can await now)
    let lang = if let Some(ref model_id) = model_id_opt {
        let manager = state.model_manager.lock().await;
        if let Ok(registry) = manager.registry() {
            registry.get_model(model_id).map(|e| e.lang.clone())
        } else {
            None
        }
    } else {
        None
    };

    Ok(KwsStatus {
        mode,
        model_id: model_id_opt,
        keyword,
        lang,
        enabled,
    })
}

/// Tauri command: List available KWS models from registry
#[tauri::command]
async fn kws_list_models(
    state: State<'_, AppState>,
) -> Result<Vec<model_manager::KwsModelEntry>, String> {
    let manager = state.model_manager.lock().await;
    let registry = manager.registry().map_err(|e| e.to_string())?;

    let mut models: Vec<_> = registry
        .models
        .iter()
        .map(|(id, entry)| {
            let mut e = entry.clone();
            e.description = format!("{} ({})", id, e.description);
            e
        })
        .collect();

    // Sort by language and wakeword
    models.sort_by(|a, b| a.lang.cmp(&b.lang).then(a.wakeword.cmp(&b.wakeword)));

    Ok(models)
}

/// Tauri command: Download a KWS model (without enabling)
#[tauri::command]
async fn kws_download_model(
    model_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate model_id
    model_manager::ModelManager::validate_model_id(&model_id).map_err(|e| e.to_string())?;

    // Check if already downloaded
    let manager = state.model_manager.lock().await;
    if manager.is_model_ready(&model_id).map_err(|e| e.to_string())? {
        return Ok(format!("Model '{}' is already downloaded and verified", model_id));
    }

    log::info!("Downloading model: {}", model_id);

    // Download model
    manager
        .download_model(&app_handle, &model_id)
        .await
        .map_err(|e| {
            let error_msg = format!("Model download failed: {}", e);
            log::error!("{}", error_msg);
            let _ = app_handle.emit("audio:error", error_msg.clone());
            error_msg
        })?;

    // Verify model
    let registry = manager.registry().map_err(|e| e.to_string())?;
    let entry = registry
        .get_model(&model_id)
        .ok_or_else(|| format!("Model '{}' not found in registry", model_id))?;

    let is_valid = manager
        .verify_model(&model_id, &entry.sha256)
        .map_err(|e| e.to_string())?;

    if !is_valid {
        // Verification failed - remove corrupted model
        manager.remove_model(&model_id).ok();
        let error_msg = format!("Model verification failed for '{}'", model_id);
        log::error!("{}", error_msg);
        let _ = app_handle.emit("kws:model_verify_failed", &model_id);
        let _ = app_handle.emit("audio:error", error_msg.clone());
        return Err(error_msg);
    }

    log::info!("Model '{}' verified successfully", model_id);
    let _ = app_handle.emit("kws:model_verified", &model_id);

    Ok(format!("Model '{}' downloaded and verified", model_id))
}

/// Tauri command: Enable real KWS with a specific model
#[tauri::command]
async fn kws_enable(
    model_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate model_id
    model_manager::ModelManager::validate_model_id(&model_id).map_err(|e| e.to_string())?;

    #[cfg(not(feature = "kws_real"))]
    {
        return Err(
            "Real KWS not available: app was built without kws_real feature".to_string(),
        );
    }

    #[cfg(feature = "kws_real")]
    {
        // Check if model is ready, download if needed
        {
            let manager = state.model_manager.lock().await;
            if !manager.is_model_ready(&model_id).map_err(|e| e.to_string())? {
                log::info!("Model '{}' not found, downloading...", model_id);
                drop(manager);

                // Download and verify
                kws_download_model(model_id.clone(), app_handle.clone(), state.clone()).await?;
            }
        }

        // Update config
        {
            let mut config = state.config.lock().unwrap();
            config.kws.model_id = Some(model_id.clone());
            config.kws.mode = "real".to_string();
            config.kws.enabled = true;
        }

        // Restart audio runtime with real KWS
        restart_audio_capture_internal(app_handle.clone(), state.clone()).await?;

        log::info!("Real KWS enabled with model: {}", model_id);
        let _ = app_handle.emit("kws:enabled", &model_id);

        Ok(format!("Real KWS enabled with model '{}'", model_id))
    }
}

/// Tauri command: Disable real KWS and return to stub
#[tauri::command]
async fn kws_disable(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Update config to use stub
    {
        let mut config = state.config.lock().unwrap();
        config.kws.mode = "stub".to_string();
        config.kws.model_id = None;
        config.kws.enabled = true; // Keep KWS enabled, just switch to stub
    }

    // Restart audio runtime with stub KWS
    restart_audio_capture_internal(app_handle.clone(), state.clone()).await?;

    log::info!("KWS disabled, returned to stub mode");
    let _ = app_handle.emit("kws:disabled", ());

    Ok("KWS disabled, returned to stub mode".to_string())
}

/// Tauri command: Get current configuration
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().unwrap().clone();
    Ok(config)
}

// ===== DEVICE MANAGEMENT COMMANDS =====

/// Tauri command: List all available input devices
#[tauri::command]
async fn list_input_devices() -> Result<Vec<audio::DeviceInfo>, String> {
    audio::list_input_devices().map_err(|e| e.to_string())
}

/// Tauri command: Get the current input device name
#[tauri::command]
async fn current_input_device(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.audio.device_name.clone())
}

/// Tauri command: Set the input device (requires restart of audio capture)
#[tauri::command]
async fn set_input_device(
    name: String,
    persist: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // SEC-001B: Validate device name if provided
    if !name.is_empty() {
        if let Err(e) = validate_device_name(&name) {
            emit_validation_error(
                &app,
                "invalid_device_name",
                "device_name",
                &e.to_string(),
                Some(serde_json::json!(name)),
            );
            return Err(e.to_string());
        }
    }

    // Look up stable_id for the device
    let stable_id = if !name.is_empty() {
        audio::list_input_devices().ok().and_then(|devices| {
            devices
                .into_iter()
                .find(|d| d.name == name)
                .and_then(|d| d.stable_id)
        })
    } else {
        None
    };

    // Update in-memory config
    {
        let mut config = state.config.lock().unwrap();
        config.audio.device_name = if name.is_empty() {
            None
        } else {
            Some(name.clone())
        };
        config.audio.stable_input_id = stable_id;
    }

    // Save to disk if requested
    if persist {
        let config = state.config.lock().unwrap().clone();
        let config_path = state.paths.config_file();
        let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
        fs::write(&config_path, toml_str).map_err(|e| e.to_string())?;
        log::info!("Device config saved to: {}", config_path.display());
    }

    Ok(format!(
        "Device set to '{}'. Restart the app to apply changes.",
        if name.is_empty() {
            "default".to_string()
        } else {
            name
        }
    ))
}

/// Tauri command: Get audio pipeline debug information
#[tauri::command]
async fn get_audio_debug(state: State<'_, AppState>) -> Result<audio::AudioDebugInfo, String> {
    let config = state.config.lock().unwrap();
    Ok(audio::AudioDebugInfo {
        processing_rate: config.audio.sample_rate_hz,
        processing_channels: audio::TARGET_CHANNELS,
        frame_ms: config.audio.frame_ms,
        hop_ms: config.audio.hop_ms,
        samples_per_frame: config.audio.samples_per_frame(),
        samples_per_hop: config.audio.samples_per_hop(),
        input_device: config.audio.device_name.clone(),
        output_device: config.audio.output_device_name.clone(),
    })
}

/// Comprehensive audio snapshot for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSnapshot {
    pub debug_info: audio::AudioDebugInfo,
    pub input_devices: Vec<audio::DeviceInfo>,
    pub output_devices: Vec<audio::DeviceInfo>,
    pub selected_input_device: Option<String>,
    pub selected_output_device: Option<String>,
    pub monitor_active: bool,
    pub last_restart_ms: u64,
    pub timestamp_ms: u64,
}

/// Tauri command: Get comprehensive audio snapshot for diagnostics
#[tauri::command]
async fn get_audio_snapshot(state: State<'_, AppState>) -> Result<AudioSnapshot, String> {
    // Get current debug info
    let config = state.config.lock().unwrap();
    let debug_info = audio::AudioDebugInfo {
        processing_rate: config.audio.sample_rate_hz,
        processing_channels: audio::TARGET_CHANNELS,
        frame_ms: config.audio.frame_ms,
        hop_ms: config.audio.hop_ms,
        samples_per_frame: config.audio.samples_per_frame(),
        samples_per_hop: config.audio.samples_per_hop(),
        input_device: config.audio.device_name.clone(),
        output_device: config.audio.output_device_name.clone(),
    };
    drop(config);

    // Get device lists
    let input_devices = audio::list_input_devices().unwrap_or_else(|e| {
        log::warn!("Failed to list input devices: {}", e);
        Vec::new()
    });
    let output_devices = audio::list_output_devices().unwrap_or_else(|e| {
        log::warn!("Failed to list output devices: {}", e);
        Vec::new()
    });

    // Get selected devices
    let config = state.config.lock().unwrap();
    let selected_input_device = config.audio.device_name.clone();
    let selected_output_device = config.audio.output_device_name.clone();
    drop(config);

    // Get monitor state
    let monitor_active = state.mic_monitor.lock().unwrap().is_some();

    // Get last restart timestamp
    let last_restart_ms = *state.last_restart_ms.lock().unwrap();

    // Current timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    Ok(AudioSnapshot {
        debug_info,
        input_devices,
        output_devices,
        selected_input_device,
        selected_output_device,
        monitor_active,
        last_restart_ms,
        timestamp_ms,
    })
}

/// Tauri command: List all available output devices
#[tauri::command]
async fn list_output_devices() -> Result<Vec<audio::DeviceInfo>, String> {
    audio::list_output_devices().map_err(|e| e.to_string())
}

/// Tauri command: Get the current output device name
#[tauri::command]
async fn current_output_device(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.audio.output_device_name.clone())
}

/// Tauri command: Set the output device (for future TTS)
#[tauri::command]
async fn set_output_device(
    name: String,
    persist: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // SEC-001B: Validate device name if provided
    if !name.is_empty() {
        if let Err(e) = validate_device_name(&name) {
            emit_validation_error(
                &app,
                "invalid_device_name",
                "device_name",
                &e.to_string(),
                Some(serde_json::json!(name)),
            );
            return Err(e.to_string());
        }
    }

    // Look up stable_id for the device
    let stable_id = if !name.is_empty() {
        audio::list_output_devices().ok().and_then(|devices| {
            devices
                .into_iter()
                .find(|d| d.name == name)
                .and_then(|d| d.stable_id)
        })
    } else {
        None
    };

    // Update in-memory config
    {
        let mut config = state.config.lock().unwrap();
        config.audio.output_device_name = if name.is_empty() {
            None
        } else {
            Some(name.clone())
        };
        config.audio.stable_output_id = stable_id;
    }

    // Save to disk if requested
    if persist {
        let config = state.config.lock().unwrap().clone();
        let config_path = state.paths.config_file();
        let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
        fs::write(&config_path, toml_str).map_err(|e| e.to_string())?;
        log::info!("Output device config saved to: {}", config_path.display());
    }

    Ok(format!(
        "Output device set to '{}'. Will be used for TTS when available.",
        if name.is_empty() {
            "default".to_string()
        } else {
            name
        }
    ))
}

/// Response structure for restart_audio_capture command
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RestartResponse {
    ok: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    elapsed_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

/// Tauri command: Restart audio capture and KWS worker (with reentrancy guard)
#[tauri::command]
async fn restart_audio_capture(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<RestartResponse, String> {
    let start_time = std::time::Instant::now();

    // Reentrancy guard: check if restart is already in progress
    if state
        .restart_in_progress
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        log::warn!("Restart already in progress, ignoring duplicate call");

        // Emit blocked event
        let _ = app_handle.emit("audio:restart_blocked", ());

        return Ok(RestartResponse {
            ok: false,
            message: "Restart already in progress, please wait...".to_string(),
            elapsed_ms: None,
            reason: Some("in_progress".to_string()),
        });
    }

    // Ensure flag is cleared on exit
    let _guard = ResetOnDrop(&state.restart_in_progress);

    log::info!("Restarting audio capture...");

    // 1. Check if mic monitor is active and stop it
    let monitor_was_active = state.mic_monitor.lock().unwrap().is_some();
    if monitor_was_active {
        log::info!("Stopping mic monitor before restart...");
        if let Some(monitor) = state.mic_monitor.lock().unwrap().take() {
            monitor.stop();
        }
        *state.monitor_was_active.lock().unwrap() = true;
    }

    // 2. Stop current runtime
    if let Some(runtime) = state.audio_runtime.lock().unwrap().take() {
        runtime.stop();
    }

    // 3. Read latest config
    let config = state.config.lock().unwrap().clone();
    let paths = state.paths.clone();

    // 4. Start fresh runtime
    let result = match audio::runtime::AudioRuntime::start(
        app_handle.clone(),
        paths,
        config.audio.clone(),
        config.kws.clone(),
        config.vad.clone(),
    ) {
        Ok((runtime, _stop_rx)) => {
            *state.audio_runtime.lock().unwrap() = Some(runtime);

            // Update restart timestamp
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            *state.last_restart_ms.lock().unwrap() = now_ms;

            let elapsed_ms = start_time.elapsed().as_millis() as u64;
            let device_name = config
                .audio
                .device_name
                .clone()
                .unwrap_or_else(|| "default".to_string());

            log::info!("✓ Audio restarted successfully in {}ms", elapsed_ms);

            // Emit success event
            #[derive(serde::Serialize, Clone)]
            struct RestartOkPayload {
                device: String,
                elapsed_ms: u64,
            }
            let _ = app_handle.emit(
                "audio:restart_ok",
                RestartOkPayload {
                    device: device_name.clone(),
                    elapsed_ms,
                },
            );

            Ok(RestartResponse {
                ok: true,
                message: format!("Reconnected to {}", device_name),
                elapsed_ms: Some(elapsed_ms),
                reason: None,
            })
        }
        Err(e) => {
            log::error!("Failed to restart audio: {}", e);

            // Emit friendly error event
            let friendly = audio::friendly_audio_error(&e);
            #[derive(serde::Serialize, Clone)]
            struct AudioErrorPayload {
                code: String,
                message: String,
            }
            let _ = app_handle.emit(
                "audio:error",
                AudioErrorPayload {
                    code: friendly.code,
                    message: friendly.message.clone(),
                },
            );

            Err(format!("Audio restart failed: {}", friendly.message))
        }
    };

    // 5. Optionally resume mic monitor if it was active
    if monitor_was_active && result.is_ok() {
        log::info!("Resuming mic monitor after restart...");
        let input_device = config.audio.device_name.clone();
        let output_device = config.audio.output_device_name.clone();

        // Feedback-loop check
        if input_device == output_device && input_device.is_some() {
            log::warn!("Cannot resume monitor: input and output are the same device");
            *state.monitor_was_active.lock().unwrap() = false;
        } else {
            match audio::monitor::MicMonitor::start(input_device, output_device, 0.15) {
                Ok(monitor) => {
                    *state.mic_monitor.lock().unwrap() = Some(monitor);
                    log::info!("✓ Mic monitor resumed");
                }
                Err(e) => {
                    log::error!("Failed to resume mic monitor: {}", e);
                    *state.monitor_was_active.lock().unwrap() = false;
                }
            }
        }
    }

    result
}

/// RAII guard to reset restart flag on scope exit
struct ResetOnDrop<'a>(&'a AtomicBool);
impl Drop for ResetOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Tauri command: Play a test tone on the output device
#[tauri::command]
async fn play_test_tone(
    device_name: Option<String>,
    frequency_hz: Option<f32>,
    duration_ms: Option<u32>,
    volume: Option<f32>,
    simple_mode: Option<bool>,
    app: AppHandle,
) -> Result<String, String> {
    // SEC-001B: Validate optional device name
    if let Some(name) = device_name.as_ref() {
        if let Err(e) = validate_device_name(name) {
            emit_validation_error(
                &app,
                "invalid_device_name",
                "device_name",
                &e.to_string(),
                Some(serde_json::json!(name)),
            );
            return Err(e.to_string());
        }
    }

    // SEC-001B: Validate frequency (default 440Hz if not provided)
    let freq = frequency_hz.unwrap_or(440.0);
    if let Err(e) = validate_frequency_hz_f32(freq) {
        emit_validation_error(
            &app,
            "invalid_frequency",
            "frequency_hz",
            &e.to_string(),
            Some(serde_json::json!(freq)),
        );
        return Err(e.to_string());
    }

    // SEC-001B: Validate duration (default 500ms if not provided)
    let mut dur = duration_ms.unwrap_or(500);
    if let Err(e) = validate_duration_ms(dur) {
        emit_validation_error(
            &app,
            "invalid_duration",
            "duration_ms",
            &e.to_string(),
            Some(serde_json::json!(dur)),
        );
        return Err(e.to_string());
    }

    let mut vol = volume.unwrap_or(0.2);

    // Safety caps for Simple mode: 300ms max, -12 dBFS (0.25 amplitude)
    let is_simple = simple_mode.unwrap_or(false);
    if is_simple {
        dur = dur.min(300);
        vol = vol.min(0.25); // -12 dBFS ≈ 0.25 amplitude
    }

    log::info!(
        "Playing test tone: {}Hz, {}ms, volume={:.2} (simple_mode={})",
        freq,
        dur,
        vol,
        is_simple
    );

    let result = audio::test_tone::play_tone(device_name.clone(), freq, dur, vol);

    if result.is_ok() {
        // Emit test tone event
        #[derive(serde::Serialize, Clone)]
        struct TestTonePayload {
            device: String,
            frequency_hz: f32,
            duration_ms: u32,
            volume: f32,
        }

        let _ = app.emit(
            "audio:test_tone_played",
            TestTonePayload {
                device: device_name.clone().unwrap_or_else(|| "default".to_string()),
                frequency_hz: freq,
                duration_ms: dur,
                volume: vol,
            },
        );
    }

    result
        .map(|_| {
            format!(
                "✓ Played {}Hz tone for {}ms at {:.0}% volume on {}",
                freq,
                dur,
                vol * 100.0,
                device_name.unwrap_or_else(|| "default device".to_string())
            )
        })
        .map_err(|e| format!("Test tone failed: {:#}", e))
}

/// Tauri command: Start microphone monitoring
#[tauri::command]
async fn start_mic_monitor(
    app: AppHandle,
    state: State<'_, AppState>,
    gain: Option<f32>,
) -> Result<String, String> {
    // SEC-001B: Validate gain (default 0.15 if not provided)
    let gain = gain.unwrap_or(0.15);
    if let Err(e) = validate_gain(gain) {
        emit_validation_error(
            &app,
            "invalid_gain",
            "gain",
            &e.to_string(),
            Some(serde_json::json!(gain)),
        );
        return Err(e.to_string());
    }

    // Stop existing monitor if any
    if let Some(monitor) = state.mic_monitor.lock().unwrap().take() {
        monitor.stop();
    }

    let config = state.config.lock().unwrap();
    let input_device = config.audio.device_name.clone();
    let output_device = config.audio.output_device_name.clone();
    let persist_enabled = config.ui.persist_monitor_state;
    drop(config);

    log::info!("Starting mic monitor with gain={:.2}", gain);

    match audio::monitor::MicMonitor::start(input_device.clone(), output_device.clone(), gain) {
        Ok(monitor) => {
            *state.mic_monitor.lock().unwrap() = Some(monitor);

            // Persist monitor state if enabled
            if persist_enabled {
                let mut config = state.config.lock().unwrap();
                config.ui.monitor_was_on = true;
                drop(config);

                let config = state.config.lock().unwrap().clone();
                let config_path = state.paths.config_file();
                let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
                let _ = fs::write(&config_path, toml_str);
                log::debug!("Monitor state persisted: ON");
            }

            Ok(format!(
                "✓ Mic monitor active: {} → {} (gain={:.0}%)",
                input_device.unwrap_or_else(|| "default".to_string()),
                output_device.unwrap_or_else(|| "default".to_string()),
                gain * 100.0
            ))
        }
        Err(e) => {
            log::error!("Failed to start mic monitor: {}", e);
            Err(format!("Mic monitor failed: {:#}", e))
        }
    }
}

/// Tauri command: Stop microphone monitoring
#[tauri::command]
async fn stop_mic_monitor(state: State<'_, AppState>) -> Result<String, String> {
    if let Some(monitor) = state.mic_monitor.lock().unwrap().take() {
        monitor.stop();

        // Persist monitor state if enabled
        let persist_enabled = state.config.lock().unwrap().ui.persist_monitor_state;
        if persist_enabled {
            let mut config = state.config.lock().unwrap();
            config.ui.monitor_was_on = false;
            drop(config);

            let config = state.config.lock().unwrap().clone();
            let config_path = state.paths.config_file();
            let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
            let _ = fs::write(&config_path, toml_str);
            log::debug!("Monitor state persisted: OFF");
        }

        Ok("✓ Mic monitor stopped".to_string())
    } else {
        Ok("Mic monitor was not active".to_string())
    }
}

/// Tauri command: Set persist monitor state preference
#[tauri::command]
async fn set_persist_monitor_state(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<String, String> {
    let mut config = state.config.lock().unwrap();
    config.ui.persist_monitor_state = enabled;
    drop(config);

    // Save to disk
    let config = state.config.lock().unwrap().clone();
    let config_path = state.paths.config_file();
    let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, toml_str).map_err(|e| e.to_string())?;

    log::info!("persist_monitor_state set to: {}", enabled);
    Ok(format!(
        "Monitor persistence {}",
        if enabled { "enabled" } else { "disabled" }
    ))
}

/// Tauri command: Auto-probe and suggest best input device
#[tauri::command]
async fn suggest_input_device(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<audio::probe::ProbeResult, String> {
    log::info!("Starting auto-probe for input device suggestion...");

    // Emit start event
    let _ = app.emit("audio:auto_probe_started", ());

    let config = state.config.lock().unwrap();
    let current_device = config.audio.device_name.clone();
    drop(config);

    match audio::probe::suggest_input_device(current_device.as_deref()) {
        Ok(result) => {
            log::info!("Auto-probe result: {:?}", result);

            // Emit suggestion event if we have one
            if result.suggested.is_some() {
                #[derive(serde::Serialize, Clone)]
                struct SuggestionPayload {
                    device: String,
                    reason: String,
                }

                let _ = app.emit(
                    "audio:auto_probe_suggestion",
                    SuggestionPayload {
                        device: result.suggested.clone().unwrap(),
                        reason: result.reason.clone(),
                    },
                );
            }

            Ok(result)
        }
        Err(e) => {
            log::error!("Auto-probe failed: {}", e);
            Err(format!("Auto-probe failed: {:#}", e))
        }
    }
}

// ===== BIOMETRICS COMMANDS =====

/// Tauri command: Start enrollment for a user
#[tauri::command]
async fn enroll_start(user: String, state: State<'_, AppState>) -> Result<(), String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics.enroll_start(user).map_err(|e| e.to_string())
}

/// Tauri command: Add an enrollment sample (from audio buffer)
/// Note: The frontend should capture audio and send it as f32 samples
#[tauri::command]
async fn enroll_add_sample(
    samples: Vec<f32>,
    state: State<'_, AppState>,
) -> Result<EnrollmentProgress, String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics
        .enroll_add_sample(&samples)
        .map_err(|e| e.to_string())
}

/// Tauri command: Finalize enrollment and save voiceprint
#[tauri::command]
async fn enroll_finalize(state: State<'_, AppState>) -> Result<ProfileInfo, String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics.enroll_finalize().map_err(|e| e.to_string())
}

/// Tauri command: Cancel ongoing enrollment
#[tauri::command]
async fn enroll_cancel(state: State<'_, AppState>) -> Result<(), String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics.enroll_cancel();
    Ok(())
}

/// Tauri command: Verify a speaker
#[tauri::command]
async fn verify_speaker(
    user: String,
    samples: Vec<f32>,
    state: State<'_, AppState>,
) -> Result<VerificationResult, String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics
        .verify(&user, &samples)
        .map_err(|e| e.to_string())
}

/// Tauri command: Check if a profile exists
#[tauri::command]
async fn profile_exists(user: String, state: State<'_, AppState>) -> Result<bool, String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    Ok(biometrics.profile_exists(&user))
}

/// Tauri command: Delete a user profile
#[tauri::command]
async fn delete_profile(user: String, state: State<'_, AppState>) -> Result<(), String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics.delete_profile(&user).map_err(|e| e.to_string())
}

/// Tauri command: List all enrolled users
#[tauri::command]
async fn list_profiles(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let biometrics = state.speaker_biometrics.lock().unwrap();
    let biometrics = biometrics
        .as_ref()
        .ok_or_else(|| "Speaker biometrics not initialized".to_string())?;

    biometrics.list_profiles().map_err(|e| e.to_string())
}

/// Initialize speaker biometrics
fn initialize_biometrics(
    paths: &AppPaths,
    config: &AppConfig,
) -> anyhow::Result<Option<SpeakerBiometrics>> {
    log::info!("Initializing speaker biometrics...");

    // Get speaker model path
    let model_path = paths.speaker_model_file();
    if !model_path.exists() {
        log::warn!("Speaker model not found: {}", model_path.display());
        log::warn!("Speaker biometrics will not be available");
        return Ok(None);
    }

    // Get profiles directory
    let profiles_dir = paths.profiles_dir();

    // Create biometrics system
    let biometrics = SpeakerBiometrics::new(
        model_path,
        profiles_dir,
        config.biometrics.clone(),
        config.audio.sample_rate_hz,
    )?;

    log::info!("✓ Speaker biometrics initialized");

    Ok(Some(biometrics))
}

/// Initialize audio runtime
fn initialize_audio_runtime(
    paths: &AppPaths,
    config: &AppConfig,
    app_handle: AppHandle,
) -> anyhow::Result<Option<AudioRuntime>> {
    log::info!("Initializing audio runtime...");

    // Model verification for real KWS
    #[cfg(feature = "kws_real")]
    if config.kws.enabled {
        let model_dir = paths.kws_model_dir();
        if !model_dir.exists() {
            log::warn!("KWS model directory not found: {}", model_dir.display());
            log::warn!("Please download models to: {}", model_dir.display());
            log::warn!("Continuing with stub KWS...");
        } else {
            // Verify model integrity
            log::info!("Verifying KWS model integrity...");
            match verify_onnx_set(&model_dir) {
                Ok(results) => {
                    for (file, state) in results {
                        match state {
                            registry::VerificationState::Verified => {
                                log::info!("  ✓ {} - Verified", file);
                            }
                            registry::VerificationState::Unknown => {
                                log::warn!("  ? {} - Unknown (not in registry)", file);
                                if !state.is_safe() {
                                    log::error!("Model verification failed. Set EMVER_ALLOW_UNKNOWN_MODELS=1 to override.");
                                    log::warn!("Continuing with stub KWS...");
                                }
                            }
                            registry::VerificationState::Mismatch { expected, actual } => {
                                log::error!("  ✗ {} - Hash mismatch!", file);
                                log::error!("    Expected: {}", expected);
                                log::error!("    Actual:   {}", actual);
                                log::error!("Model file corrupted or modified: {}", file);
                                log::warn!("Continuing with stub KWS...");
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Model verification failed: {}", e);
                    log::warn!("Continuing with stub KWS...");
                }
            }
        }
    }

    // Start audio runtime
    match audio::runtime::AudioRuntime::start(
        app_handle,
        paths.clone(),
        config.audio.clone(),
        config.kws.clone(),
        config.vad.clone(),
    ) {
        Ok((runtime, _stop_rx)) => {
            if config.kws.enabled {
                log::info!(
                    "✓ Audio runtime started with wake-word: '{}'",
                    config.kws.keyword
                );
            } else {
                log::info!("✓ Audio runtime started (KWS disabled)");
            }
            Ok(Some(runtime))
        }
        Err(e) => {
            log::error!("Failed to start audio runtime: {}", e);
            Err(e)
        }
    }
}

/// Device health watcher - monitors configured devices and handles loss/fallback
async fn device_health_watcher(app_handle: AppHandle) {
    use tokio::time::{sleep, Duration};

    log::info!("Device health watcher started");

    loop {
        sleep(Duration::from_secs(2)).await;

        let state: State<AppState> = app_handle.state();
        let config = state.config.lock().unwrap().clone();

        // Check input device
        if let Some(ref device_name) = config.audio.device_name {
            let stable_id = config.audio.stable_input_id.as_ref();
            let exists = audio::check_input_device_exists(stable_id, Some(device_name));

            if !exists {
                log::warn!(
                    "Input device '{}' no longer available, attempting fallback",
                    device_name
                );

                // Emit device lost event
                if let Some(ref sid) = config.audio.stable_input_id {
                    #[derive(serde::Serialize, Clone)]
                    struct DeviceLostPayload {
                        kind: String,
                        previous: audio::DeviceId,
                    }
                    let _ = app_handle.emit(
                        "audio:device_lost",
                        DeviceLostPayload {
                            kind: "input".to_string(),
                            previous: sid.clone(),
                        },
                    );
                }

                // Attempt fallback by clearing device preference and restarting
                {
                    let mut cfg = state.config.lock().unwrap();
                    cfg.audio.device_name = None;
                    cfg.audio.stable_input_id = None;
                }

                // Trigger restart via internal restart logic
                match audio::runtime::AudioRuntime::start(
                    app_handle.clone(),
                    state.paths.clone(),
                    state.config.lock().unwrap().audio.clone(),
                    state.config.lock().unwrap().kws.clone(),
                    state.config.lock().unwrap().vad.clone(),
                ) {
                    Ok((runtime, _stop_rx)) => {
                        // Check if monitor was active before fallback
                        let monitor_was_active = state.mic_monitor.lock().unwrap().is_some();

                        // Stop old runtime and monitor
                        if let Some(old_runtime) = state.audio_runtime.lock().unwrap().take() {
                            old_runtime.stop();
                        }
                        if let Some(monitor) = state.mic_monitor.lock().unwrap().take() {
                            monitor.stop();
                        }

                        // Install new runtime
                        *state.audio_runtime.lock().unwrap() = Some(runtime);

                        log::info!("✓ Successfully fell back to default input device");

                        // Emit fallback success event
                        #[derive(serde::Serialize, Clone)]
                        struct FallbackOkPayload {
                            kind: String,
                            new_device: String,
                        }
                        let _ = app_handle.emit(
                            "audio:device_fallback_ok",
                            FallbackOkPayload {
                                kind: "input".to_string(),
                                new_device: "default".to_string(),
                            },
                        );

                        // Emit restart ok with fallback reason
                        #[derive(serde::Serialize, Clone)]
                        struct RestartOkPayload {
                            device: String,
                            elapsed_ms: u64,
                            reason: String,
                        }
                        let _ = app_handle.emit(
                            "audio:restart_ok",
                            RestartOkPayload {
                                device: "default".to_string(),
                                elapsed_ms: 0,
                                reason: "device_fallback".to_string(),
                            },
                        );

                        // Attempt to resume monitor if it was active, but only if safe
                        if monitor_was_active {
                            let cfg = state.config.lock().unwrap().clone();
                            let input_device = cfg.audio.device_name.clone();
                            let output_device = cfg.audio.output_device_name.clone();

                            // Safety check: prevent feedback loop
                            if input_device == output_device && input_device.is_some() {
                                log::warn!("Cannot resume monitor after fallback: input and output are the same device (feedback prevention)");

                                // Emit monitor guarded event
                                #[derive(serde::Serialize, Clone)]
                                struct MonitorGuardedPayload {
                                    reason: String,
                                }
                                let _ = app_handle.emit(
                                    "audio:monitor_guarded",
                                    MonitorGuardedPayload {
                                        reason: "feedback_risk".to_string(),
                                    },
                                );
                            } else {
                                // Safe to resume monitor
                                match audio::monitor::MicMonitor::start(
                                    input_device,
                                    output_device,
                                    0.15,
                                ) {
                                    Ok(monitor) => {
                                        *state.mic_monitor.lock().unwrap() = Some(monitor);
                                        log::info!("✓ Mic monitor resumed after fallback");
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to resume monitor after fallback: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to fallback after device loss: {}", e);

                        // Emit fallback failed event
                        #[derive(serde::Serialize, Clone)]
                        struct FallbackFailedPayload {
                            kind: String,
                            reason: String,
                        }
                        let _ = app_handle.emit(
                            "audio:device_fallback_failed",
                            FallbackFailedPayload {
                                kind: "input".to_string(),
                                reason: format!("{}", e),
                            },
                        );

                        // Emit friendly error
                        let friendly = audio::friendly_audio_error(&e);
                        #[derive(serde::Serialize, Clone)]
                        struct AudioErrorPayload {
                            code: String,
                            message: String,
                        }
                        let _ = app_handle.emit(
                            "audio:error",
                            AudioErrorPayload {
                                code: friendly.code,
                                message: friendly.message,
                            },
                        );
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Emberleaf starting...");

    // Configure display backend EARLY (Linux only)
    #[cfg(target_os = "linux")]
    {
        display_backend::apply_env(display_backend::DisplayBackend::Auto);
        display_backend::check_linux_dependencies();
    }

    // Initialize paths
    let paths = AppPaths::new().expect("Failed to initialize application paths");
    paths
        .ensure_directories()
        .expect("Failed to create application directories");

    // Load configuration
    let config_path = paths.config_file();
    let config = AppConfig::load_or_create(&config_path).expect("Failed to load configuration");

    log::info!("Configuration loaded");

    // Clone for the setup closure
    let paths_for_setup = paths.clone();
    let config_for_setup = config.clone();

    // Attempt to run app with crash-resilient fallback
    if let Err(e) = run_app(paths, config, paths_for_setup, config_for_setup).await {
        log::error!("Application failed to start: {}", e);
        std::process::exit(1);
    }
}

/// Run the Tauri application (extracted for fallback logic)
async fn run_app(
    paths: AppPaths,
    config: AppConfig,
    paths_for_setup: AppPaths,
    config_for_setup: AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize model manager
    let mut model_manager = model_manager::ModelManager::new(paths.models_dir());

    // Try to load registry (non-fatal if missing)
    let registry_path = paths.kws_registry();
    if registry_path.exists() {
        match model_manager.load_registry(&registry_path) {
            Ok(()) => log::info!("KWS registry loaded successfully"),
            Err(e) => log::warn!("Failed to load KWS registry: {}", e),
        }
    } else {
        log::warn!("KWS registry not found at: {}", registry_path.display());
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            paths: paths.clone(),
            config: Arc::new(Mutex::new(config)),
            audio_runtime: Arc::new(Mutex::new(None)),
            speaker_biometrics: Arc::new(Mutex::new(None)),
            mic_monitor: Arc::new(Mutex::new(None)),
            model_manager: Arc::new(tokio::sync::Mutex::new(model_manager)),
            restart_in_progress: AtomicBool::new(false),
            monitor_was_active: Arc::new(Mutex::new(false)),
            last_restart_ms: Arc::new(Mutex::new(0)),
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let paths_clone = paths_for_setup.clone();
            let config_clone = config_for_setup.clone();

            // Initialize biometrics and KWS asynchronously (state is now available)
            tokio::spawn(async move {
                // Initialize speaker biometrics (pass paths directly, no state access)
                match initialize_biometrics(&paths_clone, &config_clone) {
                    Ok(Some(biometrics)) => {
                        let state: State<AppState> = app_handle.state();
                        let mut speaker_biometrics = state.speaker_biometrics.lock().unwrap();
                        *speaker_biometrics = Some(biometrics);
                        log::info!("Speaker biometrics ready");
                    }
                    Ok(None) => {
                        log::info!("Speaker biometrics not started (model missing)");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize speaker biometrics: {}", e);
                        log::error!("Application will continue without voice biometrics");
                    }
                }

                // Initialize audio runtime
                match initialize_audio_runtime(&paths_clone, &config_clone, app_handle.clone()) {
                    Ok(Some(runtime)) => {
                        let state: State<AppState> = app_handle.state();
                        let mut audio_runtime = state.audio_runtime.lock().unwrap();
                        *audio_runtime = Some(runtime);
                        log::info!("Audio runtime ready");

                        // Auto-resume mic monitor if persistence is enabled and it was on
                        drop(audio_runtime);
                        if config_clone.ui.persist_monitor_state && config_clone.ui.monitor_was_on {
                            log::info!("Attempting to auto-resume mic monitor...");

                            let input_device = config_clone.audio.device_name.clone();
                            let output_device = config_clone.audio.output_device_name.clone();

                            // Safety check: prevent feedback
                            if input_device == output_device && input_device.is_some() {
                                log::warn!("Cannot auto-resume monitor: input and output are the same device (feedback prevention)");
                            } else {
                                match audio::monitor::MicMonitor::start(input_device, output_device, 0.15) {
                                    Ok(monitor) => {
                                        *state.mic_monitor.lock().unwrap() = Some(monitor);
                                        log::info!("✓ Mic monitor auto-resumed from persisted state");
                                    }
                                    Err(e) => {
                                        log::warn!("Failed to auto-resume mic monitor: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        log::info!("Audio runtime not started");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize audio runtime: {}", e);
                        log::error!("Application will continue without audio processing");
                    }
                }

                // Start device health watcher
                tokio::spawn(device_health_watcher(app_handle.clone()));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            preflight::run_preflight_checks,
            kws_set_sensitivity,
            kws_enabled,
            kws_status,
            kws_list_models,
            kws_download_model,
            kws_enable,
            kws_disable,
            vad_set_threshold,
            save_preferences,
            get_config,
            list_input_devices,
            current_input_device,
            set_input_device,
            get_audio_debug,
            get_audio_snapshot,
            list_output_devices,
            current_output_device,
            set_output_device,
            restart_audio_capture,
            play_test_tone,
            start_mic_monitor,
            stop_mic_monitor,
            set_persist_monitor_state,
            suggest_input_device,
            enroll_start,
            enroll_add_sample,
            enroll_finalize,
            enroll_cancel,
            verify_speaker,
            profile_exists,
            delete_profile,
            list_profiles
        ])
        .run(tauri::generate_context!())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
