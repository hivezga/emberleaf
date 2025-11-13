# KWS-REAL — Real Keyword Spotting with Sherpa-ONNX

## Overview

Emberleaf supports two modes of keyword spotting (KWS) for wake-word detection:

- **Stub Mode** (default): Energy-based RMS detection for development and testing
- **Real Mode**: Sherpa-ONNX Zipformer transducer models for production wake-word recognition

Real KWS enables accurate, offline wake-word detection using neural network models trained on large speech datasets.

## Stub vs. Real Mode

| Feature | Stub Mode | Real Mode |
|---------|-----------|-----------|
| Detection Method | Energy threshold (RMS > 3000) | Neural network (Sherpa-ONNX Zipformer) |
| Accuracy | Low (any loud sound triggers) | High (trained on specific keywords) |
| False Positives | Frequent | Rare |
| Model Required | None | Yes (downloaded on demand) |
| Disk Space | 0 MB | ~4-5 MB per model |
| CPU Usage | Minimal | Moderate |
| Use Case | Development, testing | Production |

## Supported Models

Models are defined in `assets/registry/kws_registry.json` and verified using SHA-256 checksums.

### Current Registry

| Model ID | Language | Wake Word | Size | Description |
|----------|----------|-----------|------|-------------|
| `gigaspeech-en-3.3M` | English (en) | "hey ember" | 4.2 MB | Zipformer KWS trained on GigaSpeech |
| `wenetspeech-zh-3.3M` | Chinese (zh) | "小爱同学" | 4.3 MB | Zipformer KWS trained on WenetSpeech |

**Note**: Model URLs point to official Sherpa-ONNX releases on GitHub.

## How It Works

### 1. Model Discovery

On app startup, Emberleaf loads `~/.local/share/emberleaf/models/kws_registry.json`. The UI populates the model selector from this registry.

### 2. Download & Verification

When you enable real KWS:

1. **Check Local**: If model exists and passes SHA-256 verification, skip download
2. **Download**: Stream model archive from `url` (GitHub/HuggingFace only)
3. **Progress**: Emits `kws:model_download_progress` events (throttled to 10Hz)
4. **Extract**: Unpack `tar.gz` to `~/.local/share/emberleaf/models/kws/{model_id}/`
5. **Verify**: Compare SHA-256 hash of extracted files against registry
   - ✅ **Success**: Emit `kws:model_verified`
   - ❌ **Failure**: Remove files, emit `kws:model_verify_failed`

### 3. Runtime Switching

```
Enable Real KWS:
  → kws_enable(model_id)
  → Download if needed
  → Verify model
  → Restart audio runtime with real KWS
  → Emit kws:enabled

Disable Real KWS:
  → kws_disable()
  → Restart audio runtime with stub KWS
  → Emit kws:disabled
```

**No app restart required** — switching happens at runtime.

### 4. Audio Pipeline Integration

Real KWS integrates with the existing audio capture pipeline:

```
Microphone (16kHz mono)
  → CPAL device callback
  → Silero-VAD gating (optional)
  → Sherpa-ONNX Zipformer KWS
  → Refractory window (1200ms)
  → Emit wakeword::detected
```

## Events

### Download & Verification

#### `kws:model_download_progress`
```json
{
  "model_id": "gigaspeech-en-3.3M",
  "downloaded": 2100000,
  "total": 4200000,
  "percent": 50.0
}
```

#### `kws:model_verified`
```json
"gigaspeech-en-3.3M"
```

#### `kws:model_verify_failed`
```json
"gigaspeech-en-3.3M"
```

### Runtime Control

#### `kws:enabled`
```json
"gigaspeech-en-3.3M"
```

#### `kws:disabled`
```json
null
```

### Wake-Word Detection

#### `wakeword::detected`
```json
{
  "keyword": "hey ember",
  "score": 1.0
}
```

## UI Integration

### Simple Mode

Automatically selects a recommended model (prefers Spanish if available):

