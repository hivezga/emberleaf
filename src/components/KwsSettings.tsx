import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Brain, AlertCircle, CheckCircle2, Play } from "lucide-react";
import {
  kwsStatus,
  kwsListModels,
  kwsEnable,
  kwsDisable,
  subscribeKwsEvents,
  kwsArmTestWindow,
  isPipewireLoopback,
  type KwsStatus as KwsStatusType,
  type KwsModelEntry,
  type KwsDownloadProgress,
  type KwsTestPassPayload,
} from "../lib/tauriSafe";
import { useI18n, format } from "../lib/i18n";
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

export function KwsSettings() {
  const { t } = useI18n();
  const [status, setStatus] = useState<KwsStatusType | null>(null);
  const [models, setModels] = useState<KwsModelEntry[]>([]);
  const [selectedModelId, setSelectedModelId] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<KwsDownloadProgress | null>(null);
  const [verificationState, setVerificationState] = useState<"idle" | "verifying" | "verified" | "failed">("idle");
  const [isEnabling, setIsEnabling] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<"idle" | "pass" | "fail">("idle");
  const [countdown, setCountdown] = useState<number | null>(null);

  // Load initial status and models
  useEffect(() => {
    loadStatus();
    loadModels();
    setupEventListeners();
  }, []);

  async function loadStatus() {
    try {
      const s = await kwsStatus();
      setStatus(s);
      if (s.model_id) {
        setSelectedModelId(s.model_id);
      }
    } catch (error) {
      console.error("Failed to load KWS status:", error);
    }
  }

  async function loadModels() {
    setIsLoading(true);
    try {
      const m = await kwsListModels();
      setModels(m);

      // Auto-select first model if none selected
      if (m.length > 0 && !selectedModelId) {
        // Prefer Spanish model if available, otherwise first model
        const spanishModel = m.find(model => model.lang === "es");
        const modelToSelect = spanishModel || m[0];
        const modelId = extractModelId(modelToSelect.description);
        setSelectedModelId(modelId);
      }
    } catch (error) {
      toast.error(`Failed to load models: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  // Extract model ID from description (format: "id (description)")
  function extractModelId(description: string): string {
    const match = description.match(/^([^\s(]+)/);
    return match ? match[1] : description;
  }

  // Get formatted model name for display
  function getModelDisplayName(entry: KwsModelEntry): string {
    return entry.description;
  }

  async function setupEventListeners() {
    try {
      await subscribeKwsEvents({
        onDownloadProgress: (progress) => {
          setDownloadProgress(progress);
          setVerificationState("idle");
        },
        onModelVerified: (_modelId) => {
          setDownloadProgress(null);
          setVerificationState("verified");
          toast.success(t.kws.verified);
          setTimeout(() => setVerificationState("idle"), 3000);
        },
        onModelVerifyFailed: (_modelId) => {
          setDownloadProgress(null);
          setVerificationState("failed");
          toast.error(t.kws.verifyFailed);
          setIsEnabling(false);
        },
        onEnabled: (_modelId) => {
          toast.success(t.kws.enabledToast);
          setIsEnabling(false);
          loadStatus(); // Refresh status
        },
        onDisabled: () => {
          toast.info(t.kws.disabledToast);
          loadStatus(); // Refresh status
        },
        onWakeTestPass: (_payload: KwsTestPassPayload) => {
          setTestResult("pass");
          setIsTesting(false);
          setCountdown(null);
          toast.success(t.kws.testPass);
        },
      });
    } catch (error) {
      console.log("Event listeners not available (web mode)");
    }
  }

  async function handleTest() {
    setIsTesting(true);
    setTestResult("idle");
    setCountdown(null);

    try {
      // Check if loopback is available
      const hasLoopback = await isPipewireLoopback();

      if (!hasLoopback) {
        // No loopback - use countdown for user speech
        toast.info(t.kws.testHintSayNow);

        // 5-second countdown
        for (let i = 5; i > 0; i--) {
          setCountdown(i);
          await new Promise(resolve => setTimeout(resolve, 1000));
        }
        setCountdown(null);
      }

      // Arm test window for 8 seconds
      await kwsArmTestWindow(8000);

      // Wait for test window to expire
      await new Promise(resolve => setTimeout(resolve, 8000));

      // If still testing (no pass event received), mark as fail
      setIsTesting(false);
      if (testResult === "idle") {
        setTestResult("fail");
        toast.warning(t.kws.testFailRetry);
      }
    } catch (error) {
      console.error("Test failed:", error);
      toast.error(`Test error: ${error}`);
      setIsTesting(false);
      setTestResult("fail");
    }
  }

  async function handleEnable() {
    if (!selectedModelId) {
      toast.error(t.kws.selectModel);
      return;
    }

    setIsEnabling(true);
    setVerificationState("idle");
    setDownloadProgress(null);

    try {
      const result = await kwsEnable(selectedModelId);
      console.log("KWS Enable result:", result);
      // Success handled by onEnabled event
    } catch (error) {
      toast.error(`Failed to enable KWS: ${error}`);
      setIsEnabling(false);
    }
  }

  async function handleDisable() {
    setIsLoading(true);
    try {
      const result = await kwsDisable();
      console.log("KWS Disable result:", result);
      // Success handled by onDisabled event
    } catch (error) {
      toast.error(`Failed to disable KWS: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  const isRealMode = status?.mode === "real";
  const formatSize = (bytes: number) => (bytes / 1024 / 1024).toFixed(1) + " MB";

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Brain className="h-5 w-5" />
              {t.kws.title}
            </CardTitle>
            <CardDescription>
              Configure real wake-word detection with Sherpa-ONNX
            </CardDescription>
          </div>
          <Badge variant={isRealMode ? "default" : "secondary"}>
            {isRealMode ? t.kws.statusReal : t.kws.statusStub}
            {isRealMode && status?.model_id && ` (${status.model_id})`}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-6">
        {/* Model Selector */}
        <div className="space-y-2">
          <label className="text-sm font-medium flex items-center gap-2">
            {t.kws.currentModel}
          </label>
          <Select
            value={selectedModelId}
            onValueChange={setSelectedModelId}
            disabled={isLoading || isEnabling}
          >
            <SelectTrigger>
              <SelectValue placeholder={t.kws.selectModel} />
            </SelectTrigger>
            <SelectContent>
              {models.length === 0 ? (
                <SelectItem value="none" disabled>
                  {t.kws.noModels}
                </SelectItem>
              ) : (
                models.map((model) => {
                  const modelId = extractModelId(model.description);
                  return (
                    <SelectItem key={modelId} value={modelId}>
                      <div className="flex items-center gap-2">
                        <span>{getModelDisplayName(model)}</span>
                        <Badge variant="outline" className="text-xs">
                          {model.lang.toUpperCase()}
                        </Badge>
                        <span className="text-xs text-muted-foreground">
                          {formatSize(model.size)}
                        </span>
                      </div>
                    </SelectItem>
                  );
                })
              )}
            </SelectContent>
          </Select>
        </div>

        {/* Download Progress */}
        {downloadProgress && (
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>{t.kws.downloading}</span>
              <span className="text-muted-foreground">
                {format(t.kws.downloadProgress, {
                  percent: downloadProgress.percent.toFixed(1),
                })}
              </span>
            </div>
            <div className="w-full bg-secondary rounded-full h-2">
              <div
                className="bg-primary rounded-full h-2 transition-all duration-300"
                style={{ width: `${downloadProgress.percent}%` }}
              />
            </div>
          </div>
        )}

        {/* Verification State */}
        {verificationState === "verifying" && (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <AlertCircle className="h-4 w-4 animate-pulse" />
            <span>{t.kws.verifying}</span>
          </div>
        )}
        {verificationState === "verified" && (
          <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
            <CheckCircle2 className="h-4 w-4" />
            <span>{t.kws.verified}</span>
          </div>
        )}
        {verificationState === "failed" && (
          <div className="flex items-center gap-2 text-sm text-amber-600 dark:text-amber-400">
            <AlertCircle className="h-4 w-4" />
            <span>{t.kws.verifyFailed}</span>
          </div>
        )}

        {/* Actions */}
        <div className="flex flex-wrap gap-2 pt-2">
          {!isRealMode ? (
            <Button
              onClick={handleEnable}
              disabled={isLoading || isEnabling || !selectedModelId}
              variant="default"
            >
              {isEnabling ? t.kws.downloading : t.kws.enable}
            </Button>
          ) : (
            <>
              <Button
                onClick={handleDisable}
                disabled={isLoading}
                variant="outline"
              >
                {t.kws.disable}
              </Button>
              <Button
                onClick={handleTest}
                disabled={isTesting}
                variant="secondary"
              >
                <Play className="h-4 w-4 mr-2" />
                {isTesting ? t.kws.testing : t.kws.testButton}
              </Button>
            </>
          )}
        </div>

        {/* Test State Display */}
        {countdown !== null && (
          <div className="flex items-center gap-2 text-sm font-medium text-blue-600 dark:text-blue-400">
            <span>{countdown}...</span>
            <span>{t.kws.testHintSayNow}</span>
          </div>
        )}
        {testResult === "pass" && (
          <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
            <CheckCircle2 className="h-4 w-4" />
            <span>{t.kws.testPass}</span>
          </div>
        )}
        {testResult === "fail" && (
          <div className="flex items-center gap-2 text-sm text-amber-600 dark:text-amber-400">
            <AlertCircle className="h-4 w-4" />
            <span>{t.kws.testFailRetry}</span>
          </div>
        )}

        {/* Status Info */}
        {status && (
          <div className="text-xs text-muted-foreground space-y-1 pt-4 border-t">
            <div>Keyword: <code className="text-xs">{status.keyword}</code></div>
            {status.lang && <div>Language: {status.lang}</div>}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
