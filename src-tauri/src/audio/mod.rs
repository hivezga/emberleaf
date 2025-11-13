pub mod kws;
pub mod level;
pub mod monitor;
pub mod probe;
pub mod runtime;
pub mod test_tone;
pub mod vad;

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use rubato::{FftFixedIn, Resampler};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Target sample rate for all audio processing (16 kHz)
pub const TARGET_SAMPLE_RATE: u32 = 16000;

/// Target number of channels for processing (always mono)
pub const TARGET_CHANNELS: usize = 1;

/// Stable device identifier for persistence across reboots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeviceId {
    pub host_api: String,
    pub index: u32,
    pub name: String,
}

/// Device information for enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub host: String,
    pub max_channels: u16,
    pub sample_rates: Vec<u32>,
    /// Stable identifier for device persistence
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_id: Option<DeviceId>,
}

/// Audio pipeline debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDebugInfo {
    pub processing_rate: u32,
    pub processing_channels: usize,
    pub frame_ms: u32,
    pub hop_ms: u32,
    pub samples_per_frame: usize,
    pub samples_per_hop: usize,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
}

/// Friendly error message with optional error code
#[derive(Debug, Clone, Serialize)]
pub struct FriendlyError {
    pub message: String,
    pub code: String,
    pub technical: String,
}

/// Map common CPAL/audio errors to user-friendly messages
pub fn friendly_audio_error(error: &anyhow::Error) -> FriendlyError {
    let error_str = format!("{:#}", error);
    let error_lower = error_str.to_lowercase();

    // Check for common patterns
    if error_lower.contains("device busy")
        || error_lower.contains("in use")
        || error_lower.contains("already in use")
    {
        return FriendlyError {
            message: "Another app is using this microphone. Close other audio apps and try again."
                .to_string(),
            code: "device_busy".to_string(),
            technical: error_str,
        };
    }

    if error_lower.contains("no such device")
        || error_lower.contains("not found")
        || error_lower.contains("does not exist")
        || error_lower.contains("disconnected")
    {
        return FriendlyError {
            message: "Microphone was unplugged or is no longer available. Please reconnect it."
                .to_string(),
            code: "device_not_found".to_string(),
            technical: error_str,
        };
    }

    if error_lower.contains("permission")
        || error_lower.contains("access denied")
        || error_lower.contains("denied")
    {
        return FriendlyError {
            message: "Permission denied. Check your system's microphone settings.".to_string(),
            code: "permission_denied".to_string(),
            technical: error_str,
        };
    }

    if error_lower.contains("timeout") {
        return FriendlyError {
            message: "Audio device timed out. Try unplugging and reconnecting it.".to_string(),
            code: "timeout".to_string(),
            technical: error_str,
        };
    }

    if error_lower.contains("no default")
        || error_lower.contains("no input device")
        || error_lower.contains("no microphone")
    {
        return FriendlyError {
            message: "No microphone found. Please connect a microphone.".to_string(),
            code: "no_device".to_string(),
            technical: error_str,
        };
    }

    // Default fallback
    FriendlyError {
        message: "Audio system error. Try restarting the app or reconnecting your microphone."
            .to_string(),
        code: "unknown".to_string(),
        technical: error_str,
    }
}

/// List all available input devices
pub fn list_input_devices() -> Result<Vec<DeviceInfo>> {
    let host = cpal::default_host();
    let default_device_name = host.default_input_device().and_then(|d| d.name().ok());

    let host_id = host.id().name();
    let mut devices = Vec::new();

    for (index, device) in host.input_devices()?.enumerate() {
        if let Ok(name) = device.name() {
            let is_default = Some(name.clone()) == default_device_name;

            // Get device capabilities
            let (max_channels, sample_rates) = device
                .supported_input_configs()
                .ok()
                .and_then(|mut configs| configs.next())
                .map(|config| {
                    let channels = config.channels();
                    let rates = vec![config.min_sample_rate().0, config.max_sample_rate().0];
                    (channels, rates)
                })
                .unwrap_or((1, vec![16000]));

            // Create stable device identifier
            let stable_id = Some(DeviceId {
                host_api: host_id.to_string(),
                index: index as u32,
                name: name.clone(),
            });

            devices.push(DeviceInfo {
                name,
                is_default,
                host: host_id.to_string(),
                max_channels,
                sample_rates,
                stable_id,
            });
        }
    }

    Ok(devices)
}

