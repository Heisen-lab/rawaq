import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import arCatalog from "./ar.json";
import enCatalog from "./en.json";
import {
  flattenTranslations,
  formatTranslation,
  getLocaleDirection,
  type SupportedLocale,
  type TextDirection,
} from "./parity";

const FLAT_CATALOGS: Record<SupportedLocale, Record<string, string>> = {
  ar: flattenTranslations(arCatalog),
  en: flattenTranslations(enCatalog),
};

const STORAGE_KEY = "rawaq.locale";

export interface I18nContextValue {
  locale: SupportedLocale;
  direction: TextDirection;
  setLocale: (next: SupportedLocale) => void;
  toggleLocale: () => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

function readInitialLocale(defaultLocale: SupportedLocale): SupportedLocale {
  try {
    if (typeof window !== "undefined" && window.localStorage) {
      const saved = window.localStorage.getItem(STORAGE_KEY);
      if (saved === "ar" || saved === "en") {
        return saved;
      }
    }
  } catch {
    // Ignore storage access errors in restricted environments
  }
  return defaultLocale;
}

function syncDocumentDirection(locale: SupportedLocale): TextDirection {
  const dir = getLocaleDirection(locale);
  if (typeof document !== "undefined" && document.documentElement) {
    document.documentElement.lang = locale;
    document.documentElement.dir = dir;
    document.documentElement.setAttribute("data-direction", dir);
  }
  return dir;
}

export interface I18nProviderProps {
  initialLocale?: SupportedLocale;
  children: React.ReactNode;
}

export function I18nProvider({
  initialLocale = "ar",
  children,
}: I18nProviderProps): React.ReactElement {
  const [locale, setLocaleState] = useState<SupportedLocale>(() => {
    const start = readInitialLocale(initialLocale);
    syncDocumentDirection(start);
    return start;
  });

  const direction = getLocaleDirection(locale);

  useEffect(() => {
    syncDocumentDirection(locale);
  }, [locale]);

  const setLocale = useCallback((next: SupportedLocale) => {
    syncDocumentDirection(next);
    setLocaleState(next);
    try {
      if (typeof window !== "undefined" && window.localStorage) {
        window.localStorage.setItem(STORAGE_KEY, next);
      }
    } catch {
      // Ignore storage write errors
    }
  }, []);

  const toggleLocale = useCallback(() => {
    setLocale(locale === "ar" ? "en" : "ar");
  }, [locale, setLocale]);

  const t = useCallback(
    (key: string, params?: Record<string, string | number>): string => {
      const catalog = FLAT_CATALOGS[locale];
      const template = catalog[key];
      if (template === undefined) {
        return key;
      }
      return formatTranslation(template, params);
    },
    [locale],
  );

  const value = useMemo<I18nContextValue>(
    () => ({
      locale,
      direction,
      setLocale,
      toggleLocale,
      t,
    }),
    [locale, direction, setLocale, toggleLocale, t],
  );

  return (
    <I18nContext.Provider value={value}>
      <div
        data-testid="bidi-root"
        dir={direction}
        lang={locale}
        className="rawaq-root"
      >
        {children}
      </div>
    </I18nContext.Provider>
  );
}

export function useI18n(): I18nContextValue {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider");
  }
  return context;
}
