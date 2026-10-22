const UNITS = ["B", "KB", "MB", "GB"] as const;

/** Human-readable size using decimal units, matching what the OS file manager shows. */
export function formatBytes(bytes: number): string {
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < UNITS.length - 1) {
    value /= 1000;
    unit += 1;
  }
  const rounded = value >= 100 || unit === 0 ? Math.round(value) : Math.round(value * 10) / 10;
  return `${rounded} ${UNITS[unit]}`;
}

/** Short label for a model's language support; the catalog uses "multilingual" for Whisper. */
export function languageLabel(languages: readonly string[]): string {
  if (languages.length === 0) return "—";
  if (languages.includes("multilingual")) return "Multilingual";
  return languages.map((code) => code.toUpperCase()).join(", ");
}

/** Whole-number percentage, 0 when the total is unknown. */
export function percent(done: number, total: number): number {
  return total > 0 ? Math.min(100, Math.floor((done / total) * 100)) : 0;
}