/// List all available output devices
pub fn list_output_devices() -> Result<Vec<DeviceInfo>> {
    let host = cpal::default_host();
    let default_device_name = host.default_output_device().and_then(|d| d.name().ok());

    let host_id = host.id().name();
    let mut devices = Vec::new();

    for (index, device) in host.output_devices()?.enumerate() {
        if let Ok(name) = device.name() {
            let is_default = Some(name.clone()) == default_device_name;

            // Get device capabilities
            let (max_channels, sample_rates) = device
                .supported_output_configs()
                .ok()
                .and_then(|mut configs| configs.next())
                .map(|config| {
                    let channels = config.channels();
                    let rates = vec![config.min_sample_rate().0, config.max_sample_rate().0];
                    (channels, rates)
                })
                .unwrap_or((2, vec![44100, 48000]));

            // Create stable device identifier
            let stable_id = Some(DeviceId {
                host_api: host_id.to_string(),
                index: index as u32,
                name: name.clone(),
            });

            devices.push(DeviceInfo {
                name,
                is_default,
                host: host_id.to_string(),
                max_channels,
                sample_rates,
                stable_id,
            });
        }
    }

    Ok(devices)
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate_hz: u32,
    pub frame_ms: u32,
    pub hop_ms: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_device_name: Option<String>,
    /// Stable input device identifier (primary key for persistence)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_input_id: Option<DeviceId>,
    /// Stable output device identifier (primary key for persistence)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_output_id: Option<DeviceId>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: TARGET_SAMPLE_RATE,
            frame_ms: 20,
            hop_ms: 10,
            device_name: None,
            output_device_name: None,
            stable_input_id: None,
            stable_output_id: None,
        }
    }
}

impl AudioConfig {
    pub fn samples_per_frame(&self) -> usize {
        (self.sample_rate_hz * self.frame_ms / 1000) as usize
    }

    pub fn samples_per_hop(&self) -> usize {
        (self.sample_rate_hz * self.hop_ms / 1000) as usize
    }
}

/// Resolve preferred input device using stable_id (primary), name (fallback), or default
///
/// Resolution order:
/// 1. Try to match by stable_id (host_api + index + name)
/// 2. Fall back to exact name match
/// 3. Fall back to system default
pub fn resolve_preferred_input_device(
    stable_id: Option<&DeviceId>,
    name: Option<&str>,
) -> Result<Option<cpal::Device>> {
    let host = cpal::default_host();

    // If we have a stable_id, try to resolve by it first
    if let Some(sid) = stable_id {
        log::debug!("Attempting to resolve device by stable_id: {:?}", sid);

        // Try exact match: host_api + index + name
        if let Ok(devices) = host.input_devices() {
            for (idx, device) in devices.enumerate() {
                if let Ok(dev_name) = device.name() {
                    if idx as u32 == sid.index
                        && dev_name == sid.name
                        && host.id().name() == sid.host_api
                    {
                        log::info!("✓ Resolved device by stable_id: {}", dev_name);
                        return Ok(Some(device));
                    }
                }
            }
        }

        log::warn!("Device stable_id not found, falling back to name match");
    }

    // Fall back to name match
    if let Some(device_name) = name {
        log::debug!("Attempting to resolve device by name: {}", device_name);

        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(dev_name) = device.name() {
                    if dev_name == device_name {
                        log::info!("✓ Resolved device by name: {}", dev_name);
                        return Ok(Some(device));
                    }
                }
            }
        }

        log::warn!(
            "Device name '{}' not found, falling back to default",
            device_name
        );
    }

    // Fall back to system default
    Ok(None) // None means "use default"
}

/// Resolve preferred output device using stable_id (primary), name (fallback), or default
pub fn resolve_preferred_output_device(
    stable_id: Option<&DeviceId>,
    name: Option<&str>,
) -> Result<Option<cpal::Device>> {
    let host = cpal::default_host();

    // If we have a stable_id, try to resolve by it first
    if let Some(sid) = stable_id {
        log::debug!(
            "Attempting to resolve output device by stable_id: {:?}",
            sid
        );

        // Try exact match: host_api + index + name
        if let Ok(devices) = host.output_devices() {
            for (idx, device) in devices.enumerate() {
                if let Ok(dev_name) = device.name() {
                    if idx as u32 == sid.index
                        && dev_name == sid.name
                        && host.id().name() == sid.host_api
                    {
                        log::info!("✓ Resolved output device by stable_id: {}", dev_name);
                        return Ok(Some(device));
                    }
                }
            }
        }

        log::warn!("Output device stable_id not found, falling back to name match");
    }

    // Fall back to name match
    if let Some(device_name) = name {
        log::debug!(
            "Attempting to resolve output device by name: {}",
            device_name
        );

        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(dev_name) = device.name() {
                    if dev_name == device_name {
                        log::info!("✓ Resolved output device by name: {}", dev_name);
                        return Ok(Some(device));
                    }
                }
            }
        }

        log::warn!(
            "Output device name '{}' not found, falling back to default",
            device_name
        );
    }

    // Fall back to system default
    Ok(None) // None means "use default"
}

