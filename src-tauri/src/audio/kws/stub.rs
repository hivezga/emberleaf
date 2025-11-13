//! Stub KWS implementation for development without Sherpa-ONNX
//!
//! Provides a simple energy-based wake-word simulator for testing the audio
//! pipeline and UI without requiring the full Sherpa-ONNX library.

use super::super::vad::{VadConfig, VoiceActivityDetector};
use super::super::{AudioCapture, AudioConfig, AudioSource};
use super::{KwsConfig, WakeWordEvent};
use crate::audio::level;
use anyhow::Result;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Stub KWS worker that runs in a dedicated thread
pub struct KwsWorker {
    _thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl KwsWorker {
    /// Start the KWS worker thread with stub implementation
    pub fn start(
        app_handle: AppHandle,
        _paths: crate::paths::AppPaths,
        config: KwsConfig,
        vad_config: VadConfig,
        audio_config: AudioConfig,
    ) -> Result<Self> {
        log::info!("Starting stub KWS worker (energy-based detection)");

        // Spawn worker thread (NOT tokio::spawn - std::thread to avoid Send issues)
        let handle = std::thread::spawn(move || {
            if let Err(e) = run_stub_kws_worker(app_handle, config, vad_config, audio_config) {
                log::error!("KWS worker thread error: {}", e);
            }
        });

        log::info!("Stub KWS worker started");
        Ok(Self {
            _thread_handle: Some(handle),
        })
    }
}

/// Stub KWS worker loop
fn run_stub_kws_worker(
    app_handle: AppHandle,
    config: KwsConfig,
    vad_config: VadConfig,
    audio_config: AudioConfig,
) -> Result<()> {
    log::info!("Stub KWS worker: simulating wake-word detection");
    log::info!("  Keyword: '{}'", config.keyword);
    log::info!("  Threshold: {:.2}", config.score_threshold);
    log::info!("  Refractory: {}ms", config.refractory_ms);

    // Initialize audio capture (happens in this thread, so no Send issues)
    let mut audio_source = AudioCapture::new(audio_config)?;
    log::info!(
        "  Audio capture initialized @{}Hz",
        audio_source.sample_rate()
    );

    let mut vad = VoiceActivityDetector::new(vad_config, audio_source.sample_rate())?;
    let mut last_detection: Option<Instant> = None;
    let mut frame_count = 0u64;

    // RMS emission throttle (20 Hz = 50ms)
    let mut last_rms_emit = Instant::now();

    // Energy-based detection parameters (stub heuristic)
    let energy_threshold = 3000.0; // Arbitrary threshold for demo
    let min_energy_frames = 3; // Require sustained energy
    let mut high_energy_count = 0;

    loop {
        // Get next audio frame
        if let Some(samples) = audio_source.next_frame() {
            frame_count += 1;

            // Emit RMS for UI meter (throttled to 20 Hz)
            let now = Instant::now();
            if now.duration_since(last_rms_emit) >= Duration::from_millis(50) {
                level::emit_rms_i16(&app_handle, &samples);
                last_rms_emit = now;
            }

            // VAD gating
            if !vad.process_frame(&samples) {
                high_energy_count = 0;
                continue;
            }

            // Check refractory period
            if let Some(last) = last_detection {
                if last.elapsed() < Duration::from_millis(config.refractory_ms) {
                    continue;
                }
            }

            // Compute energy (simple RMS)
            let energy = compute_rms_energy(&samples);

            // Count consecutive high-energy frames
            if energy > energy_threshold {
                high_energy_count += 1;
            } else {
                high_energy_count = 0;
            }

            // Trigger detection on sustained high energy
            if high_energy_count >= min_energy_frames {
                let score = (energy / energy_threshold).min(1.0).max(0.0);

                if score >= config.score_threshold {
                    log::info!(
                        "[STUB] Wake word detected! score={:.3} energy={:.1}",
                        score,
                        energy
                    );

                    let event = WakeWordEvent {
                        keyword: config.keyword.clone(),
                        score,
                    };

                    // Emit Tauri event
                    if let Err(e) = app_handle.emit("wakeword::detected", &event) {
                        log::error!("Failed to emit wake-word event: {}", e);
                    }

                    last_detection = Some(Instant::now());
                    high_energy_count = 0;
                }
            }

            if frame_count % 100 == 0 {
                log::trace!("[STUB] Processed {} frames", frame_count);
            }
        } else {
            // No frame available, yield briefly
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

/// Compute RMS energy for stub detection heuristic
fn compute_rms_energy(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
    let rms = (sum_squares / samples.len() as f64).sqrt();
    rms as f32
}
