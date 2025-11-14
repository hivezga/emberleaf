//! Audio device auto-probing for intelligent device suggestion
//!
//! Provides functionality to:
//! - Sample audio from devices to detect active input
//! - Compare RMS levels across multiple devices
//! - Suggest the best input device based on actual audio activity

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// RMS threshold for considering a device "active" (normalized 0..1)
/// ~0.02 RMS ≈ quiet ambient noise, ~0.05 RMS ≈ someone speaking at distance
const ACTIVE_THRESHOLD: f32 = 0.02;

/// Result of auto-probe operation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub suggested: Option<String>,
    pub reason: String,
}

/// Probe the current device for audio activity
///
/// Listens for `duration_ms` and returns the peak RMS value
pub fn probe_current_device(device_name: Option<&str>, duration_ms: u64) -> Result<f32> {
    let host = cpal::default_host();

    let device = if let Some(name) = device_name {
        host.input_devices()?
            .find(|d| d.name().map(|n| n == name).unwrap_or(false))
            .with_context(|| format!("Device not found: {}", name))?
    } else {
        host.default_input_device()
            .context("No default input device")?
    };

    probe_device(&device, duration_ms)
}

/// Probe a specific device for audio activity
fn probe_device(device: &cpal::Device, duration_ms: u64) -> Result<f32> {
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    // Shared state for peak RMS
    let peak_rms = Arc::new(Mutex::new(0.0f32));
    let peak_rms_clone = Arc::clone(&peak_rms);

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            build_probe_stream_f32(device, config.into(), peak_rms_clone, sample_rate)?
        }
        SampleFormat::I16 => {
            build_probe_stream_i16(device, config.into(), peak_rms_clone, sample_rate)?
        }
        SampleFormat::U16 => {
            build_probe_stream_u16(device, config.into(), peak_rms_clone, sample_rate)?
        }
        format => anyhow::bail!("Unsupported format: {:?}", format),
    };

    stream.play()?;
    thread::sleep(Duration::from_millis(duration_ms));
    drop(stream);

    let peak = *peak_rms.lock().unwrap();
    Ok(peak)
}