/// Check if an input device with the given stable_id or name still exists
pub fn check_input_device_exists(stable_id: Option<&DeviceId>, name: Option<&str>) -> bool {
    resolve_preferred_input_device(stable_id, name)
        .ok()
        .flatten()
        .is_some()
}

/// Check if an output device with the given stable_id or name still exists
pub fn check_output_device_exists(stable_id: Option<&DeviceId>, name: Option<&str>) -> bool {
    resolve_preferred_output_device(stable_id, name)
        .ok()
        .flatten()
        .is_some()
}

/// Trait for audio sources that can provide frames
///
/// Note: This trait does not require Send since audio sources are confined to a single worker thread
pub trait AudioSource {
    /// Get the next frame of audio samples
    /// Returns None if no frame is available yet
    fn next_frame(&mut self) -> Option<Vec<i16>>;

    /// Get the sample rate of this source
    fn sample_rate(&self) -> u32;

    /// Get the frame size in samples
    fn frame_size(&self) -> usize;
}

/// Audio capture system using CPAL
pub struct AudioCapture {
    _stream: Stream,
    receiver: mpsc::UnboundedReceiver<Vec<i16>>,
    config: AudioConfig,
    device_rate: u32,
    device_channels: usize,
    needs_resampling: bool,
    resampler: Option<Arc<Mutex<FftFixedIn<f32>>>>,
    buffer: Vec<i16>,
    resample_input_buffer: Vec<f32>,
    resample_output_buffer: Vec<f32>,
}

impl AudioCapture {
    /// Create a new audio capture system with the default device
    pub fn new(config: AudioConfig) -> Result<Self> {
        // Resolve device using stable_id (primary), name (fallback), or default
        let resolved_device = resolve_preferred_input_device(
            config.stable_input_id.as_ref(),
            config.device_name.as_deref(),
        )?;

        let device = if let Some(dev) = resolved_device {
            dev
        } else {
            // Fall back to system default
            let host = cpal::default_host();
            host.default_input_device()
                .context("No input device available")?
        };

        Self::new_with_device(config, device)
    }

    /// Create a new audio capture system with a specific device name
    pub fn new_with_name(config: AudioConfig, device_name: &str) -> Result<Self> {
        let host = cpal::default_host();

        // Find the device by name
        let device = host
            .input_devices()?
            .find(|d| d.name().ok().as_deref() == Some(device_name))
            .with_context(|| format!("Device '{}' not found", device_name))?;

        Self::new_with_device(config, device)
    }

    /// Create audio capture with a specific device
    fn new_with_device(config: AudioConfig, device: cpal::Device) -> Result<Self> {
        log::info!("Using audio device: {}", device.name()?);

        // Get supported config
        let supported_config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels();

        log::info!(
            "Device config: {} Hz, {} channels, format: {:?}",
            sample_rate,
            channels,
            supported_config.sample_format()
        );

        // Determine if resampling is needed
        let needs_resampling = sample_rate != config.sample_rate_hz;

        // Create resampler if needed (always for mono processing)
        let resampler = if needs_resampling {
            log::info!(
                "Audio: device={}Hz, {}ch -> processing={}Hz, {}ch (downmix={})",
                sample_rate,
                channels,
                config.sample_rate_hz,
                TARGET_CHANNELS,
                channels != TARGET_CHANNELS as u16
            );

            let params = rubato::FftFixedIn::<f32>::new(
                sample_rate as usize,
                config.sample_rate_hz as usize,
                config.samples_per_frame(),
                2,               // Max number of channels (rubato parameter)
                TARGET_CHANNELS, // Always mono for processing
            )?;

            Some(Arc::new(Mutex::new(params)))
        } else {
            log::info!(
                "Audio: device={}Hz, {}ch (no resampling needed, downmix={})",
                sample_rate,
                channels,
                channels != TARGET_CHANNELS as u16
            );
            None
        };

        // Create channel for audio data
        let (sender, receiver) = mpsc::unbounded_channel();

        // Build stream
        let stream_config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let sender_clone = sender.clone();
        let channels_count = channels as usize;

        let stream = match supported_config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| {
                    Self::handle_input_f32(data, channels_count, &sender_clone);
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| {
                    Self::handle_input_i16(data, channels_count, &sender_clone);
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )?,
            SampleFormat::U16 => device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &_| {
                    Self::handle_input_u16(data, channels_count, &sender_clone);
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            )?,
            _ => anyhow::bail!("Unsupported sample format"),
        };

        stream.play()?;

        log::info!("Audio capture started successfully");

        Ok(Self {
            _stream: stream,
            receiver,
            config,
            device_rate: sample_rate,
            device_channels: channels as usize,
            needs_resampling,
            resampler,
            buffer: Vec::new(),
            resample_input_buffer: Vec::new(),
            resample_output_buffer: Vec::new(),
        })
    }

