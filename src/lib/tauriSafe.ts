/**
 * Safe Tauri API wrappers with web environment detection
 *
 * These functions safely check for Tauri environment before invoking
 * commands, providing mock/fallback behavior in browser environment.
 */

import type { InvokeArgs } from "@tauri-apps/api/core";

/**
 * Check if we're running in Tauri environment
 */
export async function isTauriEnv(): Promise<boolean> {
  try {
    if (typeof window === "undefined") return false;
    // @ts-expect-error - __TAURI__ is injected at runtime
    return window.__TAURI__ !== undefined;
  } catch {
    return false;
  }
}

/**
 * Safe invoke wrapper
 */
export async function tauriInvoke<T>(
  cmd: string,
  args?: InvokeArgs
): Promise<T> {
  if (!(await isTauriEnv())) {
    throw new Error(`Cannot invoke '${cmd}' in web environment`);
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// ===== AUDIO DEVICE MANAGEMENT =====

export interface DeviceId {
  host_api: string;
  index: number;
  name: string;
}

export interface DeviceInfo {
  name: string;
  is_default: boolean;
  host: string;
  max_channels: number;
  sample_rates: number[];
  stable_id?: DeviceId;
}

export async function listInputDevices(): Promise<DeviceInfo[]> {
  if (!(await isTauriEnv())) {
    return [
      { name: "Default Microphone", is_default: true, host: "web", max_channels: 1, sample_rates: [16000, 44100] },
      { name: "USB Mic (Simulated)", is_default: false, host: "web", max_channels: 1, sample_rates: [16000, 48000] },
    ];
  }
  return tauriInvoke<DeviceInfo[]>("list_input_devices");
}

export async function listOutputDevices(): Promise<DeviceInfo[]> {
  if (!(await isTauriEnv())) {
    return [
      { name: "Default Speakers", is_default: true, host: "web", max_channels: 2, sample_rates: [44100, 48000] },
      { name: "HDMI Output (Simulated)", is_default: false, host: "web", max_channels: 2, sample_rates: [48000] },
    ];
  }
  return tauriInvoke<DeviceInfo[]>("list_output_devices");
}

export async function currentInputDevice(): Promise<string | null> {
  if (!(await isTauriEnv())) return null;
  return tauriInvoke<string | null>("current_input_device");
}

export async function currentOutputDevice(): Promise<string | null> {
  if (!(await isTauriEnv())) return null;
  return tauriInvoke<string | null>("current_output_device");
}

export async function setInputDevice(
  name: string,
  persist: boolean = true
): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Would set input to: ${name} (persist=${persist})`;
  }
  return tauriInvoke<string>("set_input_device", { name, persist });
}

export async function setOutputDevice(
  name: string,
  persist: boolean = true
): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Would set output to: ${name} (persist=${persist})`;
  }
  return tauriInvoke<string>("set_output_device", { name, persist });
}

// ===== AUDIO RUNTIME MANAGEMENT =====

export async function restartAudioCapture(): Promise<string> {
  if (!(await isTauriEnv())) {
    return "[Web] Simulated audio restart";
  }
  return tauriInvoke<string>("restart_audio_capture");
}

export async function playTestTone(opts?: {
  deviceName?: string | null;
  frequencyHz?: number;
  durationMs?: number;
  volume?: number;
}): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Simulated test tone: ${opts?.frequencyHz ?? 440}Hz for ${opts?.durationMs ?? 500}ms`;
  }
  return tauriInvoke<string>("play_test_tone", {
    deviceName: opts?.deviceName ?? null,
    frequencyHz: opts?.frequencyHz ?? 440,
    durationMs: opts?.durationMs ?? 500,
    volume: opts?.volume ?? 0.2,
  });
}

// ===== MICROPHONE MONITORING =====

export async function startMicMonitor(gain?: number): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Simulated mic monitor start (gain=${gain ?? 0.15})`;
  }
  return tauriInvoke<string>("start_mic_monitor", { gain: gain ?? 0.15 });
}

export async function stopMicMonitor(): Promise<string> {
  if (!(await isTauriEnv())) {
    return "[Web] Simulated mic monitor stop";
  }
  return tauriInvoke<string>("stop_mic_monitor");
}

