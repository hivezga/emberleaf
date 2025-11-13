/**
 * Onboarding Wizard - ONB-001
 *
 * First-run wizard: Welcome → Preflight → Mic Test → Voice (stub) → Ready
 */

import { useState, useEffect } from "react";
import { CheckCircle2, Mic, Shield, Sparkles, ArrowRight, ArrowLeft } from "lucide-react";
import { useI18n } from "../../lib/i18n";
import { runPreflightChecks, PreflightReport, isTauriEnv } from "../../lib/tauriSafe";
import { listen } from "@tauri-apps/api/event";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

type OnboardingStep = "welcome" | "preflight" | "micTest" | "voice" | "ready";

interface OnboardingWizardProps {
  open: boolean;
  onComplete: () => void;
}

interface StepIndicatorProps {
  currentStep: OnboardingStep;
  steps: OnboardingStep[];
}

function StepIndicator({ currentStep, steps }: StepIndicatorProps) {
  const { t } = useI18n();
  const currentIndex = steps.indexOf(currentStep);

  const getStepLabel = (step: OnboardingStep): string => {
    switch (step) {
      case "welcome":
        return t.onboarding.stepWelcome;
      case "preflight":
        return t.onboarding.stepPreflight;
      case "micTest":
        return t.onboarding.stepMicTest;
      case "voice":
        return t.onboarding.stepVoice;
      case "ready":
        return t.onboarding.stepReady;
    }
  };

  return (
    <div className="flex items-center justify-center gap-2 mb-8" role="navigation" aria-label="Onboarding progress">
      {steps.map((step, index) => {
        const isActive = index === currentIndex;
        const isCompleted = index < currentIndex;

        return (
          <div key={step} className="flex items-center gap-2">
            <div
              className={`flex items-center justify-center w-8 h-8 rounded-full text-sm font-medium transition-colors ${
                isActive
                  ? "bg-primary text-primary-foreground"
                  : isCompleted
                  ? "bg-green-500 text-white"
                  : "bg-muted text-muted-foreground"
              }`}
              aria-current={isActive ? "step" : undefined}
              aria-label={`${getStepLabel(step)}${isCompleted ? " (completed)" : ""}`}
            >
              {isCompleted ? <CheckCircle2 className="h-4 w-4" /> : index + 1}
            </div>
            {index < steps.length - 1 && (
              <div
                className={`w-12 h-0.5 transition-colors ${
                  isCompleted ? "bg-green-500" : "bg-muted"
                }`}
                aria-hidden="true"
              />
            )}
          </div>
        );
      })}
    </div>
  );
}

interface WelcomeStepProps {
  onNext: () => void;
}

function WelcomeStep({ onNext }: WelcomeStepProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-col items-center text-center max-w-2xl mx-auto">
      <div className="mb-6">
        <Sparkles className="h-16 w-16 text-primary mx-auto mb-4" />
      </div>
      <h1 className="text-4xl font-bold mb-4">{t.onboarding.welcomeTitle}</h1>
      <p className="text-xl text-muted-foreground mb-8">{t.onboarding.welcomeSubtitle}</p>

      <Card className="mb-8 text-left">
        <CardContent className="pt-6">
          <div className="flex items-start gap-4 mb-4">
            <Shield className="h-6 w-6 text-green-500 mt-1" />
            <div>
              <p className="text-base">{t.onboarding.welcomeIntro}</p>
            </div>
          </div>
          <div className="flex items-start gap-4">
            <CheckCircle2 className="h-6 w-6 text-green-500 mt-1" />
            <div>
              <p className="text-base text-muted-foreground">{t.onboarding.welcomePrivacy}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Button size="lg" onClick={onNext} className="w-full max-w-xs">
        {t.onboarding.getStarted}
        <ArrowRight className="h-5 w-5 ml-2" />
      </Button>
    </div>
  );
}

interface PreflightStepProps {
  onNext: () => void;
  onBack: () => void;
}

