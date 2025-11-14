//! Real KWS implementation using Sherpa-ONNX v1.10.30
//!
//! This module uses the actual Sherpa-ONNX keyword spotting engine with Zipformer models.

use super::super::vad::{VadConfig, VoiceActivityDetector};
use super::super::{AudioCapture, AudioConfig, AudioSource};
use super::{KwsConfig, WakeWordEvent};
use crate::audio::level;
use crate::ffi::sherpa_onnx_bindings::*;
use crate::paths::AppPaths;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Emitter, Manager};

/// Find a model file by pattern (e.g., "encoder*.onnx"), excluding int8 quantized versions
fn find_model_file(model_dir: &Path, pattern_prefix: &str, extension: &str) -> Result<PathBuf> {
    let entries = std::fs::read_dir(model_dir)
        .with_context(|| format!("Failed to read model directory: {}", model_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Match pattern and exclude int8 quantized versions
            if filename.starts_with(pattern_prefix)
                && filename.ends_with(extension)
                && !filename.contains(".int8.")
            {
                return Ok(path);
            }
        }
    }

    bail!(
        "Model file matching pattern '{}*{}' not found in {}\nRun: npm run setup:audio",
        pattern_prefix,
        extension,
        model_dir.display()
    )
}

/// Load SentencePiece vocabulary from tokens.txt for validation
fn load_sentencepiece_vocab(tokens_path: &Path) -> Result<HashSet<String>> {
    let content = std::fs::read_to_string(tokens_path)
        .with_context(|| format!("Failed to read tokens.txt: {}", tokens_path.display()))?;

    let vocab: HashSet<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            // tokens.txt format: "token score" or just "token"
            // Take first whitespace-delimited field
            line.split_whitespace().next().unwrap_or("").to_string()
        })
        .collect();

    log::debug!("Loaded {} tokens from vocabulary", vocab.len());
    Ok(vocab)
}

/// Normalize keyword against SentencePiece vocabulary
///
/// Strategy:
/// 1. Normalize: lowercase, trim, collapse whitespace
/// 2. Validate: check tokens.txt contains expected whole-word tokens
/// 3. Return normalized string (Sherpa-ONNX will encode it properly)
fn normalize_keyword_against_vocab(raw: &str, tokens_path: &Path) -> Result<String> {
    // Step 1: Normalize (NO uppercase, NO char-splitting, NO trailing punctuation)
    let mut normalized = raw.trim().to_lowercase();
    // Collapse runs of whitespace to single spaces
    normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

    // Remove trailing punctuation
    normalized = normalized
        .trim_end_matches(|c: char| c.is_ascii_punctuation())
        .to_string();

    log::info!("Normalized keyword: '{}' -> '{}'", raw, normalized);

    // Step 2: Load vocabulary and validate
    let vocab = load_sentencepiece_vocab(tokens_path)?;

    // Check for expected whole-word tokens (SentencePiece uses ▁ prefix for word boundaries)
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let mut missing_tokens = Vec::new();

    for word in &words {
        let whole_word_token = format!("▁{}", word);
        if !vocab.contains(&whole_word_token) {
            missing_tokens.push(word.to_string());
        }
    }

    if !missing_tokens.is_empty() {
        log::warn!(
            "tokens.txt lacks expected whole-word entries for: {:?}. \
             Sherpa-ONNX will use subword tokenization. \
             Ensure models & tokens.txt are from the same model pack.",
            missing_tokens
        );
    } else {
        log::info!("✓ All keyword tokens found in vocabulary");
    }

    Ok(normalized)
}

