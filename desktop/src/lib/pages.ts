import type { MessageKey } from "../locales/en";
import type { IconName } from "./icons";
import type { PageId } from "./nav.svelte";

export interface PageEntry {
  id: PageId;
  label: MessageKey;
  icon: IconName;
}

export const MAIN_PAGES: readonly PageEntry[] = [
  { id: "transcribe", label: "nav.transcribe", icon: "doc" },
  { id: "record", label: "nav.record", icon: "mic" },
  { id: "speak", label: "nav.speak", icon: "speaker" },
  { id: "library", label: "nav.library", icon: "lib" },
  { id: "models", label: "nav.models", icon: "box" },
];

export const SETTINGS_PAGE: PageEntry = { id: "settings", label: "nav.settings", icon: "gear" };
