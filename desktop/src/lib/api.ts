import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { AppInfo, FinishedEvent, ModelStatus, ProgressEvent } from "./types";

/** True inside the Tauri window; false when the UI is opened in a plain browser (pnpm dev). */
const insideTauri = (): boolean => "__TAURI_INTERNALS__" in window;

const BROWSER_PREVIEW: AppInfo = {
  name: "Voice Studio",
  version: "browser preview",
  coreVersion: "unavailable",
  dataDir: "unavailable",
};

const noop = async (): Promise<void> => {};

export async function appInfo(): Promise<AppInfo> {
  return insideTauri() ? invoke<AppInfo>("app_info") : BROWSER_PREVIEW;
}

export async function listModels(): Promise<ModelStatus[]> {
  return insideTauri() ? invoke<ModelStatus[]>("list_models") : [];
}

/** Starts a background install; follow it with `onModelProgress` / `onModelFinished`. */
export async function installModel(id: string): Promise<void> {
  if (insideTauri()) await invoke("install_model", { id });
}

export async function cancelInstall(id: string): Promise<void> {
  if (insideTauri()) await invoke("cancel_install", { id });
}

export async function removeModel(id: string): Promise<void> {
  if (insideTauri()) await invoke("remove_model", { id });
}

export function onModelProgress(handler: (event: ProgressEvent) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<ProgressEvent>("model-progress", (event) => handler(event.payload));
}

export function onModelFinished(handler: (event: FinishedEvent) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<FinishedEvent>("model-finished", (event) => handler(event.payload));
}
