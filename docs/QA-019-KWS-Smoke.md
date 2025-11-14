# QA-019 — KWS Real Mode Smoke Test

## Objective

Verify that real keyword spotting (KWS) can be enabled, models can be downloaded and verified, and the system gracefully handles failures.

## Preconditions

- ✅ Network connection available (for first run)
- ✅ At least 50 MB free disk space
- ✅ App compiled with `kws_real` feature (or stub fallback works)
- ✅ Clean state: `~/.local/share/emberleaf/models/kws/` should be empty

## Test Scenarios

### Scenario 1: Successful Enable (Online, First Time)

**Steps**:
1. Launch Emberleaf
2. Navigate to **Advanced → Audio Settings → Wake Word (Real KWS)**
3. Verify model selector shows at least 1 model (from registry)
4. Select a model (e.g., `gigaspeech-en-3.3M`)
5. Click **Enable Real KWS**

**Expected Results**:
- ✅ Download progress bar appears (0% → 100%)
- ✅ "Downloading…" changes to "Verifying…"
- ✅ Green "Model verified" banner appears (3s auto-hide)
- ✅ Toast: "Real KWS enabled"
- ✅ Status pill changes: "Stub mode" → "Real KWS (model_id)"
- ✅ **Enable** button changes to **Disable**

**DevTools Check**:
```js
// Listen for events
window.__TAURI__.event.listen('kws:model_download_progress', (e) => console.log('Progress:', e.payload));
window.__TAURI__.event.listen('kws:model_verified', (e) => console.log('Verified:', e.payload));
window.__TAURI__.event.listen('kws:enabled', (e) => console.log('Enabled:', e.payload));
```

Expected console output:
```
Progress: {model_id: "gigaspeech-en-3.3M", downloaded: 210000, total: 4200000, percent: 5.0}
Progress: {model_id: "gigaspeech-en-3.3M", downloaded: 420000, total: 4200000, percent: 10.0}
...
Progress: {model_id: "gigaspeech-en-3.3M", downloaded: 4200000, total: 4200000, percent: 100.0}
Verified: "gigaspeech-en-3.3M"
Enabled: "gigaspeech-en-3.3M"
```

---

### Scenario 2: Already Downloaded (Cached Model)

**Steps**:
1. With model already downloaded from Scenario 1
2. Click **Disable**
3. Wait for "KWS disabled, returned to stub mode" toast
4. Re-select same model
5. Click **Enable Real KWS**

**Expected Results**:
- ✅ No download progress (instant or <1s)
- ✅ "Model gigaspeech-en-3.3M is already downloaded and verified" toast
- ✅ Status changes to "Real KWS" immediately
- ✅ No `kws:model_download_progress` events (model was cached)

---

### Scenario 3: Offline Mode (Network Unavailable)

**Steps**:
1. Disable network: `nmcli networking off` (or disconnect WiFi)
2. Ensure model **not** cached (remove `~/.local/share/emberleaf/models/kws/gigaspeech-en-3.3M/`)
3. Try to enable real KWS

**Expected Results**:
- ✅ Download fails with friendly error toast:
  - "Model download failed: <network error>"
- ✅ Amber "Model verification failed" banner appears
- ✅ **Enable** button remains enabled (allows retry)
- ✅ Status remains "Stub mode"

**Re-enable network**:
4. Re-enable network: `nmcli networking on`
5. Click **Enable Real KWS** again

**Expected Results**:
- ✅ Download succeeds
- ✅ Model verified
- ✅ Real KWS enabled

---

### Scenario 4: Disk Full (Out of Space)

**Steps**:
1. Simulate low disk space:
   ```bash
   # Create a large file to fill disk
   dd if=/dev/zero of=~/bigfile bs=1M count=<size_to_fill>
   ```
2. Try to enable real KWS

**Expected Results**:
- ✅ Download fails with error toast (disk space)
- ✅ Partial files are cleaned up (verify `~/.local/share/emberleaf/models/kws/` is empty or incomplete)
- ✅ App does not crash

**Cleanup**:
```bash
rm ~/bigfile
```

---

### Scenario 5: Simple Mode Auto-Select

**Steps**:
1. Navigate to **Simple Mode** (Home screen)
2. Verify "Wake Word (Real KWS)" section is visible
3. Status pill shows "Stub mode"
4. Click **Enable Real Recognition** button