function PreflightStep({ onNext, onBack }: PreflightStepProps) {
  const { t } = useI18n();
  const [report, setReport] = useState<PreflightReport | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const runChecks = async () => {
      try {
        const result = await runPreflightChecks();
        setReport(result);
      } catch (error) {
        console.error("Preflight check failed:", error);
      } finally {
        setLoading(false);
      }
    };

    runChecks();

    // Listen to preflight events
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      if (await isTauriEnv()) {
        unlisten = await listen("preflight:done", (event) => {
          setReport(event.payload as PreflightReport);
          setLoading(false);
        });
      }
    };
    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleContinue = () => {
    onNext();
  };

  const openFixGuide = () => {
    const guideUrl = "https://github.com/your-org/relentless/blob/main/docs/INSTALL-Linux.md";
    window.open(guideUrl, "_blank");
  };

  return (
    <div className="flex flex-col items-center max-w-2xl mx-auto">
      <h2 className="text-3xl font-bold mb-2 text-center">{t.onboarding.preflightTitle}</h2>
      <p className="text-muted-foreground mb-8 text-center">{t.onboarding.preflightSubtitle}</p>

      {loading && (
        <div className="flex flex-col items-center justify-center py-12">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mb-4"></div>
          <p className="text-muted-foreground">{t.onboarding.preflightWait}</p>
        </div>
      )}

      {!loading && report && (
        <div className="w-full space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                {report.overall === "pass" || report.overall === "warn" ? (
                  <CheckCircle2 className="h-6 w-6 text-green-500" />
                ) : (
                  <span className="text-2xl">⚠️</span>
                )}
                <CardTitle>
                  {report.can_proceed
                    ? t.preflight.canProceed
                    : t.preflight.cannotProceed}
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {report.items.map((item) => (
                  <div key={item.name} className="flex items-center justify-between p-3 bg-muted rounded-lg">
                    <span className="font-medium">{item.message}</span>
                    <Badge
                      variant={
                        item.status === "pass"
                          ? "default"
                          : item.status === "warn"
                          ? "warning"
                          : "destructive"
                      }
                    >
                      {item.status === "pass"
                        ? t.preflight.statusReady
                        : item.status === "warn"
                        ? t.preflight.statusWarning
                        : t.preflight.statusError}
                    </Badge>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <div className="flex gap-3">
            <Button variant="outline" onClick={onBack} className="flex-1">
              <ArrowLeft className="h-4 w-4 mr-2" />
              {t.onboarding.back}
            </Button>
            {report.can_proceed ? (
              <Button onClick={handleContinue} className="flex-1">
                {t.onboarding.next}
                <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            ) : (
              <Button onClick={openFixGuide} className="flex-1">
                {t.preflight.fixGuide}
              </Button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

interface MicTestStepProps {
  onNext: () => void;
  onBack: () => void;
  onSkip: () => void;
}

function MicTestStep({ onNext, onBack, onSkip }: MicTestStepProps) {
  const { t } = useI18n();
  const [rmsLevel, setRmsLevel] = useState(0);
  const [hasSignal, setHasSignal] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      if (await isTauriEnv()) {
        unlisten = await listen<{ rms: number }>("audio:rms", (event) => {
          const level = event.payload.rms;
          setRmsLevel(level);

          // Detect signal above threshold
          if (level > 0.05 && !hasSignal) {
            setHasSignal(true);
          }
        });
      }
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [hasSignal]);

  const signalStrength = Math.min(rmsLevel * 200, 100); // Scale to percentage

  return (
    <div className="flex flex-col items-center max-w-2xl mx-auto">
      <h2 className="text-3xl font-bold mb-2 text-center">{t.onboarding.micTestTitle}</h2>
      <p className="text-muted-foreground mb-8 text-center">{t.onboarding.micTestSubtitle}</p>

      <Card className="w-full mb-6">
        <CardContent className="pt-6">
          <div className="flex items-center gap-4 mb-4">
            <Mic className="h-8 w-8 text-primary" />
            <div className="flex-1">
              <p className="font-medium mb-2">{t.onboarding.micTestInstruction}</p>
              <div className="w-full h-4 bg-muted rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary transition-all duration-100"
                  style={{ width: `${signalStrength}%` }}
                  role="progressbar"
                  aria-valuenow={signalStrength}
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-label="Microphone signal level"
                />
              </div>
            </div>
          </div>

          {!hasSignal && (
            <div className="bg-muted p-4 rounded-lg">
              <p className="text-sm text-muted-foreground">{t.onboarding.micTestHint}</p>
            </div>
          )}

          {hasSignal && (
            <div className="bg-green-500/10 border border-green-500/20 p-4 rounded-lg flex items-center gap-2">
              <CheckCircle2 className="h-5 w-5 text-green-500" />
              <p className="text-sm font-medium text-green-700 dark:text-green-400">
                {t.onboarding.micTestDetected}
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      <div className="flex gap-3 w-full">
        <Button variant="outline" onClick={onBack} className="flex-1">
          <ArrowLeft className="h-4 w-4 mr-2" />
          {t.onboarding.back}
        </Button>
        <Button variant="ghost" onClick={onSkip} className="flex-1">
          {t.onboarding.skipMicTest}
        </Button>
        <Button
          onClick={onNext}
          disabled={!hasSignal}
          className="flex-1"
        >
          {t.onboarding.next}
          <ArrowRight className="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}

interface VoiceStepProps {
  onBack: () => void;
  onSkip: () => void;
}

function VoiceStep({ onBack, onSkip }: VoiceStepProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-col items-center max-w-2xl mx-auto">
      <h2 className="text-3xl font-bold mb-2 text-center">{t.onboarding.voiceTitle}</h2>
      <p className="text-muted-foreground mb-8 text-center">{t.onboarding.voiceSubtitle}</p>

      <Card className="w-full mb-6">
        <CardContent className="pt-6">
          <div className="text-center py-8">
            <Sparkles className="h-12 w-12 text-muted-foreground mx-auto mb-4 opacity-50" />
            <p className="text-muted-foreground mb-2">{t.onboarding.voiceComingSoon}</p>
            <p className="text-sm text-muted-foreground">{t.onboarding.voiceOptional}</p>
          </div>
        </CardContent>
      </Card>

      <div className="flex gap-3 w-full">
        <Button variant="outline" onClick={onBack} className="flex-1">
          <ArrowLeft className="h-4 w-4 mr-2" />
          {t.onboarding.back}
        </Button>
        <Button onClick={onSkip} className="flex-1">
          {t.onboarding.voiceSkip}
          <ArrowRight className="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}

interface ReadyStepProps {
  onFinish: () => void;
}

function ReadyStep({ onFinish }: ReadyStepProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-col items-center text-center max-w-2xl mx-auto">
      <div className="mb-6">
        <div className="w-20 h-20 bg-green-500 rounded-full flex items-center justify-center mx-auto mb-4">
          <CheckCircle2 className="h-12 w-12 text-white" />
        </div>
      </div>
      <h1 className="text-4xl font-bold mb-4">{t.onboarding.readyTitle}</h1>
      <p className="text-xl text-muted-foreground mb-8">{t.onboarding.readySubtitle}</p>

      <Card className="mb-8 text-left w-full">
        <CardContent className="pt-6">
          <p className="text-base">{t.onboarding.readyMessage}</p>
        </CardContent>
      </Card>

      <Button size="lg" onClick={onFinish} className="w-full max-w-xs">
        {t.onboarding.openEmberleaf}
        <ArrowRight className="h-5 w-5 ml-2" />
      </Button>
    </div>
  );
}

export function OnboardingWizard({ open, onComplete }: OnboardingWizardProps) {
  const steps: OnboardingStep[] = ["welcome", "preflight", "micTest", "voice", "ready"];
  const [currentStep, setCurrentStep] = useState<OnboardingStep>("welcome");

  // Emit onboarding events
  useEffect(() => {
    if (open) {
      const emitEvent = async () => {
        if (await isTauriEnv()) {
          const { emit } = await import("@tauri-apps/api/event");
          await emit("onboarding:started", {});
        }
      };
      emitEvent();
    }
  }, [open]);

  useEffect(() => {
    if (open) {
      const emitEvent = async () => {
        if (await isTauriEnv()) {
          const { emit } = await import("@tauri-apps/api/event");
          await emit("onboarding:step", { step: currentStep });
        }
      };
      emitEvent();
    }
  }, [currentStep, open]);

  const handleFinish = async () => {
    if (await isTauriEnv()) {
      const { emit } = await import("@tauri-apps/api/event");
      await emit("onboarding:completed", {});
    }
    onComplete();
  };

  const goToStep = (step: OnboardingStep) => {
    setCurrentStep(step);
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 bg-background flex flex-col"
      role="dialog"
      aria-modal="true"
      aria-labelledby="onboarding-title"
    >
      <div className="flex-1 overflow-y-auto py-12 px-4">
        <div className="max-w-4xl mx-auto">
          <StepIndicator currentStep={currentStep} steps={steps} />

          {currentStep === "welcome" && (
            <WelcomeStep onNext={() => goToStep("preflight")} />
          )}

          {currentStep === "preflight" && (
            <PreflightStep
              onNext={() => goToStep("micTest")}
              onBack={() => goToStep("welcome")}
            />
          )}

          {currentStep === "micTest" && (
            <MicTestStep
              onNext={() => goToStep("voice")}
              onBack={() => goToStep("preflight")}
              onSkip={() => goToStep("voice")}
            />
          )}

          {currentStep === "voice" && (
            <VoiceStep
              onBack={() => goToStep("micTest")}
              onSkip={() => goToStep("ready")}
            />
          )}

          {currentStep === "ready" && <ReadyStep onFinish={handleFinish} />}
        </div>
      </div>
    </div>
  );
}
