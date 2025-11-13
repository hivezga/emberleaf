//! Microphone monitoring for audio debugging
//!
//! Provides real-time input monitoring by routing microphone audio to speakers
//! at a safe, low gain level. Includes automatic feedback prevention when
//! input and output devices are the same.

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

/// Signal to stop the monitor
#[derive(Debug, Clone, Copy)]
pub struct StopMonitor;

/// Microphone monitor handle that manages a worker thread
pub struct MicMonitor {
    stop_tx: Sender<StopMonitor>,
    _thread_handle: Option<thread::JoinHandle<()>>,
}

impl MicMonitor {
    /// Start monitoring from input device to output device
    ///
    /// # Arguments
    /// * `input_device_name` - Optional input device name
    /// * `output_device_name` - Optional output device name
    /// * `gain` - Monitor gain level (0.0-1.0, clamped to 0.0-0.5 for safety)
    ///
    /// # Safety
    /// If input and output device names match, monitoring is disabled to prevent feedback
    pub fn start(
        input_device_name: Option<String>,
        output_device_name: Option<String>,
        gain: f32,
    ) -> Result<Self> {
        // Safety: prevent feedback loop
        if input_device_name == output_device_name && input_device_name.is_some() {
            anyhow::bail!(
                "Cannot monitor when input and output are the same device (feedback prevention)"
            );
        }

        // Clamp gain to safe range (max 0.5 to prevent distortion/feedback)
        let gain = gain.clamp(0.0, 0.5);

        log::info!("Starting mic monitor with gain={:.2}", gain);

        // Create stop channel
        let (stop_tx, stop_rx) = bounded::<StopMonitor>(1);

        // Spawn monitoring thread
        let thread_handle = thread::spawn(move || {
            if let Err(e) = run_monitor_worker(input_device_name, output_device_name, gain, stop_rx)
            {
                log::error!("Mic monitor worker error: {}", e);
            }
        });

        log::info!("✓ Mic monitor thread started");

        Ok(Self {
            stop_tx,
            _thread_handle: Some(thread_handle),
        })
    }

    /// Stop the monitor
    pub fn stop(self) {
        log::info!("Stopping mic monitor...");
        let _ = self.stop_tx.send(StopMonitor);
        // Thread will be dropped
        log::info!("✓ Mic monitor stopped");
    }
}

/// Monitor worker function that runs in a dedicated thread
fn run_monitor_worker(
    input_device_name: Option<String>,
    output_device_name: Option<String>,
    gain: f32,
    stop_rx: Receiver<StopMonitor>,
) -> Result<()> {
    let host = cpal::default_host();

    // Get input device
    let input_device = if let Some(ref name) = input_device_name {
        host.input_devices()?
            .find(|d| d.name().ok().as_deref() == Some(name.as_str()))
            .with_context(|| format!("Input device not found: {}", name))?
    } else {
        host.default_input_device()
            .context("No default input device")?
    };

    // Get output device
    let output_device = if let Some(ref name) = output_device_name {
        host.output_devices()?
            .find(|d| d.name().ok().as_deref() == Some(name.as_str()))
            .with_context(|| format!("Output device not found: {}", name))?
    } else {
        host.default_output_device()
            .context("No default output device")?
    };

    log::info!(
        "Monitor: {} -> {}",
        input_device.name()?,
        output_device.name()?
    );

    // Shared buffer for audio data (ring buffer approach)
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let buffer_in = Arc::clone(&buffer);
    let buffer_out = Arc::clone(&buffer);

    // Build input stream
    let input_config = input_device.default_input_config()?;
    let input_stream = match input_config.sample_format() {
        SampleFormat::F32 => build_input_stream_f32(&input_device, input_config, buffer_in)?,
        SampleFormat::I16 => build_input_stream_i16(&input_device, input_config, buffer_in)?,
        SampleFormat::U16 => build_input_stream_u16(&input_device, input_config, buffer_in)?,
        _ => anyhow::bail!("Unsupported input sample format"),
    };

    // Build output stream
    let output_config = output_device.default_output_config()?;
    let output_stream = match output_config.sample_format() {
        SampleFormat::F32 => build_output_stream_f32(
            &output_device,
            output_config,
            buffer_out,
            gain,
            stop_rx.clone(),
        )?,
        SampleFormat::I16 => build_output_stream_i16(
            &output_device,
            output_config,
            buffer_out,
            gain,
            stop_rx.clone(),
        )?,
        SampleFormat::U16 => build_output_stream_u16(
            &output_device,
            output_config,
            buffer_out,
            gain,
            stop_rx.clone(),
        )?,
        _ => anyhow::bail!("Unsupported output sample format"),
    };

    // Start streams
    input_stream.play()?;
    output_stream.play()?;

    log::info!("✓ Mic monitor streams active");

    // Keep thread alive until stop signal
    let _ = stop_rx.recv();

    log::info!("Mic monitor worker stopping...");
    drop(input_stream);
    drop(output_stream);

    Ok(())
}

