# QA-015 Manual Testing Guide

This guide provides step-by-step instructions for executing manual tests that require live Tauri execution with audio devices.

---

## Prerequisites

1. **Environment Setup**
   ```bash
   cd /mnt/data/dev/Ember
   # Verify audio devices available
   pactl list short sources
   pactl list short sinks
   ```

2. **Start Tauri Dev Server**
   ```bash
   # For Wayland/wlroots, force X11:
   GDK_BACKEND=x11 SHERPA_ONNX_DIR="/home/hivezga/.local/sherpa-onnx" \
   LD_LIBRARY_PATH="/home/hivezga/.local/sherpa-onnx/lib" \
   npm run tauri dev

   # Or use the shortcut:
   npm run dev:x11
   ```

3. **Open Browser Console**
   - Press `F12` or `Ctrl+Shift+I`
   - Go to **Console** tab
   - Keep open for event monitoring

---

## Test 1: First-Run Simple Mode (Spanish)

### Setup Event Listeners

Paste this in browser console:

```javascript
// Monitor RMS (mic level)
await window.__TAURI__.event.listen('audio:rms', e => {
  console.log('🎤 RMS:', e.payload.toFixed(3));
});

// Monitor auto-probe
await window.__TAURI__.event.listen('audio:auto_probe_started', () => {
  console.log('🔍 Auto-probe started');
});

await window.__TAURI__.event.listen('audio:auto_probe_suggestion', e => {
  console.log('💡 Suggestion:', e.payload);
});
```

### Manual Steps

1. **Verify Default Language**
   - UI should display Spanish (ES-MX)
   - Header: "Emberleaf"
   - Card title: "Audio listo"
   - Button: "Verificar micrófono"

2. **Check Mic Meter**
   - Speak into default microphone
   - Meter bar should move (green → yellow → red gradient)
   - Console shows RMS values: `0.05 - 0.40` during speech

3. **Run Auto-Probe**
   - Click **"Verificar micrófono"**
   - Console shows: `🔍 Auto-probe started`
   - Speak for 3 seconds
   - If current mic is good: Status shows ✅ "Micrófono detectado: [name] (Bien)"
   - If silent: Console shows `💡 Suggestion: { suggested: "...", reason: "..." }`

### Expected Results

- ✅ UI in Spanish
- ✅ Meter responds to voice (0.05-0.4 RMS)
- ✅ Status updates based on audio activity
- ✅ Events emitted in console
- ✅ No technical jargon (sample rates, channels hidden)

---

## Test 2: Input Switch + Hot Restart

### Setup Event Listeners

```javascript
await window.__TAURI__.event.listen('audio:restart_ok', e => {
  console.log('✅ Restart OK:', e.payload);
  console.log('⏱️ Elapsed:', e.payload.elapsed_ms + 'ms');
  if (e.payload.elapsed_ms > 3000) {
    console.error('❌ FAIL: Restart took > 3s!');
  }
});

await window.__TAURI__.event.listen('audio:restart_blocked', () => {
  console.warn('⚠️ Restart blocked (reentrancy guard)');
});
```

### Manual Steps

1. **Open Device Picker**
   - Click **"Cambiar micrófono"**
   - Drawer slides in from right

2. **Select Different Device**
   - Choose a mic different from current (e.g., USB webcam vs built-in)
   - Notice badge: "(Predeterminado)" or "(Activo recientemente)"
   - Inline feedback appears: "Aplicar ahora"

3. **Apply Hot Restart**
   - Click **"Aplicar"** button
   - Observe console timing
   - Meter should freeze briefly, then resume < 2s

4. **Test Reentrancy Guard (Edge Case)**
   - Quickly double-click **"Aplicar"**
   - Second click should show: `⚠️ Restart blocked`
   - OR toast: "Restart already in progress..."

### Expected Results

- ✅ Device picker shows grouped lists
- ✅ Restart completes < 3s (check console `elapsed_ms`)
- ✅ Event: `audio:restart_ok` with timing
- ✅ Meter resumes after restart
- ✅ Double-click blocked gracefully

---

## Test 3: Output Device + Test Tone

### Setup Event Listener

