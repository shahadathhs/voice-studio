import type { IconName } from "./icons";
import type { PageId } from "./nav.svelte";

export interface PageEntry {
  id: PageId;
  label: string;
  icon: IconName;
}

export const MAIN_PAGES: readonly PageEntry[] = [
  { id: "transcribe", label: "Transcribe", icon: "doc" },
  { id: "speak", label: "Speak", icon: "speaker" },
  { id: "library", label: "Library", icon: "lib" },
  { id: "models", label: "Models", icon: "box" },
];

export const SETTINGS_PAGE: PageEntry = { id: "settings", label: "Settings", icon: "gear" };