// Input stream builders

fn build_input_stream_f32(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<Stream> {
    let channels = config.channels() as usize;
    let stream = device.build_input_stream(
        &config.config(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Downmix to mono and store in buffer
            let mut buf = buffer.lock().unwrap();
            for frame in data.chunks(channels) {
                let mono = frame.iter().sum::<f32>() / channels as f32;
                buf.push(mono);
            }
            // Limit buffer size to prevent unbounded growth
            if buf.len() > 48000 {
                buf.drain(0..24000);
            }
        },
        |err| log::error!("Monitor input error: {}", err),
        None,
    )?;
    Ok(stream)
}

fn build_input_stream_i16(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<Stream> {
    let channels = config.channels() as usize;
    let stream = device.build_input_stream(
        &config.config(),
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            let mut buf = buffer.lock().unwrap();
            for frame in data.chunks(channels) {
                let mono = frame
                    .iter()
                    .map(|&s| s as f32 / i16::MAX as f32)
                    .sum::<f32>()
                    / channels as f32;
                buf.push(mono);
            }
            if buf.len() > 48000 {
                buf.drain(0..24000);
            }
        },
        |err| log::error!("Monitor input error: {}", err),
        None,
    )?;
    Ok(stream)
}

fn build_input_stream_u16(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<Stream> {
    let channels = config.channels() as usize;
    let stream = device.build_input_stream(
        &config.config(),
        move |data: &[u16], _: &cpal::InputCallbackInfo| {
            let mut buf = buffer.lock().unwrap();
            for frame in data.chunks(channels) {
                let mono = frame
                    .iter()
                    .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                    .sum::<f32>()
                    / channels as f32;
                buf.push(mono);
            }
            if buf.len() > 48000 {
                buf.drain(0..24000);
            }
        },
        |err| log::error!("Monitor input error: {}", err),
        None,
    )?;
    Ok(stream)
}

// Output stream builders

fn build_output_stream_f32(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    gain: f32,
    stop_rx: Receiver<StopMonitor>,
) -> Result<Stream> {
    let channels = config.channels() as usize;
    let stream = device.build_output_stream(
        &config.config(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                data.fill(0.0);
                return;
            }

            let mut buf = buffer.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = if !buf.is_empty() {
                    buf.remove(0) * gain
                } else {
                    0.0
                };
                for s in frame.iter_mut() {
                    *s = sample;
                }
            }
        },
        |err| log::error!("Monitor output error: {}", err),
        None,
    )?;
    Ok(stream)
}

fn build_output_stream_i16(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    gain: f32,
    stop_rx: Receiver<StopMonitor>,
) -> Result<Stream> {
    let channels = config.channels() as usize;
    let stream = device.build_output_stream(
        &config.config(),
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            if stop_rx.try_recv().is_ok() {
                data.fill(0);
                return;
            }

            let mut buf = buffer.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = if !buf.is_empty() {
                    (buf.remove(0) * gain * i16::MAX as f32) as i16
                } else {
                    0
                };
                for s in frame.iter_mut() {
                    *s = sample;
                }
            }
        },
        |err| log::error!("Monitor output error: {}", err),
        None,
    )?;
    Ok(stream)
}

fn build_output_stream_u16(
    device: &cpal::Device,
    config: cpal::SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    gain: f32,
    stop_rx: Receiver<StopMonitor>,
) -> Result<Stream> {
    let channels = config.channels() as usize;
    let stream = device.build_output_stream(
        &config.config(),
        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
            if stop_rx.try_recv().is_ok() {
                data.fill(32768);
                return;
            }

            let mut buf = buffer.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = if !buf.is_empty() {
                    let f = buf.remove(0) * gain;
                    ((f + 1.0) * 0.5 * u16::MAX as f32) as u16
                } else {
                    32768
                };
                for s in frame.iter_mut() {
                    *s = sample;
                }
            }
        },
        |err| log::error!("Monitor output error: {}", err),
        None,
    )?;
    Ok(stream)
}
