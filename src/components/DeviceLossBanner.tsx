/**
 * Device Loss Banner - Shows notifications when audio devices are lost/recovered
 * Handles device loss, fallback success/failure, and monitor guarded events
 */

import { useEffect, useState } from "react";
import { AlertCircle, CheckCircle, Info, X } from "lucide-react";
import { Button } from "./ui/button";
import { useI18n } from "../lib/i18n";
import { isTauriEnv, restartAudioCaptureV2, type DeviceId } from "../lib/tauriSafe";

type BannerState =
  | { type: "none" }
  | { type: "fallback_ok"; deviceName: string }
  | { type: "fallback_failed"; deviceName: string; reason: string }
  | { type: "monitor_guarded" };

export function DeviceLossBanner({ onOpenDevicePicker }: { onOpenDevicePicker: () => void }) {
  const { t } = useI18n();
  const [bannerState, setBannerState] = useState<BannerState>({ type: "none" });
  const [retrying, setRetrying] = useState(false);

  useEffect(() => {
    let unlisten: (() => void)[] = [];

    async function setupListeners() {
      if (!(await isTauriEnv())) return;

      try {
        const { listen } = await import("@tauri-apps/api/event");

        // Device lost event
        const unlistenLost = await listen<{ kind: string; previous: DeviceId }>(
          "audio:device_lost",
          (event) => {
            const { previous } = event.payload;
            console.log(`Device lost: ${previous.name}`);
            // Don't show banner yet, wait for fallback result
          }
        );
        unlisten.push(unlistenLost);

        // Fallback success
        const unlistenFallbackOk = await listen<{ kind: string; new_device: string }>(
          "audio:device_fallback_ok",
          (_event) => {
            console.log("Fallback OK");
            setBannerState({ type: "fallback_ok", deviceName: "default" });

            // Auto-dismiss after 6 seconds
            setTimeout(() => {
              setBannerState((prev) =>
                prev.type === "fallback_ok" ? { type: "none" } : prev
              );
            }, 6000);
          }
        );
        unlisten.push(unlistenFallbackOk);

        // Fallback failed
        const unlistenFallbackFailed = await listen<{ kind: string; reason: string }>(
          "audio:device_fallback_failed",
          (event) => {
            console.log("Fallback failed:", event.payload.reason);
            setBannerState({
              type: "fallback_failed",
              deviceName: "microphone",
              reason: event.payload.reason,
            });
          }
        );
        unlisten.push(unlistenFallbackFailed);

        // Monitor guarded
        const unlistenGuarded = await listen<{ reason: string }>(
          "audio:monitor_guarded",
          (event) => {
            console.log("Monitor guarded:", event.payload.reason);
            setBannerState({ type: "monitor_guarded" });

            // Auto-dismiss after 8 seconds
            setTimeout(() => {
              setBannerState((prev) =>
                prev.type === "monitor_guarded" ? { type: "none" } : prev
              );
            }, 8000);
          }
        );
        unlisten.push(unlistenGuarded);
      } catch (err) {
        console.log("Event listeners not available (web mode)");
      }
    }

    setupListeners();

    return () => {
      unlisten.forEach((fn) => fn());
    };
  }, []);

  async function handleRetry() {
    setRetrying(true);
    try {
      await restartAudioCaptureV2();
      setBannerState({ type: "none" });
    } catch (err) {
      console.error("Retry failed:", err);
    } finally {
      setRetrying(false);
    }
  }

  function handleDismiss() {
    setBannerState({ type: "none" });
  }

  if (bannerState.type === "none") return null;

  return (
    <div role="status" aria-live="polite" className="fixed top-4 left-1/2 -translate-x-1/2 z-50 w-full max-w-2xl px-4">
      {bannerState.type === "fallback_ok" && (
        <div className="bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800 rounded-lg p-4 shadow-lg">
          <div className="flex items-start gap-3">
            <CheckCircle className="h-5 w-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-green-900 dark:text-green-100">
                {t.deviceLoss?.fallbackOk || "Switched to the default device."}
              </p>
            </div>
            <div className="flex items-center gap-2 flex-shrink-0">
              <Button variant="ghost" size="sm" onClick={onOpenDevicePicker}>
                {t.deviceLoss?.change || "Change…"}
              </Button>
              <button
                onClick={handleDismiss}
                className="text-green-600 hover:text-green-800 dark:text-green-400 dark:hover:text-green-200"
                aria-label="Dismiss"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      )}

      {bannerState.type === "fallback_failed" && (
        <div className="bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded-lg p-4 shadow-lg">
          <div className="flex items-start gap-3">
            <AlertCircle className="h-5 w-5 text-amber-600 dark:text-amber-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-amber-900 dark:text-amber-100">
                {t.deviceLoss?.fallbackFailed || "We couldn't switch automatically."}
              </p>
              {bannerState.reason && (
                <p className="text-xs text-amber-700 dark:text-amber-300 mt-1">
                  {bannerState.reason}
                </p>
              )}
            </div>
            <div className="flex items-center gap-2 flex-shrink-0">
              <Button
                variant="outline"
                size="sm"
                onClick={handleRetry}
                disabled={retrying}
              >
                {retrying
                  ? (t.loading || "Loading...")
                  : (t.deviceLoss?.retry || "Retry")}
              </Button>
              <Button variant="ghost" size="sm" onClick={onOpenDevicePicker}>
                {t.deviceLoss?.change || "Change…"}
              </Button>
              <button
                onClick={handleDismiss}
                className="text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200"
                aria-label="Dismiss"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      )}

      {bannerState.type === "monitor_guarded" && (
        <div className="bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800 rounded-lg p-4 shadow-lg">
          <div className="flex items-start gap-3">
            <Info className="h-5 w-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm text-blue-900 dark:text-blue-100">
                {t.deviceLoss?.monitorGuarded || "Monitoring wasn't resumed to avoid feedback."}
              </p>
            </div>
            <button
              onClick={handleDismiss}
              className="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200 flex-shrink-0"
              aria-label="Dismiss"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
