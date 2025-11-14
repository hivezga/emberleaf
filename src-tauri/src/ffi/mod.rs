//! FFI bindings for Sherpa-ONNX
//!
//! This module provides Rust bindings to the Sherpa-ONNX C API for:
//! - Keyword spotting (KWS)
//! - Speaker embeddings (ECAPA-TDNN)
//!
//! The bindings are generated at build time by build.rs when SHERPA_ONNX_DIR is set.
//! In development mode without Sherpa-ONNX, placeholder stubs are used.

#![allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms, dead_code)]

// Link native libraries when Sherpa-ONNX FFI is enabled
#[cfg(feature = "kws_real")]
#[link(name = "sherpa-onnx-c-api")]
extern "C" {}

#[cfg(feature = "kws_real")]
#[link(name = "onnxruntime")]
extern "C" {}

#[cfg(feature = "kws_real")]
pub mod sherpa_onnx_bindings;

// Placeholder types for when Sherpa-ONNX is not available
#[cfg(not(feature = "kws_real"))]
pub mod sherpa_onnx_bindings {
    //! Placeholder bindings when Sherpa-ONNX is not available
    //!
    //! These allow the code to compile in development mode without Sherpa-ONNX.
    //! The actual KWS/speaker recognition will use placeholder implementations.

    use std::os::raw::{c_char, c_float, c_int};

    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct SherpaOnnxKeywordSpotterConfig {
        pub encoder: *const c_char,
        pub decoder: *const c_char,
        pub joiner: *const c_char,
        pub tokens: *const c_char,
        pub provider: *const c_char,
        pub num_threads: c_int,
        pub max_active_paths: c_int,
        pub sample_rate: c_int,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct SherpaOnnxKeywordSpotter {
        _private: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct SherpaOnnxKeywordSpotterStream {
        _private: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct SherpaOnnxKeywordResult {
        pub keyword: *const c_char,
        pub score: c_float,
        pub start_time: c_float,
    }

    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct SherpaOnnxSpeakerEmbeddingExtractorConfig {
        pub model: *const c_char,
        pub provider: *const c_char,
        pub num_threads: c_int,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct SherpaOnnxSpeakerEmbeddingExtractor {
        _private: [u8; 0],
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct SherpaOnnxSpeakerEmbeddingExtractorStream {
        _private: [u8; 0],
    }

    // Placeholder functions (will panic if called without real Sherpa-ONNX)
    pub unsafe fn SherpaOnnxCreateKeywordSpotter(
        _config: *const SherpaOnnxKeywordSpotterConfig,
    ) -> *mut SherpaOnnxKeywordSpotter {
        panic!("Sherpa-ONNX not available. Set SHERPA_ONNX_DIR and rebuild.");
    }

    pub unsafe fn SherpaOnnxDestroyKeywordSpotter(_spotter: *mut SherpaOnnxKeywordSpotter) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxCreateKeywordSpotterStream(
        _spotter: *const SherpaOnnxKeywordSpotter,
    ) -> *mut SherpaOnnxKeywordSpotterStream {
        panic!("Sherpa-ONNX not available. Set SHERPA_ONNX_DIR and rebuild.");
    }

    pub unsafe fn SherpaOnnxDestroyKeywordSpotterStream(
        _stream: *mut SherpaOnnxKeywordSpotterStream,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxAcceptWaveform(
        _stream: *mut SherpaOnnxKeywordSpotterStream,
        _samples: *const c_float,
        _n: c_int,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxIsKeywordStreamReady(
        _stream: *const SherpaOnnxKeywordSpotterStream,
    ) -> c_int {
        0
    }

    pub unsafe fn SherpaOnnxDecodeKeywordStream(
        _spotter: *const SherpaOnnxKeywordSpotter,
        _stream: *const SherpaOnnxKeywordSpotterStream,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxGetKeywordResult(
        _stream: *const SherpaOnnxKeywordSpotterStream,
    ) -> *const SherpaOnnxKeywordResult {
        std::ptr::null()
    }

    pub unsafe fn SherpaOnnxCreateSpeakerEmbeddingExtractor(
        _config: *const SherpaOnnxSpeakerEmbeddingExtractorConfig,
    ) -> *mut SherpaOnnxSpeakerEmbeddingExtractor {
        panic!("Sherpa-ONNX not available. Set SHERPA_ONNX_DIR and rebuild.");
    }

    pub unsafe fn SherpaOnnxDestroySpeakerEmbeddingExtractor(
        _extractor: *mut SherpaOnnxSpeakerEmbeddingExtractor,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorDim(
        _extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
    ) -> c_int {
        192 // ECAPA-TDNN default dimension
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorCreateStream(
        _extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
    ) -> *mut SherpaOnnxSpeakerEmbeddingExtractorStream {
        std::ptr::null_mut()
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorDestroyStream(
        _stream: *mut SherpaOnnxSpeakerEmbeddingExtractorStream,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorAcceptWaveform(
        _extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
        _stream: *mut SherpaOnnxSpeakerEmbeddingExtractorStream,
        _samples: *const c_float,
        _n: c_int,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorInputFinished(
        _extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
        _stream: *mut SherpaOnnxSpeakerEmbeddingExtractorStream,
    ) {
        // No-op for placeholder
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorIsReady(
        _extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
        _stream: *const SherpaOnnxSpeakerEmbeddingExtractorStream,
    ) -> c_int {
        0 // Not ready in placeholder
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorComputeEmbedding(
        _extractor: *const SherpaOnnxSpeakerEmbeddingExtractor,
        _stream: *const SherpaOnnxSpeakerEmbeddingExtractorStream,
    ) -> *const c_float {
        std::ptr::null()
    }

    pub unsafe fn SherpaOnnxSpeakerEmbeddingExtractorDestroyEmbedding(_embedding: *const c_float) {
        // No-op for placeholder
    }
}

/// Check if Sherpa-ONNX FFI is available
pub fn is_sherpa_onnx_available() -> bool {
    cfg!(feature = "kws_real")
}
