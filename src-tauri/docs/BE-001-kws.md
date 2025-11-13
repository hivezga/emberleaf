# BE-001: Keyword Spotting (KWS) Implementation

## Overview

Emberleaf's wake-word detection system uses Sherpa-ONNX v1.10.30 with Zipformer keyword spotting models for real-time, on-device wake-word detection. The system supports graceful degradation to a stub implementation when the real KWS fails to initialize.

## Architecture

### Components

1. **Real KWS Worker** (`src-tauri/src/audio/kws/real.rs`)
   - Sherpa-ONNX integration with Zipformer models
   - VAD-gated audio processing
   - Keyword normalization and validation
   - SentencePiece vocabulary checking

2. **Stub KWS Worker** (`src-tauri/src/audio/kws/stub.rs`)
   - Energy-based fallback implementation
   - Used for development and when real KWS fails
   - No model dependencies

3. **Unified Worker Interface** (`src-tauri/src/audio/kws/mod.rs`)
   - `KwsWorker` enum that can hold either real or stub implementation
   - `start()` - Starts appropriate worker based on configuration
   - `start_stub()` - Explicitly starts stub worker for fallback

### Non-Fatal Initialization

The KWS system is designed to never crash the application:

- If real KWS fails to initialize:
  1. Logs a warning with the error reason
  2. Automatically falls back to stub KWS worker
  3. Emits `kws::degraded` event to frontend with reason
  4. Application window stays open

## Keyword Normalization

### Strategy

Keywords are normalized before being sent to Sherpa-ONNX to ensure compatibility with the SentencePiece vocabulary used by the model.

### Normalization Rules

The `normalize_keyword_against_vocab()` function applies the following transformations:

1. **Lowercase**: Converts entire string to lowercase
   - Input: `"HEY EMBER"` → Output: `"hey ember"`

2. **Trim**: Removes leading and trailing whitespace
   - Input: `"  hey ember  "` → Output: `"hey ember"`

3. **Collapse Whitespace**: Reduces multiple spaces to single spaces
   - Input: `"hey   ember"` → Output: `"hey ember"`

4. **Remove Trailing Punctuation**: Strips punctuation from the end
   - Input: `"hey ember!!!"` → Output: `"hey ember"`

5. **Validation**: Checks tokens.txt for expected whole-word tokens
   - Looks for `▁hey` and `▁ember` tokens
   - Warns if tokens are missing but continues
   - Sherpa-ONNX will use subword tokenization if needed

### What NOT to Do

- **Never uppercase keywords** - Sherpa-ONNX uses lowercase vocabulary
- **Never split into characters** - Use whole words
- **Never add special characters** - Keep it simple

### Example

```rust
// Good examples
"hey ember"      → "hey ember"     ✓ Perfect
"HEY EMBER"      → "hey ember"     ✓ Normalized
"  Hey Ember!  " → "hey ember"     ✓ Cleaned

// Bad examples (these would fail with old code)
"H E Y  E M B E R" → Would produce invalid tokens
"HEYEMBER"         → Would produce invalid tokens
```

## Vocabulary Self-Check

On startup, the real KWS worker performs a self-check:

```
Vocabulary self-check:
  ✓ Found representative tokens: ["▁hey", "▁ember", "▁hi", "▁hello"]
```

If common tokens are missing:

```
  ✗ Missing common tokens: ["▁hey", "▁ember"]
    This may indicate model pack mismatch. Consider re-fetching models.
```

## Configuration

### KwsConfig

```toml
[kws]
keyword = "hey ember"
score_threshold = 0.60
refractory_ms = 1200
endpoint_ms = 300
provider = "cpu"
max_active_paths = 4
enabled = true
```

### Sensitivity Presets

- **Low**: threshold 0.70, endpoint 350ms (fewer false positives)
- **Balanced**: threshold 0.60, endpoint 300ms (default)
- **High**: threshold 0.50, endpoint 250ms (more responsive)

## Model Files

### Required Files

The KWS worker auto-detects model files by prefix:

- `encoder*.onnx` - Encoder model
- `decoder*.onnx` - Decoder model
- `joiner*.onnx` - Joiner model
- `tokens*.txt` - SentencePiece vocabulary

### Model Detection

Pattern-based detection excludes int8 quantized versions and supports any naming convention:

```
encoder-epoch-12-avg-2-chunk-16-left-64.onnx          ✓ Detected
encoder-epoch-12-avg-2-chunk-16-left-64.int8.onnx    ✗ Excluded (quantized)
```

### Model Paths

Default location: `$HOME/.local/share/Emberleaf/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/`

## Troubleshooting

### Error: "Cannot find ID for token"

**Symptom**: Log shows errors like:
```
EncodeBase: Cannot find ID for token BE at line: ▁HE Y ▁E M BE R.
```

**Cause**: Keyword was improperly formatted (uppercased or char-split)

