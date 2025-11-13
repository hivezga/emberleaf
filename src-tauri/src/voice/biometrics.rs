#[cfg(feature = "kws_real")]
use crate::ffi::sherpa_onnx_bindings::*;
use anyhow::{bail, Context, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
#[cfg(feature = "kws_real")]
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

/// Configuration for speaker biometrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricsConfig {
    /// Minimum number of enrollment utterances
    pub enroll_utterances_min: usize,
    /// Minimum duration per utterance (ms)
    pub utterance_min_ms: u64,
    /// Cosine similarity threshold for verification
    pub verify_threshold: f32,
    /// Maximum verification duration (ms)
    pub max_verify_ms: u64,
}

impl Default for BiometricsConfig {
    fn default() -> Self {
        Self {
            enroll_utterances_min: 3,
            utterance_min_ms: 2000,
            verify_threshold: 0.82,
            max_verify_ms: 4000,
        }
    }
}

/// Enrollment progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentProgress {
    pub user: String,
    pub utterances_collected: usize,
    pub utterances_required: usize,
    pub completed: bool,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub user: String,
    pub verified: bool,
    pub score: f32,
    pub threshold: f32,
}

/// Profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub user: String,
    pub created_at: String,
    pub utterances_count: usize,
}

/// Encrypted voiceprint storage
#[derive(Serialize, Deserialize)]
struct EncryptedVoiceprint {
    /// XChaCha20-Poly1305 nonce (192-bit)
    nonce: Vec<u8>,
    /// Encrypted embedding data
    ciphertext: Vec<u8>,
    /// Metadata (unencrypted)
    created_at: String,
    utterances_count: usize,
}

/// Speaker biometrics system using ECAPA-TDNN
#[cfg(feature = "kws_real")]
pub struct SpeakerBiometrics {
    config: BiometricsConfig,
    model_path: PathBuf,
    profiles_dir: PathBuf,
    embedding_extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
    encryption_key: Zeroizing<[u8; 32]>,
    enrollment_state: Arc<Mutex<Option<EnrollmentState>>>,
    sample_rate: u32,
    _model_path_cstr: CString,
}

/// Placeholder when kws_real feature is not enabled
#[cfg(not(feature = "kws_real"))]
pub struct SpeakerBiometrics {
    _private: std::marker::PhantomData<()>,
}

/// Internal enrollment state
#[cfg(feature = "kws_real")]
struct EnrollmentState {
    user: String,
    embeddings: Vec<Vec<f32>>,
    required_count: usize,
}

#[cfg(feature = "kws_real")]
impl SpeakerBiometrics {
    /// Create a new speaker biometrics system
    pub fn new(
        model_path: PathBuf,
        profiles_dir: PathBuf,
        config: BiometricsConfig,
        sample_rate: u32,
    ) -> Result<Self> {
        log::info!(
            "Initializing speaker biometrics from: {}",
            model_path.display()
        );

        // Verify model file exists
        if !model_path.exists() {
            bail!(
                "Speaker embedding model not found: {}",
                model_path.display()
            );
        }

        // Create profiles directory if it doesn't exist
        fs::create_dir_all(&profiles_dir).context("Failed to create profiles directory")?;

        // Convert model path to CString
        let model_path_cstr = CString::new(model_path.to_str().unwrap())
            .context("Failed to convert model path to CString")?;

        // Create Sherpa-ONNX speaker embedding extractor config
        let extractor_config = SherpaOnnxSpeakerEmbeddingExtractorConfig {
            model: model_path_cstr.as_ptr(),
            num_threads: 2,
            debug: 0,
            provider: std::ptr::null(), // Use default provider (CPU)
        };

        log::info!("Creating speaker embedding extractor...");
        let embedding_extractor =
            unsafe { SherpaOnnxCreateSpeakerEmbeddingExtractor(&extractor_config) };

        if embedding_extractor.is_null() {
            bail!("Failed to create speaker embedding extractor. Check model file.");
        }

        // Get embedding dimension
        let dim = unsafe { SherpaOnnxSpeakerEmbeddingExtractorDim(embedding_extractor) };
        log::info!("Speaker embedding dimension: {}", dim);

        // Generate or load encryption key
        let encryption_key = Self::get_or_create_encryption_key(&profiles_dir)?;

        log::info!(
            "Speaker biometrics initialized: sample_rate={}Hz, threshold={:.2}",
            sample_rate,
            config.verify_threshold
        );

        Ok(Self {
            config,
            model_path,
            profiles_dir,
            embedding_extractor,
            encryption_key,
            enrollment_state: Arc::new(Mutex::new(None)),
            sample_rate,
            _model_path_cstr: model_path_cstr,
        })
    }

