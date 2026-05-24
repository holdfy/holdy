import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import en from "./locales/en.json";
import ptBR from "./locales/pt-BR.json";
import es from "./locales/es.json";

export const SUPPORTED_LOCALES = ["pt-BR", "es", "en"] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

export const LOCALE_LABELS: Record<SupportedLocale, string> = {
  "pt-BR": "Português (BR)",
  es: "Español",
  en: "English",
};

export const LOCALE_HTML_LANG: Record<SupportedLocale, string> = {
  "pt-BR": "pt-BR",
  es: "es",
  en: "en",
};

export const LOCALE_NUMBER_FORMAT: Record<SupportedLocale, string> = {
  "pt-BR": "pt-BR",
  es: "es-ES",
  en: "en-US",
};

/** Maps browser / legacy codes (en-US, pt, es-AR, etc.) to our supported locales. */
export function normalizeLocale(lng: string | undefined): SupportedLocale {
  const code = (lng ?? "").toLowerCase();
  if (code === "pt-br" || code === "pt") return "pt-BR";
  if (code === "es-ar" || code.startsWith("es")) return "es";
  if (code.startsWith("en")) return "en";
  return "pt-BR";
}

function migrateStoredLocale() {
  if (typeof localStorage === "undefined") return;
  const stored = localStorage.getItem("holdfy-site-locale");
  if (!stored) return;
  const normalized = normalizeLocale(stored);
  if (stored !== normalized) {
    localStorage.setItem("holdfy-site-locale", normalized);
  }
}

function applyHtmlLang(lng: string) {
  const normalized = normalizeLocale(lng);
  document.documentElement.lang = LOCALE_HTML_LANG[normalized];
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("holdfy-site-locale", normalized);
  }
}

migrateStoredLocale();

if (!i18n.isInitialized) {
  i18n.use(LanguageDetector).use(initReactI18next);
}

export const i18nReady = (
  i18n.isInitialized
    ? Promise.resolve()
    : i18n.init({
        resources: {
          en: { translation: en },
          "pt-BR": { translation: ptBR },
          es: { translation: es },
        },
        fallbackLng: {
          "pt-BR": ["en"],
          es: ["en"],
          default: ["en"],
        },
        supportedLngs: [...SUPPORTED_LOCALES],
        // Must stay false: with true, i18next strips region codes and pt-BR never resolves.
        nonExplicitSupportedLngs: false,
        interpolation: { escapeValue: false },
        react: {
          useSuspense: false,
          bindI18n: "languageChanged",
          bindI18nStore: "added removed",
        },
        detection: {
          order: ["localStorage", "navigator"],
          lookupLocalStorage: "holdfy-site-locale",
          caches: ["localStorage"],
          convertDetectedLanguage: (lng) => normalizeLocale(lng),
        },
      })
).then(() => {
  applyHtmlLang(i18n.resolvedLanguage ?? i18n.language);
});

i18n.on("languageChanged", (lng) => {
  applyHtmlLang(lng);
});

export default i18n;

if (import.meta.env.DEV) {
  (window as unknown as { __holdfyI18n?: typeof i18n }).__holdfyI18n = i18n;
}
