import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { insideTauri, noop } from "./api";
import type { DictationState } from "./types";

/** Starts or stops dictation, as the global shortcut does. */
export async function toggleDictation(): Promise<void> {
  if (insideTauri()) await invoke("toggle_dictation");
}

export function onDictationState(handler: (state: DictationState) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<DictationState>("dictation-state", (event) => handler(event.payload));
}