**Expected Results**:
- ✅ Auto-selects Spanish model if available, otherwise first model
- ✅ Toast: "Auto-selecting recommended model" (if Spanish found)
- ✅ Download + verify flow as in Scenario 1
- ✅ Status updates to "Real KWS" with model badge

---

### Scenario 6: Disable Real KWS

**Steps**:
1. With real KWS enabled
2. Click **Disable** button

**Expected Results**:
- ✅ Toast: "KWS disabled, returned to stub mode"
- ✅ Status pill: "Real KWS" → "Stub mode"
- ✅ **Disable** button changes to **Enable Real KWS**
- ✅ Audio capture restarts (logs show restart)

**DevTools Check**:
```js
window.__TAURI__.event.listen('kws:disabled', () => console.log('Disabled'));
```

Expected: `Disabled` logged

---

### Scenario 7: Corrupted Model (Verification Failure)

**Steps**:
1. Download a model successfully
2. Manually corrupt a model file:
   ```bash
   echo "corrupted" >> ~/.local/share/emberleaf/models/kws/gigaspeech-en-3.3M/encoder.onnx
   ```
3. Disable real KWS
4. Try to re-enable same model

**Expected Results**:
- ✅ Verification fails (SHA-256 mismatch)
- ✅ Amber banner: "Model verification failed"
- ✅ Toast error: "Model verification failed for 'gigaspeech-en-3.3M'"
- ✅ Corrupted files are **removed** automatically
- ✅ Status remains "Stub mode"

**Next Enable Attempt**:
5. Click **Enable Real KWS** again

**Expected Results**:
- ✅ Model re-downloads (not cached)
- ✅ Verification succeeds
- ✅ Real KWS enabled

---

### Scenario 8: No Models Available (Empty Registry)

**Steps**:
1. Empty the registry:
   ```bash
   echo '{"version":"1.0.0","models":{}}' > ~/.local/share/emberleaf/models/kws_registry.json
   ```
2. Restart Emberleaf
3. Navigate to Wake Word settings

**Expected Results**:
- ✅ Model selector shows: "No models available"
- ✅ **Enable** button is disabled
- ✅ No crashes

**Restore**:
```bash
cp assets/registry/kws_registry.json ~/.local/share/emberleaf/models/
```

---

## DevTools Debugging Snippet

Open DevTools (F12) and run:

```js
// Listen to all KWS events
const events = [
  'kws:model_download_progress',
  'kws:model_verified',
  'kws:model_verify_failed',
  'kws:enabled',
  'kws:disabled',
  'wakeword::detected'
];

events.forEach(event => {
  window.__TAURI__.event.listen(event, (e) => {
    console.log(`[${event}]`, e.payload);
  });
});

console.log('✅ KWS event listeners registered');
```

---

## Acceptance Criteria

All scenarios pass with:
- ✅ No crashes
- ✅ Friendly error messages (no raw stack traces shown to user)
- ✅ Download progress updates smoothly (no stuck at 0%)
- ✅ Verification failures are handled gracefully
- ✅ Mode switching works without app restart
- ✅ ES/EN i18n strings display correctly
- ✅ Web mode shows placeholder data (no Tauri-related crashes)

---

## Logs

Check logs for detailed info:

```bash
tail -f ~/.cache/emberleaf/logs/emberleaf.log
```

Look for:
- `Starting real KWS worker with Sherpa-ONNX`
- `Model 'gigaspeech-en-3.3M' verified successfully`
- `Real KWS enabled with model: gigaspeech-en-3.3M`
- `KWS disabled, returned to stub mode`

---

## Regression Tests

After each change to KWS code, re-run:
1. Scenario 1 (fresh download)
2. Scenario 2 (cached model)
3. Scenario 6 (disable/enable toggle)

---

## Automated Harness (QA-019B)

**Script**: `scripts/qa/run-kws-e2e.mjs`

### Prerequisites

* Real KWS build: `--features kws_real`
* App running: `npm run tauri dev`
* Optional (for deterministic testing): PipeWire loopback

### Running the Harness

