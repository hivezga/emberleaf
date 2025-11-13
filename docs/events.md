# Emberleaf Audio Events

This document describes the Tauri events emitted by Emberleaf's audio system for frontend consumption.

## Overview

All events are emitted from the Rust backend using Tauri's event system and can be listened to from the frontend using `@tauri-apps/api/event`.

## Event Reference

### `audio:rms`

Emitted continuously during audio capture to provide real-time microphone level visualization.

**Payload Type:** `number`

**Payload:** Normalized RMS value (0.0-1.0)
- `0.0` = silence
- `~0.20` = strong speech
- `1.0` = maximum (clipped)

**Frequency:** ~20-50ms intervals

**Example:**
```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen<number>("audio:rms", (event) => {
  console.log("Mic level:", event.payload);
  setMicLevel(event.payload);
});
```

---

### `audio:auto_probe_started`

Emitted when auto-probe begins scanning input devices for active audio.

**Payload Type:** `null` (no payload)

**Trigger:** User clicks "Check my microphone" button

**Example:**
```typescript
await listen("audio:auto_probe_started", () => {
  console.log("Auto-probe started...");
  setProbing(true);
});
```

---

### `audio:auto_probe_suggestion`

Emitted when auto-probe detects a device with active audio and suggests it to the user.

**Payload Type:**
```typescript
{
  device: string;  // Device name
  reason: string;  // Human-readable explanation
}
```

**Example Payloads:**
```json
{
  "device": "USB Audio Device",
  "reason": "Detected speech on this device (RMS: 0.124)"
}
```

**Example:**
```typescript
await listen<{ device: string; reason: string }>("audio:auto_probe_suggestion", (event) => {
  const { device, reason } = event.payload;
  toast.success(`Suggested: ${device} - ${reason}`);
  setSuggestedDevice(device);
});
```

---

### `audio:restart_ok`

Emitted when audio capture restarts successfully.

**Payload Type:**
```typescript
{
  device: string;     // Device name
  elapsed_ms: number; // Restart duration in milliseconds
}
```

**Example Payload:**
```json
{
  "device": "Built-in Microphone",
  "elapsed_ms": 1823
}
```

**Example:**
```typescript
await listen<{ device: string; elapsed_ms: number }>("audio:restart_ok", (event) => {
  console.log(`Audio restarted in ${event.payload.elapsed_ms}ms`);
});
```

---

### `audio:restart_blocked`

Emitted when a restart is attempted while another restart is already in progress.

**Payload Type:** `null` (no payload)

**Trigger:** Reentrancy guard prevents overlapping restarts

**Example:**
```typescript
await listen("audio:restart_blocked", () => {
  toast.warning("Restart already in progress, please wait...");
});
```

---

### `audio:test_tone_played`

Emitted when a test tone finishes playing successfully.

**Payload Type:**
```typescript
{
  device: string;         // Output device name
  frequency_hz: number;   // Tone frequency
  duration_ms: number;    // Duration
  volume: number;         // Volume (0.0-1.0)
}
```

**Example Payload:**
```json
{
  "device": "Built-in Speaker",
  "frequency_hz": 660,
  "duration_ms": 250,
  "volume": 0.15
}
```

**Example:**
```typescript
await listen<{
  device: string;
  frequency_hz: number;
  duration_ms: number;
  volume: number;
}>("audio:test_tone_played", (event) => {
  console.log("Test tone played:", event.payload);
});
```

---

## Usage Pattern

Typical setup in a React component:

```typescript
useEffect(() => {
  let unlistenRms: (() => void) | undefined;
  let unlistenSuggestion: (() => void) | undefined;
  let unlistenRestart: (() => void) | undefined;

  async function setupListeners() {
    try {
      const { listen } = await import("@tauri-apps/api/event");

      // RMS for mic meter
      unlistenRms = await listen<number>("audio:rms", (event) => {
        setMicLevel(event.payload);
      });

      // Auto-probe suggestions
      unlistenSuggestion = await listen<{ device: string; reason: string }>(
        "audio:auto_probe_suggestion",
        (event) => {
          setSuggestedDevice(event.payload.device);
          toast.success(event.payload.reason);
        }
      );

      // Restart completion
      unlistenRestart = await listen<{ device: string; elapsed_ms: number }>(
        "audio:restart_ok",
        (event) => {
          toast.success(`Reconnected in ${event.payload.elapsed_ms}ms`);
        }
      );
    } catch (err) {
      console.log("Event listeners not available (web mode)");
    }
  }

  setupListeners();

  return () => {
    if (unlistenRms) unlistenRms();
    if (unlistenSuggestion) unlistenSuggestion();
    if (unlistenRestart) unlistenRestart();
  };
}, []);
```

