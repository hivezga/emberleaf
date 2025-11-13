//! Audio level metering and RMS emission for UI visualization

use tauri::{AppHandle, Emitter};

/// Emit a normalized 0..1 RMS value to the frontend (`audio:rms`).
///
/// `frame` is mono f32 at any rate; call this ~20–50ms for smooth UI updates.
///
/// The RMS is normalized such that ~0.20 RMS ≈ strong speech,
/// values are soft-clipped to [0.0, 1.0] range.
pub fn emit_rms(app: &AppHandle, frame: &[f32]) {
    if frame.is_empty() {
        let _ = app.emit("audio:rms", 0.0f32);
        return;
    }

    let sum = frame.iter().map(|x| x * x).sum::<f32>();
    let rms = (sum / frame.len() as f32).sqrt();

    // Simple soft clip/normalization. ~0.20 RMS ≈ strong speech.
    let norm = (rms / 0.20).clamp(0.0, 1.0);
    let _ = app.emit("audio:rms", norm);
}

/// Emit RMS from i16 samples (converts to f32 internally)
pub fn emit_rms_i16(app: &AppHandle, frame: &[i16]) {
    if frame.is_empty() {
        let _ = app.emit("audio:rms", 0.0f32);
        return;
    }

    // Convert i16 to f32 for RMS calculation
    let sum: f32 = frame
        .iter()
        .map(|&s| {
            let normalized = s as f32 / i16::MAX as f32;
            normalized * normalized
        })
        .sum();

    let rms = (sum / frame.len() as f32).sqrt();

    // Normalize: ~0.20 RMS ≈ strong speech
    let norm = (rms / 0.20).clamp(0.0, 1.0);
    let _ = app.emit("audio:rms", norm);
}