1. Shows status pill: "Stub mode" / "Real KWS"
2. Big "Enable Real Recognition" button
3. Auto-downloads and verifies on click
4. Shows progress toast during download

### Advanced Mode

Full control in **Audio Settings → Wake Word (Real KWS)**:

1. Model selector dropdown (language + size badges)
2. Download progress bar (live percent updates)
3. Enable / Disable buttons
4. Verification status banners
5. Current model and keyword display

## Configuration

### Paths

- **Registry**: `~/.local/share/emberleaf/models/kws_registry.json`
- **Models**: `~/.local/share/emberleaf/models/kws/{model_id}/`
- **Config**: `~/.config/Emberleaf/config.toml`

### Config File (TOML)

```toml
[kws]
keyword = "hey ember"
score_threshold = 0.60
refractory_ms = 1200
endpoint_ms = 300
provider = "cpu"
max_active_paths = 4
enabled = true
model_id = "gigaspeech-en-3.3M"  # Set when real KWS is enabled
mode = "real"  # "stub" or "real"
```

## Security

- **URL Allowlist**: Downloads restricted to `github.com` and `huggingface.co`
- **SHA-256 Verification**: All model files verified against registry checksums
- **Model ID Validation**: Alphanumeric + `-_` only, max 64 characters
- **Signature Verification**: Registry can be Ed25519-signed (placeholder currently)

## Troubleshooting

### "Model verification failed"

**Cause**: Downloaded file SHA-256 mismatch (corrupted download or tampered file)

**Solution**:
1. Check network connection
2. Retry enable (files are automatically removed on failure)
3. If persistent, check registry SHA-256 is correct

### "KWS model directory not found"

**Cause**: Model was enabled but files are missing

**Solution**:
1. Disable real KWS
2. Enable again to re-download
3. Check disk space (`df -h ~/.local/share/emberleaf`)

### Download Hangs / Slow

**Cause**: GitHub/HuggingFace CDN issues or slow network

**Solution**:
1. Check internet connection
2. Try again later (GitHub has rate limits)
3. Download manually and place in correct directory

### "Real KWS not available: app was built without kws_real feature"

**Cause**: Binary compiled without `kws_real` feature flag

**Solution**:
```bash
# Enable real KWS at compile time
cargo build --release --features kws_real

# Or use npm script
npm run tauri build -- --features kws_real
```

### High CPU Usage

**Cause**: Sherpa-ONNX inference runs on every audio frame

**Mitigation**:
1. VAD gating reduces inference (only runs when voice detected)
2. Use smaller models (e.g., 3.3M params vs. 13M params)
3. Adjust `max_active_paths` in config (lower = faster but less accurate)

### Offline Usage

Real KWS works fully offline **after** initial model download. Models are cached locally and do not require internet access after download.

To prepare for offline use:
1. Enable real KWS while online (triggers download)
2. Wait for verification to complete
3. Disconnect network — KWS continues to function

## Development

### Adding New Models

1. **Find Model**: Browse [Sherpa-ONNX KWS models](https://github.com/k2-fsa/sherpa-onnx/releases?q=kws)
2. **Compute SHA-256**:
   ```bash
   curl -L <model_url> | sha256sum
   ```
3. **Add to Registry** (`assets/registry/kws_registry.json`):
   ```json
   {
     "your-model-id": {
       "url": "https://github.com/.../model.tar.bz2",
       "sha256": "abc123...",
       "size": 4200000,
       "lang": "en",
       "wakeword": "custom keyword",
       "description": "your-model-id (Custom description)"
     }
   }
   ```
4. **Restart App**: Registry is loaded on startup

### Testing Locally

```bash
# Build with real KWS feature
SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx" \
LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib" \
cargo build --features kws_real

# Run
npm run tauri dev
```

## References

- [Sherpa-ONNX Official Docs](https://k2-fsa.github.io/sherpa/onnx/kws/index.html)
- [Zipformer Architecture](https://arxiv.org/abs/2310.11230)
- [GigaSpeech Dataset](https://github.com/SpeechColab/GigaSpeech)