---

### `audio:error`

Emitted when an audio error occurs, with user-friendly error messages.

**Payload Type:**
```typescript
{
  code: string;    // Error code (device_busy, device_not_found, permission_denied, etc.)
  message: string; // User-friendly error message
}
```

**Error Codes:**
- `device_busy` - Another app is using the microphone
- `device_not_found` - Microphone was unplugged or is no longer available
- `permission_denied` - Permission denied (check system settings)
- `timeout` - Audio device timed out
- `no_device` - No microphone found
- `unknown` - Unknown audio error

**Example Payload:**
```json
{
  "code": "device_busy",
  "message": "Another app is using this microphone. Close other audio apps and try again."
}
```

**Example:**
```typescript
await listen<{ code: string; message: string }>("audio:error", (event) => {
  const { code, message } = event.payload;
  toast.error(message);

  // Handle specific error codes
  if (code === "device_busy") {
    // Show suggestion to close other apps
  }
});
```

---

## Error Handling

All events are fire-and-forget from the backend. If no listeners are attached, events are silently dropped. Frontend code should gracefully handle missing events when running in web mode (non-Tauri environment).

---

### `audio:device_lost`

Emitted when a configured audio device is no longer available (unplugged or disconnected).

**Payload Type:**
```typescript
{
  kind: "input" | "output";  // Device type
  previous: DeviceId;         // Stable ID of the lost device
}
```

**Example Payload:**
```json
{
  "kind": "input",
  "previous": {
    "host_api": "ALSA",
    "index": 2,
    "name": "USB Audio Device"
  }
}
```

**Example:**
```typescript
await listen<{ kind: string; previous: DeviceId }>("audio:device_lost", (event) => {
  const { kind, previous } = event.payload;
  console.log(`${kind} device '${previous.name}' was disconnected`);
  toast.warning(`Microphone '${previous.name}' was disconnected`);
});
```

---

### `audio:device_fallback_ok`

Emitted when the app successfully falls back to a default device after device loss.

**Payload Type:**
```typescript
{
  kind: "input" | "output";  // Device type
  new_device: string;         // Name of the fallback device
}
```

**Example Payload:**
```json
{
  "kind": "input",
  "new_device": "default"
}
```

**Example:**
```typescript
await listen<{ kind: string; new_device: string }>("audio:device_fallback_ok", (event) => {
  toast.success(`Switched to ${event.payload.new_device} ${event.payload.kind}`);
});
```

---

### `audio:device_fallback_failed`

Emitted when the app fails to fall back to a default device after device loss.

**Payload Type:**
```typescript
{
  kind: "input" | "output";  // Device type
  reason: string;             // Failure reason
}
```

**Example Payload:**
```json
{
  "kind": "input",
  "reason": "No input device available"
}
```

**Example:**
```typescript
await listen<{ kind: string; reason: string }>("audio:device_fallback_failed", (event) => {
  toast.error(`Failed to recover: ${event.payload.reason}`);
});
```

---

### `audio:monitor_guarded`

Emitted when mic monitor cannot be resumed after device change due to feedback risk.

**Payload Type:**
```typescript
{
  reason: string;  // Reason for guard (e.g., "feedback_risk")
}
```

**Example Payload:**
```json
{
  "reason": "feedback_risk"
}
```

**Example:**
```typescript
await listen<{ reason: string }>("audio:monitor_guarded", (event) => {
  console.log(`Monitor not resumed: ${event.payload.reason}`);
});
```

---

## Future Events

Planned for upcoming releases:
- `audio:monitor_started` - Mic monitor activated
- `audio:monitor_stopped` - Mic monitor deactivated