export async function setPersistMonitorState(enabled: boolean): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Would set persist_monitor_state to: ${enabled}`;
  }
  return tauriInvoke<string>("set_persist_monitor_state", { enabled });
}

// ===== CONFIG MANAGEMENT =====

export interface UiConfig {
  focus_ring_contrast_min: number;
  min_touch_target_px: number;
  persist_monitor_state: boolean;
  monitor_was_on: boolean;
}

export interface AudioConfig {
  device_name: string | null;
  output_device_name: string | null;
  stable_input_id: DeviceId | null;
  stable_output_id: DeviceId | null;
}

export interface AppConfig {
  ui: UiConfig;
  audio: AudioConfig;
}

export async function getConfig(): Promise<AppConfig> {
  if (!(await isTauriEnv())) {
    return {
      ui: {
        focus_ring_contrast_min: 1.5,
        min_touch_target_px: 44,
        persist_monitor_state: false,
        monitor_was_on: false,
      },
      audio: {
        device_name: null,
        output_device_name: null,
        stable_input_id: null,
        stable_output_id: null,
      },
    };
  }
  return tauriInvoke<AppConfig>("get_config");
}

// ===== PREFLIGHT CHECKS =====

export type CheckStatus = "pass" | "warn" | "fail";

export interface PreflightItem {
  name: string;
  status: CheckStatus;
  message: string;
  fix_hint?: string;
}

export interface PreflightReport {
  items: PreflightItem[];
  overall: CheckStatus;
  can_proceed: boolean;
}

export async function runPreflightChecks(): Promise<PreflightReport> {
  if (!(await isTauriEnv())) {
    // Mock successful preflight in web mode
    return {
      items: [
        {
          name: "audio_stack",
          status: "pass",
          message: "Web audio API available",
        },
        {
          name: "webkit",
          status: "pass",
          message: "Browser environment",
        },
        {
          name: "portal",
          status: "pass",
          message: "N/A (web mode)",
        },
        {
          name: "mic_access",
          status: "pass",
          message: "Browser will prompt for permissions",
        },
      ],
      overall: "pass",
      can_proceed: true,
    };
  }
  return tauriInvoke<PreflightReport>("run_preflight_checks");
}

// ===== AUDIO DEBUG INFO =====

export interface AudioDebugInfo {
  processing_rate: number;
  processing_channels: number;
  frame_ms: number;
  hop_ms: number;
  samples_per_frame: number;
  samples_per_hop: number;
  input_device: string | null;
  output_device: string | null;
}

export async function getAudioDebug(): Promise<AudioDebugInfo> {
  if (!(await isTauriEnv())) {
    return {
      processing_rate: 16000,
      processing_channels: 1,
      frame_ms: 20,
      hop_ms: 10,
      samples_per_frame: 320,
      samples_per_hop: 160,
      input_device: null,
      output_device: null,
    };
  }
  return tauriInvoke<AudioDebugInfo>("get_audio_debug");
}

// ===== AUDIO SNAPSHOT (DIAGNOSTICS) =====

export interface AudioSnapshot {
  debug_info: AudioDebugInfo;
  input_devices: DeviceInfo[];
  output_devices: DeviceInfo[];
  selected_input_device: string | null;
  selected_output_device: string | null;
  monitor_active: boolean;
  last_restart_ms: number;
  timestamp_ms: number;
}

export async function getAudioSnapshot(): Promise<AudioSnapshot> {
  if (!(await isTauriEnv())) {
    const now = Date.now();
    return {
      debug_info: await getAudioDebug(),
      input_devices: await listInputDevices(),
      output_devices: await listOutputDevices(),
      selected_input_device: null,
      selected_output_device: null,
      monitor_active: false,
      last_restart_ms: now - 30000, // 30s ago
      timestamp_ms: now,
    };
  }
  return tauriInvoke<AudioSnapshot>("get_audio_snapshot");
}

// ===== BE-015 NEW COMMANDS =====

export interface ProbeResult {
  suggested: string | null;
  reason: string;
}

export async function suggestInputDevice(): Promise<ProbeResult> {
  if (!(await isTauriEnv())) {
    return {
      suggested: "Default Microphone",
      reason: "Simulated auto-probe in web environment",
    };
  }
  return tauriInvoke<ProbeResult>("suggest_input_device");
}

export interface RestartResponse {
  ok: boolean;
  message: string;
  elapsedMs?: number;
  reason?: string;
}

// Note: restartAudioCapture now returns structured response
export async function restartAudioCaptureV2(): Promise<RestartResponse> {
  if (!(await isTauriEnv())) {
    return {
      ok: true,
      message: "[Web] Simulated audio restart",
      elapsedMs: 123,
    };
  }
  return tauriInvoke<RestartResponse>("restart_audio_capture");
}

// ===== GLOBAL ERROR LISTENER (SEC-001B) =====

import { toUserMessage } from "./errors";
import { toast } from "sonner";

let subscribed = false;

/**
 * Subscribe to global audio:error events and show toast notifications
 * Should be called once at app startup
 */
export async function subscribeGlobalErrors() {
  if (subscribed || !(await isTauriEnv())) return;
  const { listen } = await import("@tauri-apps/api/event");
  await listen("audio:error", (e) => {
    const msg = toUserMessage(e.payload as any);
    toast.error(msg, { duration: 5000 });
  });
  subscribed = true;
}

// ===== KWS (KEYWORD SPOTTING) MANAGEMENT =====

export interface KwsStatus {
  mode: "stub" | "real";
  model_id?: string;
  keyword: string;
  lang?: string;
  enabled: boolean;
}

export interface KwsModelEntry {
  url: string;
  sha256: string;
  size: number;
  lang: string;
  wakeword: string;
  description: string;
}

export async function kwsStatus(): Promise<KwsStatus> {
  if (!(await isTauriEnv())) {
    return {
      mode: "stub",
      keyword: "hey ember",
      enabled: true,
    };
  }
  return tauriInvoke<KwsStatus>("kws_status");
}

export async function kwsListModels(): Promise<KwsModelEntry[]> {
  if (!(await isTauriEnv())) {
    return [
      {
        url: "https://example.com/model1.tar.gz",
        sha256: "abc123",
        size: 4200000,
        lang: "en",
        wakeword: "hey ember",
        description: "gigaspeech-en-3.3M (English model)",
      },
      {
        url: "https://example.com/model2.tar.gz",
        sha256: "def456",
        size: 4300000,
        lang: "es",
        wakeword: "hola ember",
        description: "spanish-model (Spanish model)",
      },
    ];
  }
  return tauriInvoke<KwsModelEntry[]>("kws_list_models");
}

export async function kwsDownloadModel(modelId: string): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Would download model: ${modelId}`;
  }
  return tauriInvoke<string>("kws_download_model", { modelId });
}

