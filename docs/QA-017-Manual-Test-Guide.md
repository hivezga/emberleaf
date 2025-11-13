# QA-017 Manual Test Guide
## Device Resilience, Persistence & Polish

This guide covers manual testing for BE/FE-017: Device loss detection, auto-fallback, monitor persistence, and related UI improvements.

---

## Prerequisites

- Emberleaf app built and running (`npm run tauri dev`)
- At least one USB microphone available for plug/unplug testing
- Audio output device (speakers/headphones) for monitor testing
- Test on Linux (primary target platform)

---

## Test Scenarios

### 1. Device Loss & Successful Fallback

**Setup:**
1. Connect a USB microphone
2. Open Emberleaf
3. Use DevicePicker to select the USB microphone
4. Verify audio meter shows activity when speaking

**Steps:**
1. While app is running, physically unplug the USB microphone
2. Observe the behavior

**Expected Results:**
- Within ~3 seconds, a **green success banner** appears at the top of the screen
- Banner text (EN): "Switched to the default device."
- Banner text (ES): "Cambiamos al dispositivo predeterminado."
- Banner has a "Change…" / "Cambiar…" button
- Audio meter resumes showing activity with the default device
- Banner auto-dismisses after ~6 seconds
- Can manually dismiss banner with X button
- Backend logs show:
  ```
  Device loss detected
  Successfully fell back to default input device
  Mic monitor resumed (if applicable)
  ```

**Pass Criteria:**
- ✅ Banner appears within 3s of unplug
- ✅ Audio continues working with default device
- ✅ Banner shows correct localized text
- ✅ Auto-dismiss works after 6s
- ✅ Manual dismiss works

---

### 2. Device Loss & Failed Fallback

**Setup:**
1. Connect only ONE USB microphone (no built-in mic)
2. Select it in Emberleaf
3. Verify it's working

**Steps:**
1. Unplug the microphone while app is running
2. Observe the behavior

**Expected Results:**
- **Amber/yellow warning banner** appears
- Banner text (EN): "We couldn't switch automatically."
- Banner text (ES): "No pudimos cambiar automáticamente."
- Banner shows two buttons:
  - "Retry" / "Reintentar"
  - "Change…" / "Cambiar…"
- Banner does NOT auto-dismiss (persists until resolved)
- Audio meter shows no activity
- Clicking "Retry" attempts to restart audio capture
- Clicking "Change…" opens the DevicePicker
- Backend logs show:
  ```
  Device loss detected
  Failed to fallback: No input device available
  ```

**Pass Criteria:**
- ✅ Warning banner appears with correct text
- ✅ Banner persists (no auto-dismiss)
- ✅ "Retry" button triggers restart attempt
- ✅ "Change…" opens DevicePicker
- ✅ Can manually dismiss with X

---

### 3. Monitor Guarded (Feedback Prevention)

**Setup:**
1. Configure input and output to be different devices initially
2. Enable mic monitoring in Advanced settings
3. Verify monitor is active (hear mic passthrough)

**Steps:**
1. Unplug the input device
2. App falls back to default (which might be same as output)
3. Observe monitor behavior

**Expected Results:**
- If fallback results in input == output:
  - Monitor does NOT resume (no audible feedback)
  - **Blue info banner** appears briefly (~8s)
  - Banner text (EN): "Monitoring wasn't resumed to avoid feedback."
  - Banner text (ES): "No reanudamos el monitoreo para evitar retroalimentación."
  - Backend logs show:
    ```
    Cannot resume monitor after fallback: input and output are the same device (feedback prevention)
    ```
- If fallback results in input != output:
  - Monitor resumes safely
  - No guarded banner appears

**Pass Criteria:**
- ✅ Monitor does not resume when unsafe
- ✅ Guarded banner appears with correct text
- ✅ No audible feedback occurs
- ✅ Banner auto-dismisses after ~8s
- ✅ Monitor resumes when safe

---

### 4. Stable-ID Tooltip in DevicePicker

**Setup:**
1. Open DevicePicker (Simple or Advanced mode)
2. Look at device rows

**Steps:**
1. Hover over the small Info icon next to device names
2. Read the tooltip that appears

**Expected Results:**
- Info icon (ℹ) visible next to each device name
- On hover, tooltip appears showing:
  - (EN): "Stable ID: {host_api}#{index}"
  - (ES): "ID estable: {host_api}#{index}"
- Example: "Stable ID: ALSA#2" or "ID estable: ALSA#2"
- Tooltip is readable and positioned correctly
- Works with keyboard navigation (focus)

**Pass Criteria:**
- ✅ Info icon visible for all devices
- ✅ Tooltip shows on hover
- ✅ Text format is correct: `{host}#{index}`
- ✅ Tooltip is localized properly
- ✅ Keyboard accessible

---

### 5. Monitor Persistence Toggle

**Setup:**
1. Navigate to Advanced settings (AudioSettings component)
2. Find the "Monitor" section

**Steps:**
1. Locate the "Remember mic monitoring" toggle
2. Toggle it ON
3. Start mic monitor
4. Restart the application
5. Observe monitor state

**Expected Results:**
- Toggle labeled:
  - (EN): "Remember mic monitoring"
  - (ES): "Recordar monitoreo del micrófono"