```javascript
await window.__TAURI__.event.listen('audio:test_tone_played', e => {
  console.log('🔊 Test tone:', e.payload);
  console.log('   Device:', e.payload.device);
  console.log('   Freq:', e.payload.frequency_hz + 'Hz');
  console.log('   Duration:', e.payload.duration_ms + 'ms');
  console.log('   Volume:', (e.payload.volume * 100).toFixed(0) + '%');
});
```

### Manual Steps

1. **Auto Test Tone on Device Change**
   - Toggle **Advanced mode** ON (top-right)
   - Go to "Output Device" dropdown
   - Select different output (e.g., headset vs speakers)
   - Should hear short beep automatically (660Hz, 250ms)

2. **Manual Test Tone**
   - Click **"Probar sonido"** button
   - Should hear slightly longer beep (880Hz, 400ms)
   - Verify console event emission

3. **Verify Safety Caps (Simple Mode)**
   - Toggle **Advanced mode** OFF
   - Click **"Probar sonido"**
   - Check console: `duration_ms` ≤ 300, `volume` ≤ 0.25

### Expected Results

- ✅ Auto beep on output device change
- ✅ Manual test tone plays
- ✅ Event: `audio:test_tone_played` with parameters
- ✅ Simple mode caps: 300ms max, 25% volume max
- ✅ No clipping, no ear-piercing volume

---

## Test 4: Mic Monitor & Feedback Prevention

### Manual Steps

1. **Enable Mic Monitor**
   - Toggle **Advanced mode** ON
   - Find **"Monitor de micrófono"** toggle
   - Switch ON
   - Should hear your mic routed to speakers (low gain)

2. **Test Feedback Prevention**
   - Set input and output to SAME device (e.g., both "Built-in")
   - Try to enable Monitor
   - Should get warning toast: feedback loop prevented

3. **Auto-Resume After Restart**
   - With Monitor ON and safe devices (input ≠ output)
   - Change input device and click **"Aplicar"**
   - After restart, Monitor should auto-resume

4. **Disable Monitor**
   - Toggle OFF
   - Verify audio returns to normal (no monitoring)

### Expected Results

- ✅ Monitor routes input → output (audible)
- ✅ Feedback prevention when input == output
- ✅ Auto-resume after safe restart
- ✅ Clean disable when toggled OFF

---

## Test 5: Diagnostics Modal

### Manual Steps

1. **Open Diagnostics**
   - Toggle **Advanced mode** ON
   - Click **"Diagnosticar audio"** (or "Diagnose Audio" in EN)
   - Modal opens

2. **Review Snapshot Fields**
   - **Summary:** Timestamp, Last Restart, Monitor Active, Processing Rate
   - **Selected Devices:** Input, Output
   - **Available Inputs:** List with "(default)" badges
   - **Available Outputs:** List with "(default)" badges
   - **Full JSON:** Pretty-printed snapshot

3. **Copy JSON**
   - Click **"Copy JSON"** button
   - Paste in text editor
   - Verify valid JSON structure

### Expected JSON Structure

```json
{
  "debug_info": {
    "processing_rate": 16000,
    "processing_channels": 1,
    "frame_ms": 20,
    "hop_ms": 10,
    "samples_per_frame": 320,
    "samples_per_hop": 160,
    "input_device": "USB Mic",
    "output_device": "Built-in Speakers"
  },
  "input_devices": [
    { "name": "USB Mic", "is_default": false, "host": "ALSA", ... }
  ],
  "output_devices": [
    { "name": "Built-in Speakers", "is_default": true, ... }
  ],
  "selected_input_device": "USB Mic",
  "selected_output_device": null,
  "monitor_active": false,
  "last_restart_ms": 1731423456789,
  "timestamp_ms": 1731423457890
}
```

### Expected Results

- ✅ All fields present in snapshot
- ✅ Accurate device lists
- ✅ Valid JSON copied to clipboard
- ✅ Modal closes with Esc or X button

---

## Test 7: i18n & Accessibility

### Language Switching

1. **Toggle Advanced Mode**
   - Switch should appear in header
   - Verify persists across page refresh

2. **Switch Language**
   - In Advanced mode, language dropdown appears
   - Select "English"
   - Verify all strings translate:
     - "Check my microphone" (was "Verificar micrófono")
     - "Change microphone" (was "Cambiar micrófono")
     - "Apply now" (was "Aplicar ahora")

