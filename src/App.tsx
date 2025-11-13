/**
 * Emberleaf App - BE-015 "Simple First" UI
 * Features: Simple/Advanced mode toggle, i18n, device picker
 */

import { useState, useEffect } from "react";
import { Toaster } from "sonner";
import { Settings, CheckCircle2, RotateCcw } from "lucide-react";
import { I18nProvider, useI18n, type Language } from "./lib/i18n";
import { SimpleAudioHome } from "./components/SimpleAudioHome";
import { DevicePicker } from "./components/DevicePicker";
import { AudioSettings } from "./components/AudioSettings";
import { PreflightDialog } from "./components/preflight/PreflightDialog";
import { OnboardingWizard } from "./components/onboarding/OnboardingWizard";
import { Switch } from "./components/ui/switch";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./components/ui/select";
import { subscribeGlobalErrors } from "./lib/tauriSafe";

const ADVANCED_MODE_KEY = "emberleaf_advanced_mode";
const PREFLIGHT_COMPLETED_KEY = "emberleaf_preflight_completed";
const ONBOARDING_COMPLETED_KEY = "emberleaf_onboarding_completed";

function AppContent() {
  const { t, language, setLanguage } = useI18n();
  const [advancedMode, setAdvancedModeState] = useState(() => {
    if (typeof window === "undefined") return false;
    return localStorage.getItem(ADVANCED_MODE_KEY) === "true";
  });
  const [devicePickerOpen, setDevicePickerOpen] = useState(false);
  const [suggestedDevice, setSuggestedDevice] = useState<string | null>(null);

  // Onboarding runs BEFORE preflight on first run
  const [onboardingOpen, setOnboardingOpen] = useState(() => {
    if (typeof window === "undefined") return false;
    return localStorage.getItem(ONBOARDING_COMPLETED_KEY) !== "true";
  });

  // Preflight only shows if onboarding is complete
  const [preflightOpen, setPreflightOpen] = useState(false);

  // Persist advanced mode
  const setAdvancedMode = (enabled: boolean) => {
    setAdvancedModeState(enabled);
    if (typeof window !== "undefined") {
      localStorage.setItem(ADVANCED_MODE_KEY, enabled.toString());
    }
  };

  // Handle onboarding completion
  const handleOnboardingComplete = () => {
    if (typeof window !== "undefined") {
      localStorage.setItem(ONBOARDING_COMPLETED_KEY, "true");
      localStorage.setItem(PREFLIGHT_COMPLETED_KEY, "true"); // Preflight runs inside onboarding
    }
    setOnboardingOpen(false);
  };

  // Handle preflight completion
  const handlePreflightComplete = () => {
    if (typeof window !== "undefined") {
      localStorage.setItem(PREFLIGHT_COMPLETED_KEY, "true");
    }
  };

  // Re-run onboarding (from Advanced settings)
  const handleRerunOnboarding = () => {
    setOnboardingOpen(true);
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

  // Subscribe to global validation errors (SEC-001B)
  useEffect(() => {
    subscribeGlobalErrors();
  }, []);

  return (
    <div className="min-h-screen bg-background text-foreground p-4">
      <Toaster richColors position="top-right" />

      {/* Header with Advanced Mode Toggle */}
      <header className="max-w-4xl mx-auto mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Emberleaf</h1>
          <p className="text-sm text-muted-foreground">
            Private Voice Assistant
          </p>
        </div>

        <div className="flex items-center gap-4">
          {/* Onboarding Re-run Button (in Advanced mode) */}
          {advancedMode && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleRerunOnboarding}
            >
              <RotateCcw className="h-4 w-4 mr-2" />
              {t.onboarding.rerunOnboarding}
            </Button>
          )}

          {/* Preflight Check Button (in Advanced mode) */}
          {advancedMode && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPreflightOpen(true)}
            >
              <CheckCircle2 className="h-4 w-4 mr-2" />
              {t.preflight.checkMySetup}
            </Button>
          )}

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

      {/* Preflight Dialog */}
      <PreflightDialog
        open={preflightOpen}
        onOpenChange={setPreflightOpen}
        onComplete={handlePreflightComplete}
      />

      {/* Onboarding Wizard (First Run) */}
      <OnboardingWizard open={onboardingOpen} onComplete={handleOnboardingComplete} />
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
