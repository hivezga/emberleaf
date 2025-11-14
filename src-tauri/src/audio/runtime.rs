//! Audio runtime management for safe restart and lifecycle control
//!
//! This module provides a high-level AudioRuntime that manages the audio capture
//! stream and KWS worker together, allowing for safe stop/restart cycles without
//! requiring full application restart.

use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};

use crate::audio::kws::KwsConfig;
use crate::audio::kws::KwsWorker;
use crate::audio::vad::VadConfig;
use crate::audio::AudioConfig;
use crate::paths::AppPaths;

/// Signal type for stopping the audio runtime
#[derive(Debug, Clone, Copy)]
pub struct StopSignal;

/// Audio runtime that manages capture stream and KWS worker lifecycle
pub struct AudioRuntime {
    pub kws_worker: Option<KwsWorker>,
    stop_tx: Sender<StopSignal>,
}

impl AudioRuntime {
    /// Start the audio runtime with given configuration
    pub fn start(
        app_handle: tauri::AppHandle,
        paths: AppPaths,
        audio_cfg: AudioConfig,
        kws_cfg: KwsConfig,
        vad_cfg: VadConfig,
    ) -> Result<(Self, Receiver<StopSignal>)> {
        log::info!("Starting audio runtime...");

        // Create stop channel
        let (stop_tx, stop_rx) = bounded::<StopSignal>(1);

        // Start KWS worker if enabled
        let kws_worker = if kws_cfg.enabled {
            // Check mode: "real" or "stub"
            if kws_cfg.mode == "real" {
                #[cfg(feature = "kws_real")]
                {
                    if let Some(ref model_id) = kws_cfg.model_id {
                        log::info!("Starting real KWS with model: {}", model_id);
                        match crate::audio::kws::real::KwsWorker::start(
                            app_handle.clone(),
                            paths.clone(),
                            kws_cfg.clone(),
                            vad_cfg.clone(),
                            audio_cfg.clone(),
                            model_id.clone(),
                        ) {
                            Ok(worker) => {
                                log::info!("✓ Audio runtime started with real KWS");
                                Some(KwsWorker::Real(worker))
                            }
                            Err(e) => {
                                log::warn!("Real KWS failed, falling back to stub: {}", e);
                                // Fall back to stub
                                match KwsWorker::start_stub(
                                    app_handle.clone(),
                                    paths,
                                    kws_cfg,
                                    vad_cfg,
                                    audio_cfg,
                                ) {
                                    Ok(stub_worker) => {
                                        log::info!(
                                            "✓ Audio runtime started with stub KWS (fallback)"
                                        );
                                        Some(stub_worker)
                                    }
                                    Err(stub_err) => {
                                        log::error!("Failed to start stub KWS: {}", stub_err);
                                        None
                                    }
                                }
                            }
                        }
                    } else {
                        log::warn!("Real KWS mode requested but no model_id provided, using stub");
                        match KwsWorker::start_stub(
                            app_handle.clone(),
                            paths,
                            kws_cfg,
                            vad_cfg,
                            audio_cfg,
                        ) {
                            Ok(stub_worker) => {
                                log::info!("✓ Audio runtime started with stub KWS");
                                Some(stub_worker)
                            }
                            Err(e) => {
                                log::error!("Failed to start stub KWS: {}", e);
                                None
                            }
                        }
                    }
                }
                #[cfg(not(feature = "kws_real"))]
                {
                    log::warn!("Real KWS requested but feature not enabled, using stub");
                    match KwsWorker::start_stub(
                        app_handle.clone(),
                        paths,
                        kws_cfg,
                        vad_cfg,
                        audio_cfg,
                    ) {
                        Ok(stub_worker) => {
                            log::info!("✓ Audio runtime started with stub KWS");
                            Some(stub_worker)
                        }
                        Err(e) => {
                            log::error!("Failed to start stub KWS: {}", e);
                            None
                        }
                    }
                }
            } else {
                // Stub mode (default)
                log::info!("Starting stub KWS");
                match KwsWorker::start_stub(app_handle.clone(), paths, kws_cfg, vad_cfg, audio_cfg)
                {
                    Ok(stub_worker) => {
                        log::info!("✓ Audio runtime started with stub KWS");
                        Some(stub_worker)
                    }
                    Err(e) => {
                        log::error!("Failed to start stub KWS: {}", e);
                        None
                    }
                }
            }
        } else {
            log::info!("Audio runtime started without KWS (disabled)");
            None
        };

        let runtime = Self {
            kws_worker,
            stop_tx,
        };

        Ok((runtime, stop_rx))
    }

    /// Stop the audio runtime gracefully
    pub fn stop(self) {
        log::info!("Stopping audio runtime...");

        // Send stop signal (best effort)
        let _ = self.stop_tx.send(StopSignal);

        // Drop worker to trigger cleanup
        drop(self.kws_worker);

        log::info!("✓ Audio runtime stopped");
    }

    /// Check if KWS is active
    pub fn has_kws(&self) -> bool {
        self.kws_worker.is_some()
    }
}

/// Helper to clone stop receiver for passing to worker threads
pub fn clone_stop_receiver(rx: &Receiver<StopSignal>) -> Receiver<StopSignal> {
    rx.clone()
}