/// Real KWS worker using Sherpa-ONNX
pub struct KwsWorker {
    _thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl KwsWorker {
    /// Start the KWS worker thread with real Sherpa-ONNX implementation
    pub fn start(
        app_handle: AppHandle,
        paths: AppPaths,
        config: KwsConfig,
        vad_config: VadConfig,
        audio_config: AudioConfig,
        model_id: String,
    ) -> Result<Self> {
        log::info!("Starting real KWS worker with Sherpa-ONNX v1.10.30");
        log::info!("  Model ID: {}", model_id);

        // Get model directory for the specified model_id
        let model_dir = paths.kws_model_dir(&model_id);

        if !model_dir.exists() {
            bail!(
                "KWS model directory not found: {}. Enable and download model first.",
                model_dir.display()
            );
        }

        // Spawn worker thread (std::thread to avoid Send issues with FFI pointers)
        let handle = std::thread::spawn(move || {
            if let Err(e) = run_real_kws_worker(
                app_handle,
                config,
                vad_config,
                audio_config,
                model_dir,
                model_id,
            ) {
                log::error!("Real KWS worker thread error: {}", e);
            }
        });

        log::info!("Real KWS worker started");
        Ok(Self {
            _thread_handle: Some(handle),
        })
    }
}

/// Real KWS worker loop with Sherpa-ONNX
fn run_real_kws_worker(
    app_handle: AppHandle,
    config: KwsConfig,
    vad_config: VadConfig,
    audio_config: AudioConfig,
    model_dir: std::path::PathBuf,
    model_id: String,
) -> Result<()> {
    log::info!("Initializing real KWS worker with Sherpa-ONNX");
    log::info!("  Keyword: '{}'", config.keyword);
    log::info!("  Score threshold: {:.2}", config.score_threshold);
    log::info!("  Model dir: {}", model_dir.display());

    // Auto-detect model files by pattern (supports any filename format, excludes int8 quantized)
    let encoder_path = find_model_file(&model_dir, "encoder", ".onnx")?;
    let decoder_path = find_model_file(&model_dir, "decoder", ".onnx")?;
    let joiner_path = find_model_file(&model_dir, "joiner", ".onnx")?;
    let tokens_path = find_model_file(&model_dir, "tokens", ".txt")?;
    let keywords_file = model_dir.join("keywords.txt");

    log::info!(
        "  Encoder: {}",
        encoder_path.file_name().unwrap().to_str().unwrap()
    );
    log::info!(
        "  Decoder: {}",
        decoder_path.file_name().unwrap().to_str().unwrap()
    );
    log::info!(
        "  Joiner: {}",
        joiner_path.file_name().unwrap().to_str().unwrap()
    );
    log::info!(
        "  Tokens: {}",
        tokens_path.file_name().unwrap().to_str().unwrap()
    );

    // Normalize keyword against vocabulary
    let normalized_keyword = normalize_keyword_against_vocab(&config.keyword, &tokens_path)
        .context("Failed to normalize keyword")?;

    log::info!("Using keyword for Sherpa-ONNX: '{}'", normalized_keyword);

    // Dev-only self-check: scan tokens.txt for representative entries
    if let Ok(vocab) = load_sentencepiece_vocab(&tokens_path) {
        log::info!("Vocabulary self-check:");
        let test_tokens = ["▁hey", "▁ember", "▁hi", "▁hello"];
        let mut found = Vec::new();
        let mut missing = Vec::new();

        for token in &test_tokens {
            if vocab.contains(&token.to_string()) {
                found.push(*token);
            } else {
                missing.push(*token);
            }
        }

        if !found.is_empty() {
            log::info!("  ✓ Found representative tokens: {:?}", found);
        }
        if !missing.is_empty() {
            log::warn!("  ✗ Missing common tokens: {:?}", missing);
            log::warn!("    This may indicate model pack mismatch. Consider re-fetching models.");
        }
    }

    // Always recreate keywords file with normalized keyword
    log::info!("Writing keywords file: {}", keywords_file.display());
    let keywords_content = format!("{}\n", normalized_keyword);
    std::fs::write(&keywords_file, &keywords_content).context("Failed to write keywords file")?;

    // Convert paths to CString
    let encoder_cstr = CString::new(encoder_path.to_str().unwrap())?;
    let decoder_cstr = CString::new(decoder_path.to_str().unwrap())?;
    let joiner_cstr = CString::new(joiner_path.to_str().unwrap())?;
    let tokens_cstr = CString::new(tokens_path.to_str().unwrap())?;
    let keywords_cstr = CString::new(keywords_file.to_str().unwrap())?;
    let provider_cstr = CString::new(config.provider.as_str())?;

    // Build Sherpa-ONNX config (v1.10.30 flat structure)
    let feat_config = SherpaOnnxFeatureConfig {
        sample_rate: audio_config.sample_rate_hz as i32,
        feature_dim: 80,
    };

    let transducer_config = SherpaOnnxOnlineTransducerModelConfig {
        encoder: encoder_cstr.as_ptr(),
        decoder: decoder_cstr.as_ptr(),
        joiner: joiner_cstr.as_ptr(),
    };

    let model_config = SherpaOnnxOnlineModelConfig {
        transducer: transducer_config,
        paraformer: Default::default(),
        zipformer2_ctc: Default::default(),
        tokens: tokens_cstr.as_ptr(),
        num_threads: 2,
        provider: provider_cstr.as_ptr(),
        debug: 0,
        model_type: std::ptr::null(),
        modeling_unit: std::ptr::null(),
        bpe_vocab: std::ptr::null(),
        tokens_buf: std::ptr::null(),
        tokens_buf_size: 0,
    };

    let kws_config = SherpaOnnxKeywordSpotterConfig {
        feat_config,
        model_config,
        max_active_paths: config.max_active_paths as i32,
        num_trailing_blanks: 1,
        keywords_score: config.score_threshold,
        keywords_threshold: config.score_threshold,
        keywords_file: keywords_cstr.as_ptr(),
        keywords_buf: std::ptr::null(),
        keywords_buf_size: 0,
    };

    log::info!("Creating Sherpa-ONNX keyword spotter...");
    let kws = unsafe { SherpaOnnxCreateKeywordSpotter(&kws_config) };

    if kws.is_null() {
        bail!("Failed to create Sherpa-ONNX keyword spotter. Check model files.");
    }

    // Create stream
    let stream = unsafe { SherpaOnnxCreateKeywordStream(kws) };
    if stream.is_null() {
        unsafe { SherpaOnnxDestroyKeywordSpotter(kws) };
        bail!("Failed to create keyword spotter stream");
    }

    log::info!("Sherpa-ONNX keyword spotter initialized successfully");

    // Initialize audio capture
    let mut audio_source = AudioCapture::new(audio_config)?;
    log::info!(
        "Audio capture initialized @{}Hz",
        audio_source.sample_rate()
    );

    let mut vad = VoiceActivityDetector::new(vad_config, audio_source.sample_rate())?;
    let mut last_detection: Option<Instant> = None;
    let mut frame_count = 0u64;

    // RMS emission throttle (20 Hz = 50ms)
    let mut last_rms_emit = Instant::now();

    log::info!("Real KWS worker loop started");

    // Main processing loop
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

            // VAD gating (optional, can improve efficiency)
            if !vad.process_frame(&samples) {
                continue;
            }

            // Check refractory period
            if let Some(last) = last_detection {
                if last.elapsed() < Duration::from_millis(config.refractory_ms) {
                    continue;
                }
            }

            // Convert i16 samples to f32 for Sherpa-ONNX
            let samples_f32: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();

            // Feed audio to keyword spotter
            unsafe {
                SherpaOnnxOnlineStreamAcceptWaveform(
                    stream,
                    audio_source.sample_rate() as i32,
                    samples_f32.as_ptr(),
                    samples_f32.len() as i32,
                );
            }

            // Check for keyword detection
            let is_ready = unsafe { SherpaOnnxIsKeywordStreamReady(kws, stream) };

            if is_ready != 0 {
                // Decode result
                unsafe { SherpaOnnxDecodeKeywordStream(kws, stream) };

                // Get keyword result
                let result_ptr = unsafe { SherpaOnnxGetKeywordResult(kws, stream) };

                if !result_ptr.is_null() {
                    let result = unsafe { &*result_ptr };

                    // Check if keyword was detected
                    if !result.keyword.is_null() {
                        let keyword_cstr = unsafe { std::ffi::CStr::from_ptr(result.keyword) };
                        if let Ok(keyword_str) = keyword_cstr.to_str() {
                            if !keyword_str.is_empty() {
                                let score = 1.0; // Sherpa-ONNX uses binary detection

                                log::info!(
                                    "✓ KEYWORD DETECTED [real]: '{}' (frame #{})",
                                    keyword_str,
                                    frame_count
                                );

                                // Emit wake-word event
                                let event = WakeWordEvent {
                                    keyword: keyword_str.to_string(),
                                    score,
                                };

                                if let Err(e) = app_handle.emit("wakeword::detected", &event) {
                                    log::error!("Failed to emit wake-word event: {}", e);
                                }

                                // QA-019: Check if test window is armed and emit test pass event
                                // We emit a separate internal event that main.rs will listen for
                                // to check test window state and conditionally emit kws:wake_test_pass
                                #[derive(serde::Serialize, Clone)]
                                struct TestDetectionPayload {
                                    model_id: String,
                                    keyword: String,
                                    ts: u64,
                                }

                                let ts = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64;

                                let test_payload = TestDetectionPayload {
                                    model_id: model_id.clone(),
                                    keyword: keyword_str.to_string(),
                                    ts,
                                };

                                // Emit internal event for test window checker
                                if let Err(e) =
                                    app_handle.emit("_kws_internal_detection", &test_payload)
                                {
                                    log::error!("Failed to emit internal detection event: {}", e);
                                }

                                last_detection = Some(Instant::now());
                            }
                        }
                    }

                    // Free result
                    unsafe { SherpaOnnxDestroyKeywordResult(result_ptr) };
                }
            }
        } else {
            // Audio source ended (shouldn't happen in normal operation)
            log::warn!("Audio source ended unexpectedly");
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    // Cleanup (unreachable in infinite loop, but good practice)
    // unsafe {
    //     SherpaOnnxDestroyOnlineStream(stream);
    //     SherpaOnnxDestroyKeywordSpotter(kws);
    // }

    // Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_normalize_keyword_against_vocab() {
        // Create a temporary tokens.txt fixture with representative entries
        let temp_dir = std::env::temp_dir();
        let tokens_path = temp_dir.join("test_tokens.txt");

        // Write mini tokens list with SentencePiece format
        let tokens_content = "\
<blk> -100
<sos/eos> -100
<unk> -100
▁hey 0.0
▁ember 0.0
▁hi 0.0
▁hello 0.0
▁world 0.0
";
        std::fs::write(&tokens_path, tokens_content).expect("Failed to write test tokens.txt");

        // Test 1: Normal keyword (already lowercase, no punctuation)
        let result = normalize_keyword_against_vocab("hey ember", &tokens_path)
            .expect("Normalization should succeed");
        assert_eq!(result, "hey ember", "Should return 'hey ember' unchanged");

        // Test 2: Uppercase keyword (should be lowercased)
        let result = normalize_keyword_against_vocab("HEY EMBER", &tokens_path)
            .expect("Normalization should succeed");
        assert_eq!(result, "hey ember", "Should lowercase 'HEY EMBER'");

        // Test 3: Keyword with extra whitespace
        let result = normalize_keyword_against_vocab("  hey   ember  ", &tokens_path)
            .expect("Normalization should succeed");
        assert_eq!(result, "hey ember", "Should trim and collapse whitespace");

        // Test 4: Keyword with trailing punctuation
        let result = normalize_keyword_against_vocab("hey ember!", &tokens_path)
            .expect("Normalization should succeed");
        assert_eq!(result, "hey ember", "Should remove trailing punctuation");

        // Test 5: Mixed case with punctuation
        let result = normalize_keyword_against_vocab("Hey Ember!!!", &tokens_path)
            .expect("Normalization should succeed");
        assert_eq!(
            result, "hey ember",
            "Should normalize mixed case and remove punctuation"
        );

        // Cleanup
        std::fs::remove_file(&tokens_path).ok();
    }

    #[test]
    fn test_load_sentencepiece_vocab() {
        // Create a temporary tokens.txt file
        let temp_dir = std::env::temp_dir();
        let tokens_path = temp_dir.join("test_vocab.txt");

        let tokens_content = "\
<blk> -100
▁hey 0.0
▁ember 0.0
▁test 1.5
";
        std::fs::write(&tokens_path, tokens_content).expect("Failed to write test vocab");

        let vocab = load_sentencepiece_vocab(&tokens_path).expect("Should load vocab");

        assert!(vocab.contains("<blk>"), "Should contain <blk>");
        assert!(vocab.contains("▁hey"), "Should contain ▁hey");
        assert!(vocab.contains("▁ember"), "Should contain ▁ember");
        assert!(vocab.contains("▁test"), "Should contain ▁test");
        assert_eq!(vocab.len(), 4, "Should have 4 tokens");

        // Cleanup
        std::fs::remove_file(&tokens_path).ok();
    }
}
