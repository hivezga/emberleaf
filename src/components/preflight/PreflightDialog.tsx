/**
 * Preflight Dialog - REL-001-UI
 *
 * User-friendly system check UI with Simple and Advanced modes
 */

import { useState, useEffect } from "react";
import { CheckCircle2, AlertTriangle, XCircle, ChevronDown, ChevronUp, Copy } from "lucide-react";
import { PreflightReport, PreflightItem, CheckStatus, runPreflightChecks } from "../../lib/tauriSafe";
import { useI18n } from "../../lib/i18n";
import { listen } from "@tauri-apps/api/event";
import { isTauriEnv } from "../../lib/tauriSafe";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "../ui/sheet";
import { Card, CardContent } from "../ui/card";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";

interface PreflightDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: () => void;
}

interface PreflightItemRowProps {
  item: PreflightItem;
  expanded: boolean;
  onToggle: () => void;
}

function getStatusIcon(status: CheckStatus) {
  switch (status) {
    case "pass":
      return <CheckCircle2 className="h-5 w-5 text-green-500" />;
    case "warn":
      return <AlertTriangle className="h-5 w-5 text-amber-glow" />;
    case "fail":
      return <XCircle className="h-5 w-5 text-destructive" />;
  }
}

function getStatusBadgeVariant(status: CheckStatus): "default" | "warning" | "destructive" {
  switch (status) {
    case "pass":
      return "default";
    case "warn":
      return "warning";
    case "fail":
      return "destructive";
  }
}

function PreflightItemRow({ item, expanded, onToggle }: PreflightItemRowProps) {
  const { t } = useI18n();

  const getCheckName = (name: string) => {
    switch (name) {
      case "audio_stack":
        return t.preflight.checkAudio;
      case "webkit":
        return t.preflight.checkWebkit;
      case "portal":
        return t.preflight.checkPortal;
      case "mic_access":
        return t.preflight.checkMicAccess;
      default:
        return name;
    }
  };

  const getStatusLabel = (status: CheckStatus) => {
    switch (status) {
      case "pass":
        return t.preflight.statusReady;
      case "warn":
        return t.preflight.statusWarning;
      case "fail":
        return t.preflight.statusError;
    }
  };

  return (
    <Card className="overflow-hidden">
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3 flex-1">
            {getStatusIcon(item.status)}
            <div className="flex-1">
              <div className="font-medium">{getCheckName(item.name)}</div>
              <div className="text-sm text-muted-foreground">{item.message}</div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={getStatusBadgeVariant(item.status)}>
              {getStatusLabel(item.status)}
            </Badge>
            {item.fix_hint && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onToggle}
                aria-label={expanded ? "Collapse details" : "Expand details"}
                aria-expanded={expanded}
              >
                {expanded ? (
                  <ChevronUp className="h-4 w-4" />
                ) : (
                  <ChevronDown className="h-4 w-4" />
                )}
              </Button>
            )}
          </div>
        </div>

        {/* Advanced details */}
        {expanded && item.fix_hint && (
          <div
            className="mt-4 pt-4 border-t space-y-2"
            role="region"
            aria-label="Additional details"
          >
            <div className="text-sm">
              <span className="font-medium">{t.preflight.detailsLabel}: </span>
              <span className="text-muted-foreground">{item.fix_hint}</span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function PreflightDialog({ open, onOpenChange, onComplete }: PreflightDialogProps) {
  const { t } = useI18n();
  const [report, setReport] = useState<PreflightReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [advancedMode, setAdvancedMode] = useState(false);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  const [copySuccess, setCopySuccess] = useState(false);

  const runChecks = async () => {
    setLoading(true);
    try {
      const result = await runPreflightChecks();
      setReport(result);
    } catch (error) {
      console.error("Preflight check failed:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (open && !report) {
      runChecks();
    }
  }, [open]);

  // Listen to preflight events if in Tauri environment
  useEffect(() => {
    if (!open) return;

    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      if (await isTauriEnv()) {
        unlisten = await listen("preflight:done", (event) => {
          setReport(event.payload as PreflightReport);
        });
      }
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [open]);

  const toggleItemExpanded = (itemName: string) => {
    setExpandedItems((prev) => {
      const next = new Set(prev);
      if (next.has(itemName)) {
        next.delete(itemName);
      } else {
        next.add(itemName);
      }
      return next;
    });
  };

  const handleCopyJson = async () => {
    if (!report) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(report, null, 2));
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    } catch (error) {
      console.error("Failed to copy JSON:", error);
    }
  };

  const handleContinue = () => {
    if (onComplete) {
      onComplete();
    }
    onOpenChange(false);
  };

  const openFixGuide = () => {
    // Open the INSTALL-Linux.md file
    // In web mode, we could link to GitHub; in Tauri, open in external browser
    const guideUrl = "https://github.com/your-org/relentless/blob/main/docs/INSTALL-Linux.md";
    window.open(guideUrl, "_blank");
  };

  const getOverallStatus = () => {
    if (!report) return null;
    switch (report.overall) {
      case "pass":
        return { icon: <CheckCircle2 className="h-6 w-6 text-green-500" />, text: t.preflight.statusReady };
      case "warn":
        return { icon: <AlertTriangle className="h-6 w-6 text-amber-glow" />, text: t.preflight.statusWarning };
      case "fail":
        return { icon: <XCircle className="h-6 w-6 text-destructive" />, text: t.preflight.statusError };
    }
  };

  const overallStatus = getOverallStatus();

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-full sm:max-w-lg overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            {overallStatus && overallStatus.icon}
            {t.preflight.title}
          </SheetTitle>
          <SheetDescription>
            {loading && t.preflight.checking}
            {!loading && report && (
              report.can_proceed ? t.preflight.canProceed : t.preflight.cannotProceed
            )}
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-4">
          {/* Loading state */}
          {loading && (
            <div className="flex items-center justify-center py-8">
              <div className="text-muted-foreground">{t.loading}</div>
            </div>
          )}

          {/* Results */}
          {!loading && report && (
            <>
              {/* Overall status badge */}
              {overallStatus && (
                <div className="flex items-center justify-between pb-4 border-b">
                  <div className="flex items-center gap-2">
                    {overallStatus.icon}
                    <span className="font-semibold">{overallStatus.text}</span>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setAdvancedMode(!advancedMode)}
                    aria-pressed={advancedMode}
                  >
                    {t.preflight.advancedMode}
                  </Button>
                </div>
              )}

              {/* Check items */}
              <div className="space-y-3">
                {report.items.map((item) => (
                  <PreflightItemRow
                    key={item.name}
                    item={item}
                    expanded={advancedMode && expandedItems.has(item.name)}
                    onToggle={() => toggleItemExpanded(item.name)}
                  />
                ))}
              </div>

              {/* Advanced mode - Copy JSON */}
              {advancedMode && (
                <div className="pt-4 border-t">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleCopyJson}
                    className="w-full"
                  >
                    <Copy className="h-4 w-4 mr-2" />
                    {copySuccess ? t.preflight.copiedToClipboard : t.preflight.copyJson}
                  </Button>
                </div>
              )}

              {/* Actions */}
              <div className="flex flex-col gap-2 pt-4 border-t">
                {report.can_proceed ? (
                  <Button onClick={handleContinue} className="w-full">
                    {t.preflight.continue}
                  </Button>
                ) : (
                  <>
                    <Button onClick={openFixGuide} variant="default" className="w-full">
                      {t.preflight.fixGuide}
                    </Button>
                    <Button onClick={runChecks} variant="outline" className="w-full">
                      {t.preflight.rerun}
                    </Button>
                  </>
                )}
              </div>
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