3. **Persistence**
   - Refresh page
   - Language selection should persist

### Keyboard Navigation

1. **Tab Through UI**
   - Press `Tab` repeatedly
   - Verify focus rings visible on:
     - Buttons
     - Toggles
     - Dropdowns
   - Focus order should be logical (top → bottom, left → right)

2. **Close Modals with Esc**
   - Open Device Picker
   - Press `Esc`
   - Drawer should close

3. **Screen Reader Test (Optional)**
   - If using Orca/NVDA:
   - Navigate to mic meter
   - Should announce: "Meter, value X.XX out of 1"

### Expected Results

- ✅ English ↔ Spanish toggle works
- ✅ Translations accurate
- ✅ Persistence across refresh
- ✅ Keyboard navigation functional
- ✅ Focus rings visible (4.5:1 contrast)
- ✅ ARIA labels present

---

## Test 8: Edge Cases & Error Handling

### Negative Test 1: Rapid Double-Click

```javascript
// Watch for blocked events
await window.__TAURI__.event.listen('audio:restart_blocked', () => {
  console.log('✅ PASS: Reentrancy guard worked!');
});
```

1. Open Device Picker
2. Select different device
3. **Rapidly double-click "Aplicar"**
4. Verify second click blocked

**Expected:** Toast or console shows "Restart already in progress"

---

### Negative Test 2: Unplug Device During Use

1. Select USB mic as input
2. **Physically unplug the USB mic**
3. Speak or click test button

**Expected:** Friendly error toast "The selected mic was unplugged" (instead of crash)

---

### Negative Test 3: Busy Device

1. Open another app (e.g., Discord, Zoom)
2. Set that app to use specific mic
3. Try to select same mic in Emberleaf

**Expected:** Friendly error "Another app is using this mic."

---

### Negative Test 4: Wayland Quirks

1. Start with: `npm run tauri dev` (pure Wayland)
2. If app crashes or closes unexpectedly:
3. Restart with: `GDK_BACKEND=x11 npm run tauri dev`

**Expected:** App stays stable with X11 fallback

---

## Event Summary Reference

For quick console monitoring, run all these at once:

```javascript
// Complete event monitoring suite
(async () => {
  const { listen } = await import('@tauri-apps/api/event');

  await listen('audio:rms', e => console.log('🎤', e.payload.toFixed(3)));
  await listen('audio:auto_probe_started', () => console.log('🔍 Probe started'));
  await listen('audio:auto_probe_suggestion', e => console.log('💡', e.payload));
  await listen('audio:restart_ok', e => console.log('✅ Restart:', e.payload.elapsed_ms + 'ms'));
  await listen('audio:restart_blocked', () => console.warn('⚠️ Blocked'));
  await listen('audio:test_tone_played', e => console.log('🔊', e.payload));

  console.log('✅ All event listeners registered!');
})();
```

---

## Reporting Issues

If you encounter a failure, document:

1. **Test Number:** (e.g., "Test 2: Hot Restart")
2. **Steps to Reproduce:** Exact sequence
3. **Expected:** What should happen
4. **Actual:** What actually happened
5. **Console Logs:** Copy relevant output
6. **Environment:** `uname -a`, `pactl info`, display server
7. **Severity:** Critical / Major / Minor

**Example Issue:**

```markdown
### Test 2 Failure: Restart Timeout

**Steps:**
1. Opened Device Picker
2. Selected "USB Webcam Mic"
3. Clicked "Aplicar"

**Expected:** Restart < 3s
**Actual:** Restart took 5.2s

**Console:**
```
✅ Restart OK: { elapsed_ms: 5234, message: "..." }
```

**Environment:** Wayland + PipeWire 1.4.9
**Severity:** Major (performance regression)
```

---

## Sign-off Checklist

- [ ] Test 1: First-run Simple Mode (Spanish) - PASS
- [ ] Test 2: Input Switch + Hot Restart - PASS
- [ ] Test 3: Output Device + Test Tone - PASS
- [ ] Test 4: Mic Monitor & Feedback Prevention - PASS
- [ ] Test 5: Diagnostics Modal - PASS
- [ ] Test 7: i18n & Accessibility - PASS
- [ ] Test 8: Edge Cases (4 negative tests) - PASS

**Tester Signature:** ________________
**Date:** ________________