    /// Get the device sample rate (actual mic rate)
    pub fn device_rate(&self) -> u32 {
        self.device_rate
    }

    fn handle_input_f32(data: &[f32], channels: usize, sender: &mpsc::UnboundedSender<Vec<i16>>) {
        // Convert to mono i16
        let mono: Vec<i16> = data
            .chunks(channels)
            .map(|chunk| {
                let avg = chunk.iter().sum::<f32>() / channels as f32;
                (avg * i16::MAX as f32) as i16
            })
            .collect();

        let _ = sender.send(mono);
    }

    fn handle_input_i16(data: &[i16], channels: usize, sender: &mpsc::UnboundedSender<Vec<i16>>) {
        // Convert to mono
        let mono: Vec<i16> = data
            .chunks(channels)
            .map(|chunk| {
                let avg: i32 = chunk.iter().map(|&s| s as i32).sum();
                (avg / channels as i32) as i16
            })
            .collect();

        let _ = sender.send(mono);
    }

    fn handle_input_u16(data: &[u16], channels: usize, sender: &mpsc::UnboundedSender<Vec<i16>>) {
        // Convert to mono i16
        let mono: Vec<i16> = data
            .chunks(channels)
            .map(|chunk| {
                let avg: i32 = chunk.iter().map(|&s| s as i32).sum();
                let avg_u16 = (avg / channels as i32) as u16;
                (avg_u16 as i32 - 32768) as i16 // Convert u16 to i16
            })
            .collect();

        let _ = sender.send(mono);
    }
}

impl AudioSource for AudioCapture {
    fn next_frame(&mut self) -> Option<Vec<i16>> {
        let frame_size = self.config.samples_per_frame();

        // Accumulate data from receiver
        while let Ok(data) = self.receiver.try_recv() {
            if self.needs_resampling {
                if let Some(ref resampler) = self.resampler {
                    // Convert i16 to f32 for resampling
                    let f32_samples: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

                    self.resample_input_buffer.extend(f32_samples);

                    // Process resampling when we have enough input
                    let mut resampler = resampler.lock().unwrap();
                    let input_frames_needed = resampler.input_frames_next();

                    while self.resample_input_buffer.len() >= input_frames_needed {
                        // Prepare input for resampler (mono channel)
                        let input_chunk: Vec<f32> = self
                            .resample_input_buffer
                            .drain(..input_frames_needed)
                            .collect();

                        // Resample (single channel)
                        let input_vec = vec![input_chunk];
                        match resampler.process(&input_vec, None) {
                            Ok(output_vec) => {
                                // Convert resampled f32 back to i16
                                let resampled_i16: Vec<i16> = output_vec[0]
                                    .iter()
                                    .map(|&s| {
                                        (s * i16::MAX as f32)
                                            .clamp(i16::MIN as f32, i16::MAX as f32)
                                            as i16
                                    })
                                    .collect();

                                self.buffer.extend(resampled_i16);
                            }
                            Err(e) => {
                                log::error!("Resampling error: {}", e);
                            }
                        }
                    }
                } else {
                    // Fallback if resampler not available
                    self.buffer.extend_from_slice(&data);
                }
            } else {
                // No resampling needed, pass through
                self.buffer.extend_from_slice(&data);
            }
        }

        // Return frame if we have enough data
        if self.buffer.len() >= frame_size {
            let frame = self.buffer.drain(..frame_size).collect();
            Some(frame)
        } else {
            None
        }
    }

    fn sample_rate(&self) -> u32 {
        // Always return target sample rate (16 kHz output)
        TARGET_SAMPLE_RATE
    }

    fn frame_size(&self) -> usize {
        self.config.samples_per_frame()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate_hz, 16000);
        assert_eq!(config.samples_per_frame(), 320); // 20ms @ 16kHz
        assert_eq!(config.samples_per_hop(), 160); // 10ms @ 16kHz
    }
}
