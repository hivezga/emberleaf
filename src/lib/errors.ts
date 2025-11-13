/**
 * Error mapping utilities for validation errors
 * Maps backend validation errors to localized user messages
 */

import type { Language, Translations } from "./i18n/translations";
import { translations } from "./i18n/translations";

export type ValidationErrorPayload = {
  code: string;
  message: string;
  field?: string;
  value?: unknown;
};

/**
 * Get current translations based on stored language preference
 * Falls back to Spanish (default) if no preference is found
 */
function getI18n(): { t: Translations; language: Language } {
  const STORAGE_KEY = "emberleaf_language";
  const storedLang =
    typeof window !== "undefined" ? localStorage.getItem(STORAGE_KEY) : null;
  const language: Language = (storedLang as Language) || "es";
  return { t: translations[language], language };
}

/**
 * Convert error payload to user-friendly localized message
 * Handles validation errors from audio:error events and Tauri invoke errors
 */
export function toUserMessage(err: unknown): string {
  const { t } = getI18n();

  // Backend uniform payload from audio:error
  if (typeof err === "object" && err && "code" in err) {
    const code = (err as any).code as string;
    // Try to find localized message for the error code
    if (code in t.errors) {
      return (t.errors as any)[code];
    }
    // Fallback to message from payload
    return (err as any).message ?? t.errors.unknown_error;
  }

  // Tauri invoke errors often come as strings
  if (typeof err === "string") return err;

  // Fallback
  return t.errors.unknown_error;
}
