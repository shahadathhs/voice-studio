/** Languages offered for a fixed source language; Whisper accepts many more by code. */
export const LANGUAGES: readonly { code: string; label: string }[] = [
  { code: "en", label: "English" },
  { code: "bn", label: "Bengali" },
  { code: "zh", label: "Chinese" },
  { code: "nl", label: "Dutch" },
  { code: "fr", label: "French" },
  { code: "de", label: "German" },
  { code: "hi", label: "Hindi" },
  { code: "it", label: "Italian" },
  { code: "ja", label: "Japanese" },
  { code: "ko", label: "Korean" },
  { code: "pl", label: "Polish" },
  { code: "pt", label: "Portuguese" },
  { code: "ru", label: "Russian" },
  { code: "es", label: "Spanish" },
  { code: "sv", label: "Swedish" },
  { code: "tr", label: "Turkish" },
  { code: "uk", label: "Ukrainian" },
];

/** Name for a language code such as "en"; falls back to the code itself. */
export function languageName(code: string | null): string {
  if (!code) return "Unknown";
  return LANGUAGES.find((language) => language.code === code)?.label ?? code.toUpperCase();
}
