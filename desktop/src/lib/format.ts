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

/** `m:ss`, or `h:mm:ss` from one hour up. */
export function formatClock(ms: number): string {
  const total = Math.floor(ms / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${minutes}:${seconds}`;
}

/** Local date of a Unix timestamp in seconds, e.g. "Oct 22, 2026". */
export function formatDate(seconds: number): string {
  return new Date(seconds * 1000).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/** The last path component, for showing where a file came from. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}
