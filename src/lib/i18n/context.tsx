/**
 * i18n Context for Emberleaf
 * Provides language switching and translation hooks
 */

import { createContext, useContext, useState, ReactNode } from "react";
import type { Language, Translations } from "./translations";
import { translations } from "./translations";

interface I18nContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: Translations;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

const STORAGE_KEY = "emberleaf_language";

export function I18nProvider({ children }: { children: ReactNode }) {
  // Default to Spanish (MX) as specified in the ticket
  const [language, setLanguageState] = useState<Language>(() => {
    if (typeof window === "undefined") return "es";
    const saved = localStorage.getItem(STORAGE_KEY);
    return (saved as Language) || "es";
  });

  const setLanguage = (lang: Language) => {
    setLanguageState(lang);
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, lang);
    }
  };

  const t = translations[language];

  return (
    <I18nContext.Provider value={{ language, setLanguage, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return context;
}