    /// Get or create encryption key for voiceprint storage
    fn get_or_create_encryption_key(profiles_dir: &Path) -> Result<Zeroizing<[u8; 32]>> {
        let key_path = profiles_dir.join(".key");

        let key = if key_path.exists() {
            // Load existing key
            let key_bytes = fs::read(&key_path).context("Failed to read encryption key")?;
            if key_bytes.len() != 32 {
                bail!("Invalid encryption key length");
            }
            let mut key_array = Zeroizing::new([0u8; 32]);
            key_array.copy_from_slice(&key_bytes);
            key_array
        } else {
            // Generate new key
            let mut key_array = Zeroizing::new([0u8; 32]);
            OsRng.fill_bytes(&mut *key_array);

            // Save key (with restrictive permissions)
            fs::write(&key_path, &*key_array).context("Failed to write encryption key")?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))
                    .context("Failed to set key file permissions")?;
            }

            log::info!("Generated new encryption key");
            key_array
        };

        Ok(key)
    }

    /// Extract speaker embedding from audio samples
    fn extract_embedding(&self, samples: &[f32]) -> Result<Vec<f32>> {
        // Create online stream
        let stream =
            unsafe { SherpaOnnxSpeakerEmbeddingExtractorCreateStream(self.embedding_extractor) };

        if stream.is_null() {
            bail!("Failed to create speaker embedding stream");
        }

        // Feed audio samples
        unsafe {
            SherpaOnnxOnlineStreamAcceptWaveform(
                stream,
                self.sample_rate as i32,
                samples.as_ptr(),
                samples.len() as i32,
            );

            // Signal end of audio
            SherpaOnnxOnlineStreamInputFinished(stream);
        }

        // Check if ready
        let is_ready =
            unsafe { SherpaOnnxSpeakerEmbeddingExtractorIsReady(self.embedding_extractor, stream) };

        if is_ready == 0 {
            unsafe {
                SherpaOnnxDestroyOnlineStream(stream);
            }
            bail!("Embedding extractor not ready (audio may be too short)");
        }

        // Compute embedding
        let embedding_ptr = unsafe {
            SherpaOnnxSpeakerEmbeddingExtractorComputeEmbedding(self.embedding_extractor, stream)
        };

        if embedding_ptr.is_null() {
            unsafe {
                SherpaOnnxDestroyOnlineStream(stream);
            }
            bail!("Failed to compute speaker embedding");
        }

        // Get embedding dimension
        let dim =
            unsafe { SherpaOnnxSpeakerEmbeddingExtractorDim(self.embedding_extractor) } as usize;

        // Copy embedding to Rust Vec
        let embedding = unsafe { std::slice::from_raw_parts(embedding_ptr, dim).to_vec() };

        // Free resources
        unsafe {
            SherpaOnnxSpeakerEmbeddingExtractorDestroyEmbedding(embedding_ptr);
            SherpaOnnxDestroyOnlineStream(stream);
        }

        Ok(embedding)
    }

    /// Compute cosine similarity between two embeddings
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Normalize an embedding to unit length
    fn normalize_embedding(embedding: &mut [f32]) {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in embedding.iter_mut() {
                *x /= norm;
            }
        }
    }

    /// Average multiple embeddings
    fn average_embeddings(embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dim = embeddings[0].len();
        let mut avg = vec![0.0f32; dim];

        for embedding in embeddings {
            for (i, &val) in embedding.iter().enumerate() {
                avg[i] += val;
            }
        }

        let count = embeddings.len() as f32;
        for val in avg.iter_mut() {
            *val /= count;
        }

        avg
    }

    /// Encrypt embedding data
    fn encrypt_embedding(&self, embedding: &[f32]) -> Result<EncryptedVoiceprint> {
        let cipher = XChaCha20Poly1305::new((&*self.encryption_key).into());

        // Generate random nonce
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from(nonce_bytes);

        // Serialize embedding as bytes
        let plaintext: Vec<u8> = embedding.iter().flat_map(|&f| f.to_le_bytes()).collect();

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        Ok(EncryptedVoiceprint {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            created_at: chrono::Utc::now().to_rfc3339(),
            utterances_count: 0, // Will be set by caller
        })
    }

    /// Decrypt embedding data
    fn decrypt_embedding(&self, encrypted: &EncryptedVoiceprint) -> Result<Vec<f32>> {
        let cipher = XChaCha20Poly1305::new((&*self.encryption_key).into());

        let nonce: &XNonce = encrypted
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        // Deserialize bytes to f32 array
        let embedding: Vec<f32> = plaintext
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok(embedding)
    }

    /// Get profile file path for a user
    fn profile_path(&self, user: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.voiceprint", user))
    }

    /// Start enrollment for a user
    pub fn enroll_start(&self, user: String) -> Result<()> {
        let mut state = self.enrollment_state.lock().unwrap();

        if state.is_some() {
            bail!("Enrollment already in progress");
        }

        log::info!("Starting enrollment for user: {}", user);

        *state = Some(EnrollmentState {
            user,
            embeddings: Vec::new(),
            required_count: self.config.enroll_utterances_min,
        });

        Ok(())
    }

    /// Add an enrollment sample (audio samples as f32)
    pub fn enroll_add_sample(&self, samples: &[f32]) -> Result<EnrollmentProgress> {
        let mut state = self.enrollment_state.lock().unwrap();

        let enrollment = state.as_mut().ok_or_else(|| {
            anyhow::anyhow!("No enrollment in progress. Call enroll_start first.")
        })?;

        // Check minimum duration (rough estimate: samples / sample_rate * 1000)
        let duration_ms = (samples.len() as f32 / self.sample_rate as f32 * 1000.0) as u64;
        if duration_ms < self.config.utterance_min_ms {
            bail!(
                "Utterance too short: {}ms (minimum {}ms)",
                duration_ms,
                self.config.utterance_min_ms
            );
        }

        // Extract embedding
        let mut embedding = self.extract_embedding(samples)?;
        Self::normalize_embedding(&mut embedding);

        enrollment.embeddings.push(embedding);

        log::info!(
            "Enrollment progress: {}/{}",
            enrollment.embeddings.len(),
            enrollment.required_count
        );

        Ok(EnrollmentProgress {
            user: enrollment.user.clone(),
            utterances_collected: enrollment.embeddings.len(),
            utterances_required: enrollment.required_count,
            completed: enrollment.embeddings.len() >= enrollment.required_count,
        })
    }

    /// Finalize enrollment and save voiceprint
    pub fn enroll_finalize(&self) -> Result<ProfileInfo> {
        let mut state = self.enrollment_state.lock().unwrap();

        let enrollment = state
            .take()
            .ok_or_else(|| anyhow::anyhow!("No enrollment in progress"))?;

        if enrollment.embeddings.len() < enrollment.required_count {
            bail!(
                "Not enough utterances: {}/{}",
                enrollment.embeddings.len(),
                enrollment.required_count
            );
        }

        // Average embeddings
        let mut avg_embedding = Self::average_embeddings(&enrollment.embeddings);
        Self::normalize_embedding(&mut avg_embedding);

        // Encrypt voiceprint
        let mut encrypted = self.encrypt_embedding(&avg_embedding)?;
        encrypted.utterances_count = enrollment.embeddings.len();

        // Save to disk
        let profile_path = self.profile_path(&enrollment.user);
        let json =
            serde_json::to_string_pretty(&encrypted).context("Failed to serialize voiceprint")?;
        fs::write(&profile_path, json).context("Failed to write voiceprint file")?;

        log::info!(
            "Enrollment complete for user '{}': {} utterances, saved to {}",
            enrollment.user,
            enrollment.embeddings.len(),
            profile_path.display()
        );

        Ok(ProfileInfo {
            user: enrollment.user,
            created_at: encrypted.created_at,
            utterances_count: encrypted.utterances_count,
        })
    }

    /// Cancel ongoing enrollment
    pub fn enroll_cancel(&self) {
        let mut state = self.enrollment_state.lock().unwrap();
        if let Some(enrollment) = state.take() {
            log::info!("Enrollment cancelled for user: {}", enrollment.user);
        }
    }

    /// Verify a speaker against a stored voiceprint
    pub fn verify(&self, user: &str, samples: &[f32]) -> Result<VerificationResult> {
        // Load voiceprint
        let profile_path = self.profile_path(user);
        if !profile_path.exists() {
            bail!("No voiceprint found for user: {}", user);
        }

        let json = fs::read_to_string(&profile_path).context("Failed to read voiceprint file")?;
        let encrypted: EncryptedVoiceprint =
            serde_json::from_str(&json).context("Failed to deserialize voiceprint")?;

        // Decrypt voiceprint
        let stored_embedding = self.decrypt_embedding(&encrypted)?;

        // Extract embedding from input audio
        let mut test_embedding = self.extract_embedding(samples)?;
        Self::normalize_embedding(&mut test_embedding);

        // Compute similarity
        let score = Self::cosine_similarity(&stored_embedding, &test_embedding);

        let verified = score >= self.config.verify_threshold;

        log::info!(
            "Verification for user '{}': score={:.3}, threshold={:.3}, result={}",
            user,
            score,
            self.config.verify_threshold,
            if verified { "PASS" } else { "FAIL" }
        );

        Ok(VerificationResult {
            user: user.to_string(),
            verified,
            score,
            threshold: self.config.verify_threshold,
        })
    }

    /// Check if a profile exists for a user
    pub fn profile_exists(&self, user: &str) -> bool {
        self.profile_path(user).exists()
    }

    /// Delete a user's voiceprint
    pub fn delete_profile(&self, user: &str) -> Result<()> {
        let profile_path = self.profile_path(user);
        if !profile_path.exists() {
            bail!("No voiceprint found for user: {}", user);
        }

        fs::remove_file(&profile_path).context("Failed to delete voiceprint")?;
        log::info!("Deleted voiceprint for user: {}", user);

        Ok(())
    }

    /// List all enrolled users
    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut users = Vec::new();

        for entry in
            fs::read_dir(&self.profiles_dir).context("Failed to read profiles directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("voiceprint") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    users.push(stem.to_string());
                }
            }
        }

        Ok(users)
    }
}

