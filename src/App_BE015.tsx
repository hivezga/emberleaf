/**
 * Emberleaf App - BE-015 "Simple First" UI
 * Features: Simple/Advanced mode toggle, i18n, device picker
 */

import { useState, useEffect } from "react";
import { Toaster } from "sonner";
import { Settings } from "lucide-react";
import { I18nProvider, useI18n, type Language } from "./lib/i18n";
import { SimpleAudioHome } from "./components/SimpleAudioHome";
import { DevicePicker } from "./components/DevicePicker";
import { DeviceLossBanner } from "./components/DeviceLossBanner";
import { AudioSettings } from "./components/AudioSettings";
import { Switch } from "./components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./components/ui/select";

const ADVANCED_MODE_KEY = "emberleaf_advanced_mode";

function AppContent() {
  const { t, language, setLanguage } = useI18n();
  const [advancedMode, setAdvancedModeState] = useState(() => {
    if (typeof window === "undefined") return false;
    return localStorage.getItem(ADVANCED_MODE_KEY) === "true";
  });
  const [devicePickerOpen, setDevicePickerOpen] = useState(false);
  const [suggestedDevice, setSuggestedDevice] = useState<string | null>(null);

  // Persist advanced mode
  const setAdvancedMode = (enabled: boolean) => {
    setAdvancedModeState(enabled);
    if (typeof window !== "undefined") {
      localStorage.setItem(ADVANCED_MODE_KEY, enabled.toString());
    }
  };

  // Listen for auto-probe suggestions
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function setupListener() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<{ device: string; reason: string }>(
          "audio:auto_probe_suggestion",
          (event) => {
            setSuggestedDevice(event.payload.device);
          }
        );
      } catch (err) {
        console.log("Event listener not available (web mode)");
      }
    }

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return (
    <div className="min-h-screen bg-background text-foreground p-4">
      <Toaster richColors position="top-right" />
      <DeviceLossBanner onOpenDevicePicker={() => setDevicePickerOpen(true)} />

      {/* Header with Advanced Mode Toggle */}
      <header className="max-w-4xl mx-auto mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Emberleaf</h1>
          <p className="text-sm text-muted-foreground">
            Private Voice Assistant
          </p>
        </div>

        <div className="flex items-center gap-4">
          {/* Language Switcher (in Advanced mode) */}
          {advancedMode && (
            <Select value={language} onValueChange={(val) => setLanguage(val as Language)}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
                <SelectItem value="es">Español</SelectItem>
              </SelectContent>
            </Select>
          )}

          {/* Advanced Mode Toggle */}
          <div className="flex items-center gap-2">
            <Settings className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">{t.advanced}</span>
            <Switch checked={advancedMode} onCheckedChange={setAdvancedMode} />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-4xl mx-auto space-y-4">
        {!advancedMode ? (
          // Simple Mode
          <SimpleAudioHome onChangeMicrophone={() => setDevicePickerOpen(true)} />
        ) : (
          // Advanced Mode
          <div className="space-y-4">
            <SimpleAudioHome onChangeMicrophone={() => setDevicePickerOpen(true)} />

            <Card>
              <CardHeader>
                <CardTitle>{t.advanced}</CardTitle>
              </CardHeader>
              <CardContent>
                <AudioSettings />
              </CardContent>
            </Card>
          </div>
        )}
      </main>

      {/* Device Picker Drawer */}
      <DevicePicker
        open={devicePickerOpen}
        onOpenChange={setDevicePickerOpen}
        suggestedDevice={suggestedDevice}
      />
    </div>
  );
}

export default function App() {
  return (
    <I18nProvider>
      <AppContent />
    </I18nProvider>
  );
}
