/**
 * Simple Audio Home - User-friendly audio setup interface
 * Designed for non-technical users with 1-click microphone testing
 */

import { useEffect, useState, useRef } from "react";
import { toast } from "sonner";
import { Mic, Volume2, AlertCircle, CheckCircle, AlertTriangle } from "lucide-react";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "./ui/card";
import { useI18n, format } from "../lib/i18n";
import {
  suggestInputDevice,
  playTestTone,
  currentInputDevice,
  type ProbeResult,
} from "../lib/tauriSafe";

interface MicMeterProps {
  level: number;
  label: string;
  showClippingHint: boolean;
}

function MicMeter({ level, label, showClippingHint }: MicMeterProps) {
  return (
    <div className="space-y-2" role="meter" aria-valuenow={level} aria-valuemin={0} aria-valuemax={1} aria-label={label}>
      <div className="flex items-center justify-between">
        <div className="text-sm text-muted-foreground">{label}</div>
        {showClippingHint && (
          <div className="flex items-center gap-1 text-xs text-amber-600">
            <AlertTriangle className="h-3 w-3" />
            <span>Too loud</span>
          </div>
        )}
      </div>
      <div className="h-12 bg-muted rounded-lg overflow-hidden border">
        <div
          className="h-full bg-gradient-to-r from-green-500 via-yellow-500 to-red-500 transition-all duration-100"
          style={{ width: `${level * 100}%` }}
        />
      </div>
    </div>
  );
}

export function SimpleAudioHome({ onChangeMicrophone }: { onChangeMicrophone: () => void }) {
  const { t } = useI18n();
  const [_micLevel, setMicLevel] = useState(0); // Raw level (unused, smoothed version is displayed)
  const [smoothedMicLevel, setSmoothedMicLevel] = useState(0);
  const [showClippingHint, setShowClippingHint] = useState(false);
  const [currentDevice, setCurrentDevice] = useState<string | null>(null);
  const [probing, setProbing] = useState(false);
  const [lastProbe, setLastProbe] = useState<ProbeResult | null>(null);

  // Smoothing parameters
  const smoothingAlpha = 0.3; // EMA smoothing factor
  const clippingThreshold = 0.6; // RMS threshold for "too loud"
  const clippingDuration = 300; // milliseconds
  const clippingStartTime = useRef<number | null>(null);

  // Listen to RMS events with smoothing
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function setupListener() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<number>("audio:rms", (event) => {
          const rawLevel = event.payload;
          setMicLevel(rawLevel);

          // Apply EMA smoothing: smoothed = alpha * raw + (1 - alpha) * previous
          setSmoothedMicLevel((prev) => smoothingAlpha * rawLevel + (1 - smoothingAlpha) * prev);
        });
      } catch (err) {
        console.log("Event listener not available (web mode)");
      }
    }

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // Clipping detection: show hint if smoothed level > threshold for > 300ms
  useEffect(() => {
    if (smoothedMicLevel > clippingThreshold) {
      if (clippingStartTime.current === null) {
        clippingStartTime.current = Date.now();
      } else {
        const elapsed = Date.now() - clippingStartTime.current;
        if (elapsed > clippingDuration) {
          setShowClippingHint(true);
        }
      }
    } else {
      clippingStartTime.current = null;
      setShowClippingHint(false);
    }
  }, [smoothedMicLevel, clippingThreshold, clippingDuration]);

  // Load current device on mount
  useEffect(() => {
    loadCurrentDevice();
  }, []);

  async function loadCurrentDevice() {
    try {
      const device = await currentInputDevice();
      setCurrentDevice(device);
    } catch (err) {
      console.error("Failed to load current device:", err);
    }
  }

  async function handleAutoProbe() {
    setProbing(true);
    try {
      const result = await suggestInputDevice();
      setLastProbe(result);

      if (result.suggested) {
        toast.success(result.reason);
      } else {
        toast.warning(result.reason);
      }
    } catch (err) {
      toast.error(`${t.error}: ${err}`);
    } finally {
      setProbing(false);
    }
  }

  async function handleTestSound() {
    try {
      const result = await playTestTone({
        deviceName: null, // Use default output
        frequencyHz: 660,
        durationMs: 250,
        volume: 0.15,
      });
      toast.info(result);
    } catch (err) {
      toast.error(`${t.error}: ${err}`);
    }
  }

  // Determine status (use smoothed level for stability)
  const isGood = smoothedMicLevel > 0.02; // Threshold for "activity"
  const deviceName = currentDevice || "default";

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Mic className="h-5 w-5" />
          {t.audioReady}
        </CardTitle>
        <CardDescription>
          {isGood
            ? format(t.micDetected, { device: deviceName })
            : t.micNotHeard}
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-6">
        {/* Mic Meter - using smoothed level for stable visualization */}
        <MicMeter level={smoothedMicLevel} label={t.speakNow} showClippingHint={showClippingHint} />

        {/* Status Badge */}
        <div className="flex items-center gap-2">
          {isGood ? (
            <>
              <CheckCircle className="h-5 w-5 text-green-600" />
              <span className="text-sm font-medium text-green-700">{t.good}</span>
            </>
          ) : (
            <>
              <AlertCircle className="h-5 w-5 text-amber-600" />
              <span className="text-sm text-amber-700">{t.checkMicOrChange}</span>
            </>
          )}
        </div>

        {/* Primary CTA: Auto-probe */}
        <Button
          onClick={handleAutoProbe}
          disabled={probing}
          className="w-full"
          size="lg"
        >
          <Mic className="h-4 w-4 mr-2" />
          {probing ? t.loading : t.checkMicrophone}
        </Button>

        {/* Last probe result */}
        {lastProbe && lastProbe.suggested && (
          <div className="p-3 bg-muted rounded-lg text-sm">
            <div className="font-medium">{t.suggested}:</div>
            <div className="text-muted-foreground">{lastProbe.suggested}</div>
            <div className="text-xs text-muted-foreground mt-1">{lastProbe.reason}</div>
          </div>
        )}

        {/* Secondary Actions */}
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={onChangeMicrophone}>
            <Mic className="h-4 w-4 mr-2" />
            {t.changeMicrophone}
          </Button>
          <Button variant="outline" onClick={handleTestSound}>
            <Volume2 className="h-4 w-4 mr-2" />
            {t.playTestSound}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
