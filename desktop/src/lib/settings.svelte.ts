import { getSettings, saveSettings } from "./api";
import type { Settings } from "./types";

const messageOf = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** The saved defaults. Changes are written straight away, so there is no Save button. */
class SettingsStore {
  value = $state<Settings>({
    model: null,
    language: null,
    device: "auto",
    identify_speakers: false,
  });
  error = $state<string | null>(null);

  async load(): Promise<void> {
    try {
      this.value = await getSettings();
      this.error = null;
    } catch (error) {
      this.error = messageOf(error);
    }
  }

  async update(change: Partial<Settings>): Promise<void> {
    try {
      this.value = await saveSettings({ ...this.value, ...change });
      this.error = null;
    } catch (error) {
      this.error = messageOf(error);
    }
  }
}

export const settings = new SettingsStore();
