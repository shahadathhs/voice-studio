import { i18n, t } from "./i18n.svelte";

/** Source languages offered in the language list; Whisper accepts many more by code. */
const CODES = [
  "en",
  "bn",
  "zh",
  "nl",
  "fr",
  "de",
  "hi",
  "it",
  "ja",
  "ko",
  "pl",
  "pt",
  "ru",
  "es",
  "sv",
  "tr",
  "uk",
] as const;

/** A language's name in the interface language, from the platform's own language data. */
function displayName(code: string): string {
  try {
    return new Intl.DisplayNames([i18n.locale.code], { type: "language" }).of(code) ?? code;
  } catch {
    return code.toUpperCase();
  }
}

/** The languages to choose from, named in the interface language and sorted by those names. */
export function languageOptions(): { code: string; label: string }[] {
  return CODES.map((code) => ({ code, label: displayName(code) })).sort((a, b) =>
    a.label.localeCompare(b.label, i18n.locale.code),
  );
}

/** Name for a language code such as "en"; "Unknown" when there is none. */
export function languageName(code: string | null): string {
  return code ? displayName(code) : t("common.unknown");
}
