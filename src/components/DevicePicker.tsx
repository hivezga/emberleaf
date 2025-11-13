/**
 * Device Picker Drawer - Simple microphone selection UI
 * Features grouped lists (Suggested, Other) with inline feedback
 */

import { useState, useEffect } from "react";
import { toast } from "sonner";
import { Mic, Check, Info } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "./ui/sheet";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { useI18n, format } from "../lib/i18n";
import {
  listInputDevices,
  currentInputDevice,
  setInputDevice,
  restartAudioCaptureV2,
  type DeviceInfo,
} from "../lib/tauriSafe";

interface DevicePickerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  suggestedDevice?: string | null;
}

export function DevicePicker({ open, onOpenChange, suggestedDevice }: DevicePickerProps) {
  const { t } = useI18n();
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [currentDevice, setCurrentDevice] = useState<string | null>(null);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
  const [applying, setApplying] = useState(false);

  useEffect(() => {
    if (open) {
      loadDevices();
    }
  }, [open]);

  async function loadDevices() {
    try {
      const [deviceList, current] = await Promise.all([
        listInputDevices(),
        currentInputDevice(),
      ]);
      setDevices(deviceList);
      setCurrentDevice(current);
      setSelectedDevice(current);
    } catch (err) {
      toast.error(`Failed to load devices: ${err}`);
    }
  }

  async function handleApply() {
    if (!selectedDevice) return;

    setApplying(true);
    try {
      // 1. Set the device (persist)
      await setInputDevice(selectedDevice, true);

      // 2. Restart audio capture
      const result = await restartAudioCaptureV2();

      if (result.ok) {
        toast.success(result.message);
        setCurrentDevice(selectedDevice);
        onOpenChange(false);
      } else {
        toast.error(result.message);
      }
    } catch (err) {
      toast.error(`Failed to apply: ${err}`);
    } finally {
      setApplying(false);
    }
  }

  async function handleSaveForLater() {
    if (!selectedDevice) return;

    try {
      await setInputDevice(selectedDevice, true);
      toast.info(t.saveForLater);
      onOpenChange(false);
    } catch (err) {
      toast.error(`Failed to save: ${err}`);
    }
  }

  // Group devices
  const suggested = devices.filter((d) => d.name === suggestedDevice);
  const others = devices.filter((d) => d.name !== suggestedDevice);

  const hasChanged = selectedDevice !== currentDevice;

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent>
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Mic className="h-5 w-5" />
            {t.changeMicrophone}
          </SheetTitle>
          <SheetDescription>{t.quicklyReconnect}</SheetDescription>
        </SheetHeader>

        <div className="space-y-6 mt-6">
          {/* Suggested Devices */}
          {suggested.length > 0 && (
            <div>
              <h3 className="text-sm font-medium mb-2">{t.suggested}</h3>
              <div className="space-y-2">
                {suggested.map((device) => (
                  <DeviceItem
                    key={device.name}
                    device={device}
                    selected={selectedDevice === device.name}
                    current={currentDevice === device.name}
                    onSelect={() => setSelectedDevice(device.name)}
                    badges={
                      <>
                        {device.is_default && <Badge variant="secondary">{t.defaultDevice}</Badge>}
                        <Badge variant="outline">{t.recentlyActive}</Badge>
                      </>
                    }
                  />
                ))}
              </div>
            </div>
          )}

          {/* Other Devices */}
          {others.length > 0 && (
            <div>
              <h3 className="text-sm font-medium mb-2">{t.otherMicrophones}</h3>
              <div className="space-y-2">
                {others.map((device) => (
                  <DeviceItem
                    key={device.name}
                    device={device}
                    selected={selectedDevice === device.name}
                    current={currentDevice === device.name}
                    onSelect={() => setSelectedDevice(device.name)}
                    badges={
                      device.is_default ? (
                        <Badge variant="secondary">{t.defaultDevice}</Badge>
                      ) : null
                    }
                  />
                ))}
              </div>
            </div>
          )}

          {/* Inline Feedback */}
          {hasChanged && (
            <div className="p-3 bg-muted rounded-lg text-sm">
              <div className="font-medium">{t.applyNow}</div>
              <div className="text-muted-foreground text-xs">{t.quicklyReconnect}</div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2">
            <Button onClick={handleApply} disabled={!hasChanged || applying} className="flex-1">
              {applying ? t.loading : t.apply}
            </Button>
            <Button
              variant="outline"
              onClick={handleSaveForLater}
              disabled={!hasChanged || applying}
              className="flex-1"
            >
              {t.saveForLater}
            </Button>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}

interface DeviceItemProps {
  device: DeviceInfo;
  selected: boolean;
  current: boolean;
  onSelect: () => void;
  badges?: React.ReactNode;
}

function DeviceItem({ device, selected, current, onSelect, badges }: DeviceItemProps) {
  const { t } = useI18n();

  const stableIdText = device.stable_id
    ? format(t.deviceLoss?.stableId || "Stable ID: {id}", {
        id: `${device.stable_id.host_api}#${device.stable_id.index}`,
      })
    : null;

  return (
    <button
      onClick={onSelect}
      className={cn(
        "w-full p-3 rounded-lg border text-left transition-colors",
        "hover:bg-accent hover:border-accent-foreground/20",
        "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        selected && "border-primary bg-primary/5"
      )}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            {selected && <Check className="h-4 w-4 text-primary flex-shrink-0" />}
            <span className="text-sm font-medium truncate">{device.name}</span>
            {device.stable_id && (
              <div
                className="group relative"
                role="tooltip"
                aria-label={stableIdText || undefined}
              >
                <Info className="h-3 w-3 text-muted-foreground cursor-help" />
                <div className="invisible group-hover:visible absolute left-0 top-full mt-1 z-10 w-max max-w-xs bg-popover text-popover-foreground border rounded-md px-2 py-1 text-xs shadow-md">
                  {stableIdText}
                </div>
              </div>
            )}
          </div>
          {current && (
            <div className="text-xs text-muted-foreground mt-1">Currently active</div>
          )}
        </div>
        {badges && <div className="flex gap-1 ml-2 flex-shrink-0">{badges}</div>}
      </div>
    </button>
  );
}

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(" ");
}
