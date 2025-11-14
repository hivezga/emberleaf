# Audio Assets for QA-019 Testing

## wake-hey-ember-16k.wav

**Status**: TODO - Needs to be recorded

**Specifications**:
- Format: WAV (PCM)
- Sample Rate: 16000 Hz
- Channels: 1 (mono)
- Bit Depth: 16-bit signed integer
- Duration: ~0.8-1.0 seconds
- RMS Level: 0.18-0.22 (avoid clipping)
- Content: Clear pronunciation of "hey ember"

**How to Create**:

1. **Record with Audacity** (or similar):
   - Set project rate to 16000 Hz
   - Record "hey ember" clearly
   - Normalize to -12 dBFS peak
   - Export as WAV (16-bit PCM)

2. **Or use SoX** (command-line):
   ```bash
   # Record from microphone
   sox -d -r 16000 -c 1 -b 16 wake-hey-ember-16k.wav trim 0 1.0

   # Normalize
   sox wake-hey-ember-16k.wav wake-hey-ember-16k-normalized.wav norm -0.5
   ```

3. **Or use Python** (synthetic placeholder for testing infrastructure):
   ```python
   import numpy as np
   from scipy.io import wavfile

   # Generate a simple chirp as placeholder (NOT ACTUAL WAKE WORD)
   sample_rate = 16000
   duration = 0.8
   t = np.linspace(0, duration, int(sample_rate * duration))

   # Chirp from 200Hz to 800Hz
   f0, f1 = 200, 800
   chirp = np.sin(2 * np.pi * (f0 * t + (f1 - f0) * t**2 / (2 * duration)))

   # Apply envelope
   envelope = np.exp(-3 * np.abs(t - duration/2) / duration)
   signal = chirp * envelope * 0.2

   # Convert to 16-bit PCM
   signal_int16 = (signal * 32767).astype(np.int16)

   wavfile.write('wake-hey-ember-16k.wav', sample_rate, signal_int16)
   ```

**Placeholder for Now**:

Until a real voice recording is available, the test harness will fail gracefully if this file is missing. To test the infrastructure without the actual wake word:

1. Create a silent or tone placeholder:
   ```bash
   sox -n -r 16000 -c 1 -b 16 wake-hey-ember-16k.wav trim 0 0.8 synth 0.8 sine 440
   ```

2. Or skip loopback testing and use manual speech testing.

## Future Assets

- `wake-hey-ember-es-16k.wav` - Spanish pronunciation
- `wake-hey-ember-zh-16k.wav` - Chinese pronunciation
- Additional test samples for false positive testing