**With loopback (deterministic, recommended for CI)**:
```bash
# 1. Set up PipeWire loopback (one-time setup)
pactl load-module module-null-sink sink_name=virtual_speaker
pactl load-module module-loopback source=virtual_speaker.monitor sink=@DEFAULT_SINK@

# 2. Run harness
EMB_LOOPBACK=1 node scripts/qa/run-kws-e2e.mjs
echo $?  # 0 = pass, 1 = fail, 2 = error
```

**Without loopback (manual speech)**:
```bash
node scripts/qa/run-kws-e2e.mjs
# Follow on-screen prompts to say "Hey Ember"
```

**Simulation mode (validates command wiring only, no audio)**:
```bash
EMB_SIM=1 node scripts/qa/run-kws-e2e.mjs
```

### What It Tests

1. **Model listing**: `kws_list_models` returns at least one model
2. **Model selection**: Auto-selects Spanish or first available model
3. **Enable flow**: `kws_enable` triggers download → verify → enabled
4. **Test window**: `kws_arm_test_window` arms 8-second detection window
5. **Detection**: Plays bundled `wake-hey-ember-16k.wav` (loopback) or prompts user speech
6. **Pass event**: Waits for `kws:wake_test_pass` event within timeout
7. **Cleanup**: Disables KWS after test

### Troubleshooting

**"Asset not found: wake-hey-ember-16k.wav"**:
→ Sample file needs to be created. See `assets/audio/README.md` for instructions.
→ Fallback: Use manual speech mode (no `EMB_LOOPBACK`)

**Test times out (no detection)**:
* Check mic input levels in app (Advanced → Audio Settings)
* Verify model is actually enabled (status pill shows "Real KWS")
* Ensure PipeWire loopback is routing correctly: `pactl list sinks`
* Try increasing test window duration in script

**"App not running" or command errors**:
* Start app first: `npm run tauri dev`
* Ensure app is fully loaded (wait for window to appear)
* Check DevTools console for errors

---

## CI Integration (QA-019C)

**GitHub Actions Job**: `qa-kws-e2e-loopback`

The CI pipeline includes an automated wake-word test that runs on every PR and push to `main`/`dev`. This job:

- **Runs on**: `ubuntu-22.04` with PipeWire + WirePlumber
- **Duration**: ~3-4 minutes with caching
- **Required**: Yes (blocks merging if it fails)

### What It Does

1. Installs system dependencies (PipeWire, WirePlumber, xvfb, webkit)
2. Builds the Tauri app in debug mode
3. Starts PipeWire + WirePlumber in a dbus session
4. Sets up audio loopback (null-sink + loopback module)
5. Runs E2E harness with `EMB_LOOPBACK=1` via xvfb
6. Validates exit code: 0=pass, 1=timeout, 2=error

### Local Parity

Run the same test locally with:

```bash
make qa-kws-e2e-loopback
```

This uses `dbus-run-session` + `xvfb-run` to match CI conditions.

### Viewing CI Logs

If the job fails in CI:

1. Go to **Actions** tab in GitHub
2. Click the failing workflow run
3. Expand **QA KWS E2E (PipeWire Loopback)** job
4. Check **Run E2E harness (loopback)** step for harness output
5. Look for errors in **Start PipeWire & WirePlumber** step

Common CI failures:

- **PipeWire not starting**: dbus session issue (rare on ubuntu-22.04)
- **Timeout (exit 1)**: Model download slow or wake word not detected
- **Harness error (exit 2)**: App not starting, missing deps

### Caching

The job caches:
- Cargo dependencies (~200 MB)
- npm dependencies (~100 MB)
- **Not cached**: Downloaded KWS models (re-download each run)

To enable model caching (optional):

1. Add step after harness run:
   ```yaml
   - uses: actions/cache@v4
     with:
       path: ~/.local/share/emberleaf/models/kws
       key: kws-models-${{ hashFiles('assets/registry/kws_registry.json') }}
   ```

2. Move cache step before harness run to restore cached models

---

## Known Issues / Limitations

- Download progress throttled to 10Hz (may appear choppy)
- No resume support for partial downloads (starts fresh on retry)
- Registry signature verification is placeholder (zeros key)
- macOS/Windows packaging not yet implemented (Linux-first)

---

## References

- [KWS-REAL.md](./KWS-REAL.md) — Full KWS documentation
- [BE-001-kws.md](./BE-001-kws.md) — Original KWS implementation spec
