# Emberleaf

A fully private, offline, open-source voice assistant powered by Sherpa-ONNX and built with Rust + Tauri.

## Features

- **Wake-Word Detection**: Responds to "Hey Ember" using Zipformer KWS
- **Privacy-First**: All processing happens locally, no cloud services
- **Cross-Platform**: Runs on Linux, macOS, and Windows
- **Model Integrity**: SHA256 + Ed25519 verification for all AI models
- **Low Latency**: <500ms detection latency on typical hardware
- **Configurable**: Adjustable sensitivity, VAD, and audio settings

## Quick Start

### Prerequisites

1. **Rust** (latest stable):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (v18+) and npm

3. **Tauri Prerequisites**: https://tauri.app/start/prerequisites/

### Setup

1. **Install Dependencies**
   ```bash
   npm install
   ```

2. **Download KWS Model**

   Place the Sherpa-ONNX KWS model files in the data directory:

   - **Linux:** `~/.local/share/Emberleaf/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/`
   - **macOS:** `~/Library/Application Support/Emberleaf/models/kws/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/`
   - **Windows:** `%LOCALAPPDATA%\Emberleaf\models\kws\sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01\`

   Required files:
   - `encoder.onnx`
   - `decoder.onnx`
   - `joiner.onnx`
   - `tokens.txt`

3. **Run Development Build**
   ```bash
   npm run tauri dev
   ```

4. **Build for Production**
   ```bash
   npm run tauri build
   ```

## Project Structure

```
Ember/
├── src/                    # React frontend
│   ├── App.tsx            # Main application component
│   ├── main.tsx           # React entry point
│   └── *.css              # Styling
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── audio/         # Audio capture and processing
│   │   │   ├── mod.rs     # Audio capture, resampling, AudioSource trait
│   │   │   ├── vad.rs     # Voice Activity Detection (Silero)
│   │   │   └── kws.rs     # Wake-word detection (Zipformer KWS)
│   │   ├── paths.rs       # OS-specific path management
│   │   ├── registry.rs    # Model integrity verification
│   │   ├── main.rs        # Application entry point
│   │   └── lib.rs         # Library exports
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── docs/                  # Documentation
│   └── BE-001-kws.md      # KWS implementation guide
├── config.toml.example    # Example configuration
└── README.md              # This file
```

## Configuration

Configuration is created automatically on first launch at:

- **Linux:** `~/.config/Emberleaf/config.toml`
- **macOS:** `~/Library/Preferences/Emberleaf/config.toml`
- **Windows:** `%APPDATA%\Emberleaf\config\config.toml`

See `config.toml.example` for available options.

### Adjusting Sensitivity

From the frontend:
```typescript
import { invoke } from "@tauri-apps/api/core";

// Set to "low", "balanced", or "high"
await invoke("kws_set_sensitivity", { level: "balanced" });
```

## Events

### Wake-Word Detection

Listen for wake-word detections:

```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("wakeword::detected", (event) => {
  console.log(`Detected: ${event.payload.keyword}`);
  console.log(`Score: ${event.payload.score}`);
});
```

## Development

### Logging

Control log verbosity with `RUST_LOG`:

```bash
# Info level (default)
npm run tauri dev

# Debug level
RUST_LOG=debug npm run tauri dev

# Debug specific module
RUST_LOG=emberleaf_kws=debug npm run tauri dev
```

### Development Mode Features

- Auto-reload on file changes (frontend)
- Allows unknown models (skips registry verification)
- Detailed logging
- DevTools available

### Environment Variables

- `RUST_LOG`: Set logging level
- `EMVER_ALLOW_UNKNOWN_MODELS=1`: Allow models not in registry

## Architecture

Emberleaf uses a modular architecture:

1. **Audio Capture** (CPAL): Cross-platform microphone access
2. **VAD** (Silero): Filter out non-speech audio
3. **KWS** (Sherpa-ONNX): Keyword spotting with Zipformer
4. **Event System** (Tauri): Frontend/backend communication
5. **Model Verification**: Cryptographic integrity checks

See `docs/BE-001-kws.md` for detailed implementation notes.

## Roadmap

- [x] BE-001: Wake-word detection (KWS) foundation
- [ ] BE-002: Speaker embedding & verification (ECAPA-TDNN)
- [ ] BE-003: Local LLM integration (TinyLLaMA/Mistral)
- [ ] BE-004: Text-to-Speech (TTS)
- [ ] BE-005: Privacy-focused web search (Whoogle)

## Troubleshooting

See `docs/BE-001-kws.md` for detailed troubleshooting steps.

**Common Issues:**

- **No detections**: Check microphone permissions and model files
- **High CPU**: Enable VAD, reduce `max_active_paths`
- **False positives**: Increase `score_threshold` or use "low" sensitivity
- **Model errors**: Ensure all 4 model files are present and named correctly

## License

[Add your license here]

## Acknowledgments

- [Sherpa-ONNX](https://github.com/k2-fsa/sherpa-onnx) - Speech recognition toolkit
- [Tauri](https://tauri.app/) - Desktop application framework
- [CPAL](https://github.com/RustAudio/cpal) - Cross-platform audio I/O