#[cfg(feature = "kws_real")]
impl Drop for SpeakerBiometrics {
    fn drop(&mut self) {
        // Cleanup Sherpa-ONNX resources
        unsafe {
            if !self.embedding_extractor.is_null() {
                SherpaOnnxDestroySpeakerEmbeddingExtractor(self.embedding_extractor);
            }
        }
        log::info!("Speaker biometrics resources released");
    }
}

// Stub impl when kws_real is not enabled
#[cfg(not(feature = "kws_real"))]
impl SpeakerBiometrics {
    pub fn new(
        _model_path: PathBuf,
        _profiles_dir: PathBuf,
        _config: BiometricsConfig,
        _sample_rate: u32,
    ) -> Result<Self> {
        bail!("Speaker biometrics requires kws_real feature. Build with --features kws_real")
    }

    pub fn enroll_start(&self, _user: String) -> Result<()> {
        bail!("Speaker biometrics not available")
    }

    pub fn enroll_add_sample(&self, _samples: &[f32]) -> Result<EnrollmentProgress> {
        bail!("Speaker biometrics not available")
    }

    pub fn enroll_finalize(&self) -> Result<ProfileInfo> {
        bail!("Speaker biometrics not available")
    }

    pub fn enroll_cancel(&self) {}

    pub fn verify(&self, _user: &str, _samples: &[f32]) -> Result<VerificationResult> {
        bail!("Speaker biometrics not available")
    }

    pub fn profile_exists(&self, _user: &str) -> bool {
        false
    }

    pub fn delete_profile(&self, _user: &str) -> Result<()> {
        bail!("Speaker biometrics not available")
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

// Mark SpeakerBiometrics as Send for use across threads
#[cfg(feature = "kws_real")]
unsafe impl Send for SpeakerBiometrics {}

#[cfg(not(feature = "kws_real"))]
unsafe impl Send for SpeakerBiometrics {}

#[cfg(all(test, feature = "kws_real"))]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((SpeakerBiometrics::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((SpeakerBiometrics::cosine_similarity(&a, &b) - 0.0).abs() < 0.001);

        let a = vec![1.0, 1.0];
        let b = vec![-1.0, -1.0];
        assert!((SpeakerBiometrics::cosine_similarity(&a, &b) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_embedding() {
        let mut embedding = vec![3.0, 4.0];
        SpeakerBiometrics::normalize_embedding(&mut embedding);

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_average_embeddings() {
        let embeddings = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![3.0, 4.0, 5.0],
        ];

        let avg = SpeakerBiometrics::average_embeddings(&embeddings);
        assert_eq!(avg, vec![2.0, 3.0, 4.0]);
    }
}
