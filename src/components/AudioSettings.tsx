import { useEffect, useState } from "react";
import { toast } from "sonner";
import { RefreshCw, Volume2, Mic, Settings } from "lucide-react";
import { DiagnoseAudioButton } from "./DiagnoseAudio";
import { KwsSettings } from "./KwsSettings";
import {
  listInputDevices,
  listOutputDevices,
  currentInputDevice,
  currentOutputDevice,
  setInputDevice,
  setOutputDevice,
  restartAudioCapture,
  playTestTone,
  startMicMonitor,
  stopMicMonitor,
  getConfig,
  setPersistMonitorState,
  type DeviceInfo,
} from "../lib/tauriSafe";
import { useI18n } from "../lib/i18n";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { Switch } from "./ui/switch";

export function AudioSettings() {
  const { t } = useI18n();
  const [inputDevices, setInputDevices] = useState<DeviceInfo[]>([]);
  const [outputDevices, setOutputDevices] = useState<DeviceInfo[]>([]);
  const [selectedInput, setSelectedInput] = useState<string | null>(null);
  const [selectedOutput, setSelectedOutput] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [needsRestart, setNeedsRestart] = useState(false);
  const [persistMonitorState, setPersistMonitorStateLocal] = useState(false);
  const [monitorGuarded, setMonitorGuarded] = useState(false);

  // Load devices and config on mount
  useEffect(() => {
    loadDevices();
    loadConfig();
  }, []);

  async function loadConfig() {
    try {
      const config = await getConfig();
      setPersistMonitorStateLocal(config.ui.persist_monitor_state);
    } catch (err) {
      console.error("Failed to load config:", err);
    }
  }

  // Listen for monitor_guarded event
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function setupListener() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<{ reason: string }>("audio:monitor_guarded", (event) => {
          console.log("Monitor guarded:", event.payload.reason);
          setMonitorGuarded(true);

          // Auto-clear after 8 seconds
          setTimeout(() => {
            setMonitorGuarded(false);
          }, 8000);
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

  async function loadDevices() {
    setIsLoading(true);
    try {
      const [inputs, outputs, currentIn, currentOut] = await Promise.all([
        listInputDevices(),
        listOutputDevices(),
        currentInputDevice(),
        currentOutputDevice(),
      ]);

      setInputDevices(inputs);
      setOutputDevices(outputs);
      setSelectedInput(currentIn);
      setSelectedOutput(currentOut);
    } catch (error) {
      toast.error(`Failed to load devices: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleInputChange(deviceName: string) {
    try {
      const result = await setInputDevice(deviceName, true);
      setSelectedInput(deviceName);
      setNeedsRestart(true);
      toast.success(result);
    } catch (error) {
      toast.error(`Failed to set input device: ${error}`);
    }
  }

  async function handleOutputChange(deviceName: string) {
    try {
      const result = await setOutputDevice(deviceName, true);
      setSelectedOutput(deviceName);
      toast.success(result);

      // Auto-play test tone on output device change
      setTimeout(() => {
        playTestTone({ deviceName, frequencyHz: 660, durationMs: 250, volume: 0.15 })
          .then((msg) => toast.info(msg))
          .catch((err) => toast.error(`Test tone failed: ${err}`));
      }, 100);
    } catch (error) {
      toast.error(`Failed to set output device: ${error}`);
    }
  }

  async function handleRestart() {
    setIsLoading(true);
    try {
      const result = await restartAudioCapture();
      setNeedsRestart(false);
      toast.success(result);
    } catch (error) {
      toast.error(`Failed to restart audio: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleTestTone() {
    try {
      const result = await playTestTone({
        deviceName: selectedOutput ?? undefined,
        frequencyHz: 880,
        durationMs: 400,
        volume: 0.2,
      });
      toast.success(result);
    } catch (error) {
      toast.error(`Test tone failed: ${error}`);
    }
  }

  async function handleMonitorToggle(checked: boolean) {
    try {
      if (checked) {
        const result = await startMicMonitor(0.15);
        setIsMonitoring(true);
        toast.success(result);
      } else {
        const result = await stopMicMonitor();
        setIsMonitoring(false);
        toast.info(result);
      }
    } catch (error) {
      toast.error(`Monitor toggle failed: ${error}`);
      setIsMonitoring(false);
    }
  }

  async function handlePersistMonitorToggle(checked: boolean) {
    try {
      const result = await setPersistMonitorState(checked);
      setPersistMonitorStateLocal(checked);
      toast.info(result);
    } catch (error) {
      toast.error(`Failed to update persistence setting: ${error}`);
    }
  }

  return (
    <div className="space-y-4">
      {/* KWS Settings */}
      <KwsSettings />

      {/* Audio Settings */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Settings className="h-5 w-5" />
                Audio Settings
              </CardTitle>
              <CardDescription>
                Configure input/output devices and test audio pipeline
              </CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <DiagnoseAudioButton />
              <Button
                variant="outline"
                size="sm"
                onClick={loadDevices}
                disabled={isLoading}
              >
                <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Input Device Section */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium flex items-center gap-2">
                <Mic className="h-4 w-4" />
                Input Device
              </label>
              {needsRestart && (
                <Badge variant="warning" className="text-xs">
                  Requires restart
                </Badge>
              )}
            </div>
            <Select
              value={selectedInput ?? undefined}
              onValueChange={handleInputChange}
              disabled={isLoading}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select input device..." />
              </SelectTrigger>
              <SelectContent>
                {inputDevices.map((device) => (
                  <SelectItem key={device.name} value={device.name}>
                    {device.name} {device.is_default && "(Default)"}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Output Device Section */}
          <div className="space-y-2">
            <label className="text-sm font-medium flex items-center gap-2">
              <Volume2 className="h-4 w-4" />
              Output Device
            </label>
            <Select
              value={selectedOutput ?? undefined}
              onValueChange={handleOutputChange}
              disabled={isLoading}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select output device..." />
              </SelectTrigger>
              <SelectContent>
                {outputDevices.map((device) => (
                  <SelectItem key={device.name} value={device.name}>
                    {device.name} {device.is_default && "(Default)"}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Actions */}
          <div className="flex flex-wrap gap-2 pt-2">
            {needsRestart && (
              <Button
                onClick={handleRestart}
                disabled={isLoading}
                variant="default"
              >
                <RefreshCw className="h-4 w-4" />
                Restart Audio
              </Button>
            )}
            <Button
              onClick={handleTestTone}
              disabled={isLoading}
              variant="outline"
            >
              <Volume2 className="h-4 w-4" />
              Test Tone
            </Button>
          </div>

          {/* Mic Monitor Toggle */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="space-y-0.5">
              <div className="text-sm font-medium">Microphone Monitor</div>
              <div className="text-xs text-muted-foreground">
                Route input to output for debugging (low gain)
              </div>
            </div>
            <Switch
              checked={isMonitoring}
              onCheckedChange={handleMonitorToggle}
              disabled={isLoading}
            />
          </div>

          {/* Remember Monitor State Toggle */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="space-y-0.5">
              <div className="text-sm font-medium">
                {t.settings?.rememberMonitor || "Remember mic monitoring"}
              </div>
              <div className="text-xs text-muted-foreground">
                Auto-resume monitor state on app restart
              </div>
              {monitorGuarded && (
                <div className="text-xs text-blue-600 dark:text-blue-400 mt-1 flex items-center gap-1">
                  <span>ℹ</span>
                  <span>{t.deviceLoss?.monitorGuarded || "Not resumed to avoid feedback."}</span>
                </div>
              )}
            </div>
            <Switch
              checked={persistMonitorState}
              onCheckedChange={handlePersistMonitorToggle}
              disabled={isLoading}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