export async function kwsEnable(modelId: string): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Would enable KWS with model: ${modelId}`;
  }
  return tauriInvoke<string>("kws_enable", { modelId });
}

export async function kwsDisable(): Promise<string> {
  if (!(await isTauriEnv())) {
    return "[Web] Would disable KWS (return to stub)";
  }
  return tauriInvoke<string>("kws_disable");
}

export async function kwsArmTestWindow(durationMs: number): Promise<string> {
  if (!(await isTauriEnv())) {
    return `[Web] Would arm test window for ${durationMs}ms`;
  }
  return tauriInvoke<string>("kws_arm_test_window", { durationMs });
}

export async function isPipewireLoopback(): Promise<boolean> {
  if (!(await isTauriEnv())) {
    return false; // Web mode has no loopback
  }
  return tauriInvoke<boolean>("is_pipewire_loopback");
}

// ===== KWS EVENT SUBSCRIPTIONS =====

export interface KwsDownloadProgress {
  model_id: string;
  downloaded: number;
  total: number;
  percent: number;
}

export interface KwsTestPassPayload {
  model_id: string;
  keyword: string;
  ts: number;
}

export interface KwsEventHandlers {
  onDownloadProgress?: (progress: KwsDownloadProgress) => void;
  onModelVerified?: (modelId: string) => void;
  onModelVerifyFailed?: (modelId: string) => void;
  onEnabled?: (modelId: string) => void;
  onDisabled?: () => void;
  onWakeTestPass?: (payload: KwsTestPassPayload) => void;
}

let kwsSubscribed = false;

/**
 * Subscribe to KWS events
 * Should be called once when KWS UI is mounted
 */
export async function subscribeKwsEvents(handlers: KwsEventHandlers) {
  if (kwsSubscribed || !(await isTauriEnv())) return;

  const { listen } = await import("@tauri-apps/api/event");

  if (handlers.onDownloadProgress) {
    await listen<KwsDownloadProgress>("kws:model_download_progress", (e) => {
      handlers.onDownloadProgress?.(e.payload);
    });
  }

  if (handlers.onModelVerified) {
    await listen<string>("kws:model_verified", (e) => {
      handlers.onModelVerified?.(e.payload);
    });
  }

  if (handlers.onModelVerifyFailed) {
    await listen<string>("kws:model_verify_failed", (e) => {
      handlers.onModelVerifyFailed?.(e.payload);
    });
  }

  if (handlers.onEnabled) {
    await listen<string>("kws:enabled", (e) => {
      handlers.onEnabled?.(e.payload);
    });
  }

  if (handlers.onDisabled) {
    await listen("kws:disabled", () => {
      handlers.onDisabled?.();
    });
  }

  if (handlers.onWakeTestPass) {
    await listen<KwsTestPassPayload>("kws:wake_test_pass", (e) => {
      handlers.onWakeTestPass?.(e.payload);
    });
  }

  kwsSubscribed = true;
}
