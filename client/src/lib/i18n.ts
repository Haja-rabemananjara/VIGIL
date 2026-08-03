import en from "../locales/en.json";
import fr from "../locales/fr.json";

export type Language = "en" | "fr";
export type TranslationKey = keyof typeof en;

const dictionaries: Record<Language, Record<string, string>> = { en, fr };
const STORAGE_KEY = "vigil_language";

function readStoredLanguage(): Language {
  if (typeof window === "undefined") return "en";
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "fr" ? "fr" : "en";
}

let currentLang: Language = "en";
let initialized = false;

export function initLanguage(): void {
  if (initialized) return;
  initialized = true;
  currentLang = readStoredLanguage();
}

export function setLanguage(lang: Language): void {
  currentLang = lang;
  localStorage.setItem(STORAGE_KEY, lang);
}

export function getLanguage(): Language {
  return currentLang;
}

export function t(key: TranslationKey): string {
  return dictionaries[currentLang][key] ?? en[key] ?? key;
}
