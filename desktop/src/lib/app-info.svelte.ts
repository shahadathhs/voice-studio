import { appInfo } from "./api";
import type { AppInfo } from "./types";

class AppInfoStore {
  value = $state<AppInfo | null>(null);

  async load(): Promise<void> {
    this.value = await appInfo();
  }
}

export const info = new AppInfoStore();