- When toggle is ON:
  - Starting monitor persists the ON state to config
  - After app restart, monitor auto-resumes (if safe)
  - If unsafe (input == output), guarded message appears
- When toggle is OFF:
  - Monitor state is not persisted
  - After app restart, monitor is OFF regardless of previous state

**Additional:**
- If `audio:monitor_guarded` event fires:
  - Small badge/help text appears under the toggle
  - Text (EN): "Not resumed to avoid feedback."
  - Text (ES): "No se reanudó para evitar retroalimentación."

**Pass Criteria:**
- ✅ Toggle present in Advanced settings
- ✅ Label is localized
- ✅ Persists monitor state when ON
- ✅ Does not persist when OFF
- ✅ Auto-resumes safely on restart
- ✅ Guarded badge appears when applicable

---

### 6. Localization (ES/EN)

**Setup:**
1. Test with both Spanish and English language settings
2. Trigger all device loss scenarios

**Steps:**
1. Set language to Spanish (if supported in UI)
2. Trigger device loss
3. Verify all banner text is in Spanish
4. Switch to English
5. Trigger device loss again
6. Verify all banner text is in English

**Expected Results:**
- All banner messages use correct language:
  - Device loss messages
  - Fallback success/failure
  - Monitor guarded
  - Button labels ("Change…", "Retry")
  - Tooltip text in DevicePicker
  - Toggle label in settings
- Text is natural and non-technical
- No English text appears in Spanish mode (and vice versa)

**Pass Criteria:**
- ✅ All UI text matches selected language
- ✅ Messages are clear and non-technical
- ✅ No mixed-language strings
- ✅ Format placeholders work correctly ({name}, {id})

---

### 7. Comprehensive Device Replug Test

**Setup:**
1. USB microphone connected and selected
2. Monitor active (if desired)
3. Audio meter showing activity

**Steps:**
1. Unplug USB mic → observe fallback
2. Wait for success banner to appear
3. Replug the same USB mic
4. Open DevicePicker
5. Verify the USB mic reappears
6. Select it and apply
7. Verify stable ID matches (same device recognized)

**Expected Results:**
- Unplug: Success fallback banner appears
- Replug: Device reappears in DevicePicker
- Stable ID unchanged (same host_api + index + name)
- Can re-select and use the device
- Audio resumes normally
- Monitor resumes if it was active and safe

**Pass Criteria:**
- ✅ Smooth fallback on unplug
- ✅ Device reappears on replug
- ✅ Stable ID is consistent
- ✅ Can re-select and use device
- ✅ No crashes or dead ends

---

### 8. Timing & Performance

**Setup:**
1. Normal operation with USB mic

**Steps:**
1. Unplug device
2. Measure time to fallback
3. Observe CPU/memory usage

**Expected Results:**
- Fallback occurs within **2-3 seconds** of unplug
- No noticeable CPU spike
- No memory leaks after multiple plug/unplug cycles
- App remains responsive throughout
- No stuttering in audio meter during fallback

**Pass Criteria:**
- ✅ Fallback < 3s
- ✅ No performance degradation
- ✅ App stays responsive
- ✅ No memory leaks

---

### 9. Edge Cases

**Test A: Multiple Rapid Plug/Unplug**
1. Rapidly unplug and replug device 3-4 times
2. Verify app doesn't crash or get confused
3. Verify banners don't stack up

**Test B: No Audio Devices**
1. Disable all audio devices in OS
2. Start Emberleaf
3. Verify friendly error message (not crash)

**Test C: Device Busy**
1. Open another app using the mic (e.g., Audacity)
2. Try to use that device in Emberleaf
3. Verify friendly "Another app is using this mic" error

**Pass Criteria:**
- ✅ Handles rapid changes gracefully
- ✅ Shows friendly errors (not technical)
- ✅ No crashes on edge cases

---

## Regression Checks

Verify that existing functionality still works:

1. **Audio Meter**: RMS levels display smoothly
2. **EMA Smoothing**: Meter doesn't flicker on brief peaks
3. **Clipping Hint**: "Too loud" warning appears when appropriate
4. **Device Selection**: Can manually change devices via DevicePicker
5. **Auto-Probe**: "Check my microphone" button works
6. **Test Tone**: Plays correctly on selected output
7. **Mic Monitor**: Start/stop works normally
8. **Config Persistence**: Device selections saved across restarts
9. **Reentrancy Guard**: Multiple rapid restarts don't cause issues

---

## Known Limitations

- Device watcher checks every 2 seconds (not instant detection)
- CPAL stream error callbacks not yet implemented (future enhancement)
- Watcher only monitors configured device (not all devices)

---

## Reporting Issues

If tests fail, report:
1. Scenario number and name
2. Expected vs actual behavior
3. Backend logs (check terminal output)
4. Screenshots of banners (if UI issue)
5. Device names and OS (e.g., "USB Audio Device on Arch Linux")

---

## Success Criteria

All tests must pass with:
- ✅ Correct banner behavior (appearance, text, timing)
- ✅ Audio continues working after device loss
- ✅ No crashes or unhandled errors
- ✅ Localized text displays correctly
- ✅ Performance within acceptable limits
- ✅ No regressions in existing features
