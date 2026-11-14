import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { setApiLocale } from "../api/client";
import type { MessageKey, PluralKey } from "./messages.ru";
import { pluralize, translate } from "./translate";
import type { MessageParams } from "./translate";
import type { Locale } from "./types";

const STORAGE_KEY = "todooshka.locale";

type I18nValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey, params?: MessageParams) => string;
  plural: (base: PluralKey, count: number) => string;
};

const I18nContext = createContext<I18nValue | null>(null);

function detectLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "ru" || stored === "en") return stored;
  } catch {
    // Private mode may block localStorage; the language is still needed.
  }

  const languages = navigator.languages?.length ? navigator.languages : [navigator.language];
  for (const language of languages) {
    const normalized = language.toLowerCase();
    if (normalized.startsWith("ru")) return "ru";
    if (normalized.startsWith("en")) return "en";
  }
  return "ru";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(detectLocale);

  useEffect(() => {
    document.documentElement.lang = locale;
    setApiLocale(locale);
    try {
      localStorage.setItem(STORAGE_KEY, locale);
    } catch {
      // Without localStorage the choice lasts until reload.
    }
  }, [locale]);

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
  }, []);

  const value = useMemo<I18nValue>(
    () => ({
      locale,
      setLocale,
      t: (key, params) => translate(locale, key, params),
      plural: (base, count) => pluralize(locale, base, count),
    }),
    [locale, setLocale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  const value = useContext(I18nContext);
  if (!value) {
    throw new Error("useI18n called outside I18nProvider");
  }
  return value;
}