**Solution**:
- Ensure you're running the latest code with keyword normalization
- The fix automatically lowercases and validates keywords
- Check logs for "Normalized keyword" message

### Error: "Model file not found"

**Symptom**:
```
Required model file not found: encoder.onnx
```

**Solution**:
```bash
bash scripts/fetch_models.sh
```

Or manually download to `~/.local/share/Emberleaf/models/kws/`

### Error: "tokens.txt lacks expected whole-word entries"

**Symptom**:
```
tokens.txt lacks expected whole-word entries for: ["hey", "ember"]
```

**Cause**: Model pack mismatch or incomplete download

**Solution**:
```bash
# Re-fetch models
rm -rf ~/.local/share/Emberleaf/models/kws
bash scripts/fetch_models.sh

# Verify tokens.txt contains entries like:
▁hey 0.0
▁ember 0.0
```

### KWS Degrades to Stub Mode

**Symptom**: App starts but logs show:
```
Real KWS failed: <error>. Falling back to stub KWS.
✓ Stub KWS active (degraded mode)
```

**Cause**: Sherpa-ONNX initialization failed

**Solution**:
1. Check Sherpa-ONNX installation: `ls ~/.local/sherpa-onnx/lib`
2. Verify environment variables in `scripts/dev.sh`:
   ```bash
   export SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx"
   export LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib:..."
   ```
3. Reinstall if needed: `npm run setup:audio:posix`

**Frontend Integration**: Listen for the `kws::degraded` event:
```typescript
listen('kws::degraded', (event) => {
  console.warn('KWS degraded:', event.payload.reason);
  // Show UI banner: "Wake-word detection unavailable"
});
```

### Wayland GBM Buffer Errors

**Symptom**:
```
Failed to create GBM buffer of size 800x600: Invalid argument
```

**Cause**: WebKitGTK DMA-BUF renderer incompatibility

**Solution**: Already fixed in `scripts/dev.sh`:
```bash
export WEBKIT_DISABLE_DMABUF_RENDERER=1
```

## Testing

### Unit Tests

Run keyword normalization tests:
```bash
cargo test --features kws_real --bin ember audio::kws::real::tests
```

Tests verify:
- Lowercase conversion
- Whitespace trimming and collapsing
- Punctuation removal
- Vocabulary loading
- Mixed case + punctuation handling

### Manual Testing

1. Start app with logging:
   ```bash
   bash scripts/dev.sh debug
   ```

2. Look for startup messages:
   ```
   Normalized keyword: 'hey ember' -> 'hey ember'
   Vocabulary self-check:
     ✓ Found representative tokens: ["▁hey", "▁ember", "▁hi", "▁hello"]
   Using keyword for Sherpa-ONNX: 'hey ember'
   ```

3. Speak the wake word and check logs:
   ```
   ✓ KEYWORD DETECTED [real]: 'hey ember' (frame #1234)
   ```

## Environment Variables

Required for Sherpa-ONNX:
```bash
export SHERPA_ONNX_DIR="$HOME/.local/sherpa-onnx"
export LD_LIBRARY_PATH="$HOME/.local/sherpa-onnx/lib:$LD_LIBRARY_PATH"
export PATH="$HOME/.local/sherpa-onnx/bin:$PATH"
```

Optional for debugging:
```bash
export RUST_LOG="info"                                      # General logging
export RUST_LOG="emberleaf::audio::kws=debug"               # KWS debugging
export RUST_LOG="emberleaf::audio::kws=trace"               # Detailed tracing
```

Wayland workarounds:
```bash
export GDK_BACKEND=x11                                      # Force X11/Xwayland
export WEBKIT_DISABLE_DMABUF_RENDERER=1                     # Disable DMA-BUF
```

## Performance

### Resource Usage

- **Real KWS**: ~50-100 MB RAM, ~5-15% CPU (single core)
- **Stub KWS**: ~10-20 MB RAM, ~2-5% CPU

### Latency

- VAD gate: ~30ms per frame
- KWS inference: ~50-100ms per detection
- Total latency: ~100-150ms from speech to event

## References

- Sherpa-ONNX: https://github.com/k2-fsa/sherpa-onnx
- SentencePiece: https://github.com/google/sentencepiece
- Zipformer models: https://github.com/k2-fsa/icefall/tree/master/egs/gigaspeech/ASR

## Implementation Status

- ✅ Real KWS with Sherpa-ONNX Zipformer
- ✅ Stub KWS fallback
- ✅ Non-fatal initialization
- ✅ Keyword normalization
- ✅ Vocabulary validation
- ✅ Self-check logging
- ✅ Pattern-based model detection
- ✅ Unit tests
- ✅ Graceful degradation
- ✅ Frontend event emission

## Version History

- **v0.1.0** (2025-01-11)
  - Initial KWS implementation with Sherpa-ONNX v1.10.30
  - Keyword normalization for SentencePiece compatibility
  - Non-fatal initialization with stub fallback
  - Vocabulary self-check on startup