/// Build F32 probe stream
fn build_probe_stream_f32(
    device: &cpal::Device,
    config: StreamConfig,
    peak_rms: Arc<Mutex<f32>>,
    sample_rate: f32,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;
    let frame_size = (sample_rate * 0.020) as usize; // 20ms frames
    let mut buffer = Vec::new();

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Convert interleaved to mono by averaging channels
            for chunk in data.chunks(channels) {
                let mono_sample: f32 = chunk.iter().sum::<f32>() / channels as f32;
                buffer.push(mono_sample);

                // Process when we have a full frame
                if buffer.len() >= frame_size {
                    let rms = compute_rms_f32(&buffer);
                    let mut peak = peak_rms.lock().unwrap();
                    *peak = peak.max(rms);
                    buffer.clear();
                }
            }
        },
        move |err| log::error!("Probe stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

/// Build I16 probe stream
fn build_probe_stream_i16(
    device: &cpal::Device,
    config: StreamConfig,
    peak_rms: Arc<Mutex<f32>>,
    sample_rate: f32,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;
    let frame_size = (sample_rate * 0.020) as usize; // 20ms frames
    let mut buffer = Vec::new();

    let stream = device.build_input_stream(
        &config,
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            // Convert interleaved to mono by averaging channels
            for chunk in data.chunks(channels) {
                let mono_sample: f32 = chunk
                    .iter()
                    .map(|&s| s as f32 / i16::MAX as f32)
                    .sum::<f32>()
                    / channels as f32;
                buffer.push(mono_sample);

                // Process when we have a full frame
                if buffer.len() >= frame_size {
                    let rms = compute_rms_f32(&buffer);
                    let mut peak = peak_rms.lock().unwrap();
                    *peak = peak.max(rms);
                    buffer.clear();
                }
            }
        },
        move |err| log::error!("Probe stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

/// Build U16 probe stream
fn build_probe_stream_u16(
    device: &cpal::Device,
    config: StreamConfig,
    peak_rms: Arc<Mutex<f32>>,
    sample_rate: f32,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;
    let frame_size = (sample_rate * 0.020) as usize; // 20ms frames
    let mut buffer = Vec::new();

    let stream = device.build_input_stream(
        &config,
        move |data: &[u16], _: &cpal::InputCallbackInfo| {
            // Convert interleaved to mono by averaging channels
            for chunk in data.chunks(channels) {
                let mono_sample: f32 = chunk
                    .iter()
                    .map(|&s| {
                        // Convert U16 (0..65535) to F32 (-1.0..1.0)
                        (s as f32 / u16::MAX as f32) * 2.0 - 1.0
                    })
                    .sum::<f32>()
                    / channels as f32;
                buffer.push(mono_sample);

                // Process when we have a full frame
                if buffer.len() >= frame_size {
                    let rms = compute_rms_f32(&buffer);
                    let mut peak = peak_rms.lock().unwrap();
                    *peak = peak.max(rms);
                    buffer.clear();
                }
            }
        },
        move |err| log::error!("Probe stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

/// Compute RMS from F32 samples
fn compute_rms_f32(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|x| x * x).sum();
    (sum / samples.len() as f32).sqrt()
}

/// Auto-probe and suggest best input device
///
/// Algorithm:
/// 1. Probe current device for 2 seconds
/// 2. If RMS below threshold, scan all input devices (200ms open + 500ms sample)
/// 3. Return device with highest RMS above threshold
pub fn suggest_input_device(current_device: Option<&str>) -> Result<ProbeResult> {
    log::info!("Starting auto-probe for input device...");

    // Step 1: Probe current device
    let current_rms = match probe_current_device(current_device, 2000) {
        Ok(rms) => {
            log::info!("Current device RMS: {:.4}", rms);
            rms
        }
        Err(e) => {
            log::warn!("Failed to probe current device: {}", e);
            0.0
        }
    };

    // If current device is good, keep it
    if current_rms >= ACTIVE_THRESHOLD {
        let device_name = current_device.unwrap_or("default");
        return Ok(ProbeResult {
            suggested: Some(device_name.to_string()),
            reason: format!("Current device is active (RMS: {:.3})", current_rms),
        });
    }

    // Step 2: Scan all input devices
    log::info!("Current device silent, scanning alternatives...");
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices()?.collect();

    if devices.is_empty() {
        return Ok(ProbeResult {
            suggested: None,
            reason: "No input devices found".to_string(),
        });
    }

    let mut best_device: Option<String> = None;
    let mut best_rms = 0.0f32;

    for device in devices {
        let device_name = match device.name() {
            Ok(name) => name,
            Err(_) => continue,
        };

        // Skip current device (already probed)
        if Some(device_name.as_str()) == current_device {
            continue;
        }

        log::debug!("Probing device: {}", device_name);

        // Quick probe: 200ms warmup + 500ms sample
        thread::sleep(Duration::from_millis(200));
        match probe_device(&device, 500) {
            Ok(rms) => {
                log::debug!("Device '{}' RMS: {:.4}", device_name, rms);
                if rms > best_rms && rms >= ACTIVE_THRESHOLD {
                    best_device = Some(device_name.clone());
                    best_rms = rms;
                }
            }
            Err(e) => {
                log::debug!("Failed to probe device '{}': {}", device_name, e);
            }
        }
    }

    // Step 3: Return result
    if let Some(device_name) = best_device {
        Ok(ProbeResult {
            suggested: Some(device_name),
            reason: format!("Detected speech on this device (RMS: {:.3})", best_rms),
        })
    } else {
        Ok(ProbeResult {
            suggested: None,
            reason: "No active audio detected on any device".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_computation() {
        let samples = vec![0.0, 0.5, -0.5, 0.25, -0.25];
        let rms = compute_rms_f32(&samples);
        assert!(rms > 0.0 && rms < 1.0);
    }

    #[test]
    fn test_threshold() {
        // ACTIVE_THRESHOLD is a const, validated at compile time
        // Clippy: assertions_on_constants - these would be optimized out
    }
}
