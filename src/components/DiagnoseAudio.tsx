import { useState } from "react";
import { toast } from "sonner";
import { FileText, Copy, X } from "lucide-react";
import * as Dialog from "@radix-ui/react-dialog";
import { getAudioSnapshot, type AudioSnapshot } from "../lib/tauriSafe";
import { Button } from "./ui/button";
import { cn } from "../lib/utils";

export function DiagnoseAudioButton() {
  const [open, setOpen] = useState(false);
  const [snapshot, setSnapshot] = useState<AudioSnapshot | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleOpen() {
    setOpen(true);
    setLoading(true);
    try {
      const data = await getAudioSnapshot();
      setSnapshot(data);
    } catch (error) {
      toast.error(`Failed to get audio snapshot: ${error}`);
      setOpen(false);
    } finally {
      setLoading(false);
    }
  }

  async function handleCopy() {
    if (!snapshot) return;
    try {
      const json = JSON.stringify(snapshot, null, 2);
      await navigator.clipboard.writeText(json);
      toast.success("Snapshot copied to clipboard");
    } catch (error) {
      toast.error(`Failed to copy: ${error}`);
    }
  }

  function formatTimestamp(ms: number): string {
    return new Date(ms).toLocaleString();
  }

  function formatDuration(ms: number): string {
    if (ms === 0) return "Never";
    const seconds = Math.floor((Date.now() - ms) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ago`;
  }

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        <Button variant="outline" size="sm" onClick={handleOpen}>
          <FileText className="h-4 w-4" />
          Diagnose Audio
        </Button>
      </Dialog.Trigger>

      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
        <Dialog.Content
          className={cn(
            "fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50",
            "w-[90vw] max-w-3xl max-h-[85vh] overflow-hidden",
            "bg-card border rounded-lg shadow-lg",
            "flex flex-col"
          )}
        >
          <div className="flex items-center justify-between p-4 border-b">
            <Dialog.Title className="text-lg font-semibold">
              Audio Diagnostics Snapshot
            </Dialog.Title>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleCopy}
                disabled={!snapshot}
              >
                <Copy className="h-4 w-4" />
                Copy JSON
              </Button>
              <Dialog.Close asChild>
                <Button variant="ghost" size="icon">
                  <X className="h-4 w-4" />
                </Button>
              </Dialog.Close>
            </div>
          </div>

          <div className="flex-1 overflow-auto p-4 space-y-4">
            {loading && (
              <div className="text-center py-8 text-muted-foreground">
                Loading snapshot...
              </div>
            )}

            {snapshot && (
              <>
                <div className="space-y-2">
                  <h3 className="font-medium">Summary</h3>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>
                      <span className="text-muted-foreground">Timestamp:</span>{" "}
                      {formatTimestamp(snapshot.timestamp_ms)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Last Restart:</span>{" "}
                      {formatDuration(snapshot.last_restart_ms)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Monitor Active:</span>{" "}
                      {snapshot.monitor_active ? "Yes" : "No"}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Processing Rate:</span>{" "}
                      {snapshot.debug_info.processing_rate}Hz
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <h3 className="font-medium">Selected Devices</h3>
                  <div className="text-sm space-y-1">
                    <div>
                      <span className="text-muted-foreground">Input:</span>{" "}
                      {snapshot.selected_input_device || "(default)"}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Output:</span>{" "}
                      {snapshot.selected_output_device || "(default)"}
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <h3 className="font-medium">Available Input Devices ({snapshot.input_devices.length})</h3>
                  <ul className="text-sm space-y-1">
                    {snapshot.input_devices.map((dev, i) => (
                      <li key={i} className="text-muted-foreground">
                        • {dev.name} {dev.is_default && "(default)"}
                      </li>
                    ))}
                  </ul>
                </div>

                <div className="space-y-2">
                  <h3 className="font-medium">Available Output Devices ({snapshot.output_devices.length})</h3>
                  <ul className="text-sm space-y-1">
                    {snapshot.output_devices.map((dev, i) => (
                      <li key={i} className="text-muted-foreground">
                        • {dev.name} {dev.is_default && "(default)"}
                      </li>
                    ))}
                  </ul>
                </div>

                <div className="space-y-2">
                  <h3 className="font-medium">Full JSON</h3>
                  <pre className="text-xs bg-muted p-3 rounded overflow-auto max-h-64">
                    {JSON.stringify(snapshot, null, 2)}
                  </pre>
                </div>
              </>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
