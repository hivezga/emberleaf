//! Test tone generation for audio output verification
//!
//! Provides simple sine wave tone generation using CPAL for verifying
//! output device configuration and audio pipeline functionality.

use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use std::f32::consts::PI;
use std::time::Duration;

/// Play a test tone on the specified output device with automatic format fallback
///
/// # Arguments
/// * `device_name` - Optional device name. If None, uses default output device
/// * `freq_hz` - Frequency in Hz (clamped to 40-20000 Hz)
/// * `duration_ms` - Duration in milliseconds (clamped to 50-10000 ms)
/// * `volume` - Volume level 0.0-1.0 (clamped to 0.0-1.0)
///
/// # Format Fallback
/// Tries formats in order: F32 → I16 → U16, logging each attempt
pub fn play_tone(
    device_name: Option<String>,
    freq_hz: f32,
    duration_ms: u32,
    volume: f32,
) -> Result<()> {
    // Clamp parameters to safe ranges
    let freq_hz = freq_hz.clamp(40.0, 20000.0);
    let duration_ms = duration_ms.clamp(50, 10_000);
    let volume = volume.clamp(0.0, 1.0);

    log::info!(
        "Playing test tone: {}Hz, {}ms, volume={:.2}",
        freq_hz,
        duration_ms,
        volume
    );

    let host = cpal::default_host();

    // Select output device
    let device = if let Some(ref name) = device_name {
        host.output_devices()?
            .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
            .with_context(|| format!("Output device not found: {}", name))?
    } else {
        host.default_output_device()
            .context("No default output device available")?
    };

    let device_name_str = device.name()?;
    log::info!("Using output device: {}", device_name_str);

    // Get default config and sample rate
    let default_config = device.default_output_config()?;
    let sample_rate = default_config.sample_rate().0 as f32;
    let channels = default_config.channels();

    log::debug!(
        "Device default: {}Hz, {:?}, {} channels",
        sample_rate,
        default_config.sample_format(),
        channels
    );

    // Try formats in order: F32, I16, U16
    let formats_to_try = [
        (SampleFormat::F32, "F32"),
        (SampleFormat::I16, "I16"),
        (SampleFormat::U16, "U16"),
    ];

    let mut last_error = None;

    for (format, format_name) in &formats_to_try {
        log::debug!("Attempting format: {}", format_name);

        match try_build_stream(&device, *format, channels, sample_rate, freq_hz, volume) {
            Ok(stream) => {
                log::info!("✓ Test tone using {} format", format_name);
                stream.play()?;
                std::thread::sleep(Duration::from_millis(duration_ms as u64));
                drop(stream);
                log::info!("✓ Test tone complete");
                return Ok(());
            }
            Err(e) => {
                log::warn!("Format {} failed: {}", format_name, e);
                last_error = Some((format_name, e));
            }
        }
    }

    // All formats failed
    let (failed_format, err) = last_error.unwrap();
    bail!(
        "Test tone failed on device '{}': all formats (F32, I16, U16) failed. Last error ({}): {:?}",
        device_name_str,
        failed_format,
        err
    )
}

/// Try to build a stream with a specific format
fn try_build_stream(
    device: &cpal::Device,
    format: SampleFormat,
    channels: u16,
    sample_rate: f32,
    freq_hz: f32,
    volume: f32,
) -> Result<cpal::Stream> {
    let config = cpal::StreamConfig {
        channels,
        sample_rate: cpal::SampleRate(sample_rate as u32),
        buffer_size: cpal::BufferSize::Default,
    };

    match format {
        SampleFormat::F32 => build_f32_stream_direct(device, config, freq_hz, sample_rate, volume),
        SampleFormat::I16 => build_i16_stream_direct(device, config, freq_hz, sample_rate, volume),
        SampleFormat::U16 => build_u16_stream_direct(device, config, freq_hz, sample_rate, volume),
        _ => bail!("Unsupported format: {:?}", format),
    }
}

/// Build F32 output stream (direct with StreamConfig)
fn build_f32_stream_direct(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    freq_hz: f32,
    sample_rate: f32,
    volume: f32,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;
    let mut sample_clock = 0f32;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = (2.0 * PI * freq_hz * sample_clock / sample_rate).sin() * volume;
                sample_clock = (sample_clock + 1.0) % sample_rate;

                // Write same value to all channels
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        move |err| log::error!("Test tone stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

/// Build I16 output stream (direct with StreamConfig)
fn build_i16_stream_direct(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    freq_hz: f32,
    sample_rate: f32,
    volume: f32,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;
    let mut sample_clock = 0f32;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = (2.0 * PI * freq_hz * sample_clock / sample_rate).sin() * volume;
                sample_clock = (sample_clock + 1.0) % sample_rate;

                let sample_i16 = (value * i16::MAX as f32) as i16;

                // Write same value to all channels
                for sample in frame.iter_mut() {
                    *sample = sample_i16;
                }
            }
        },
        move |err| log::error!("Test tone stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

/// Build U16 output stream (direct with StreamConfig)
fn build_u16_stream_direct(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    freq_hz: f32,
    sample_rate: f32,
    volume: f32,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;
    let mut sample_clock = 0f32;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = (2.0 * PI * freq_hz * sample_clock / sample_rate).sin() * volume;
                sample_clock = (sample_clock + 1.0) % sample_rate;

                // Convert -1.0..1.0 to 0..65535
                let sample_u16 = ((value + 1.0) * 0.5 * u16::MAX as f32) as u16;

                // Write same value to all channels
                for sample in frame.iter_mut() {
                    *sample = sample_u16;
                }
            }
        },
        move |err| log::error!("Test tone stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parameter_clamping() {
        // Test that extreme values are clamped safely
        // (We can't actually play audio in tests, but we can verify the logic compiles)
        let freq = 30000.0_f32.clamp(40.0, 20000.0);
        assert_eq!(freq, 20000.0);

        let duration = 50000_u32.clamp(50, 10_000);
        assert_eq!(duration, 10_000);

        let volume = 5.0_f32.clamp(0.0, 1.0);
        assert_eq!(volume, 1.0);
    }
}
