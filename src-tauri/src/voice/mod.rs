//! Voice biometrics module
//!
//! This module provides speaker enrollment and verification using
//! Sherpa-ONNX ECAPA-TDNN embeddings with encrypted storage.

pub mod biometrics;

pub use biometrics::{
    BiometricsConfig, EnrollmentProgress, ProfileInfo, SpeakerBiometrics, VerificationResult,
};
