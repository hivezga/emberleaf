// Library exports for Tauri
// This file is required for cdylib/staticlib builds

// Link Sherpa-ONNX native libraries when enabled
#[cfg(feature = "kws_real")]
#[link(name = "sherpa-onnx-c-api")]
extern "C" {}

#[cfg(feature = "kws_real")]
#[link(name = "onnxruntime")]
extern "C" {}
