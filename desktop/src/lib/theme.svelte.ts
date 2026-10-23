export type ThemePreference = "system" | "light" | "dark";

// Keep the key and the resolution rule in sync with public/theme-init.js.
const STORAGE_KEY = "voice-studio-theme";
const PREFERENCES: readonly ThemePreference[] = ["system", "light", "dark"];
const systemQuery = window.matchMedia("(prefers-color-scheme: dark)");

function readStored(): ThemePreference {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return PREFERENCES.find((p) => p === stored) ?? "system";
  } catch {
    return "system";
  }
}

function applyToDocument(preference: ThemePreference): void {
  const dark = preference === "dark" || (preference === "system" && systemQuery.matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
}

class ThemeStore {
  preference = $state<ThemePreference>(readStored());

  constructor() {
    applyToDocument(this.preference);
    // "System" follows the OS live, including when it switches at sunset.
    systemQuery.addEventListener("change", () => applyToDocument(this.preference));
  }

  set(preference: ThemePreference): void {
    this.preference = preference;
    applyToDocument(preference);
    try {
      localStorage.setItem(STORAGE_KEY, preference);
    } catch {
      // The choice still applies for this session if storage is unavailable.
    }
  }
}

export const theme = new ThemeStore();
