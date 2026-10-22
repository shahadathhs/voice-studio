import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";

import type {
  AppInfo,
  ExportFormat,
  ExportSettings,
  FinishedEvent,
  JobInfo,
  LibraryItem,
  ModelStatus,
  ProgressEvent,
  SearchHit,
  TranscriptView,
} from "./types";

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

/** File types the transcriber can decode. */
const MEDIA_EXTENSIONS = ["wav", "mp3", "m4a", "aac", "flac", "ogg", "mp4", "mov", "mkv"];

/** Asks the user for audio or video files; empty when they cancel or outside the desktop app. */
export async function chooseFiles(): Promise<string[]> {
  if (!insideTauri()) return [];
  const picked = await open({
    multiple: true,
    filters: [{ name: "Audio and video", extensions: MEDIA_EXTENSIONS }],
  });
  return picked ?? [];
}

/** Asks where to save an export; null when the user cancels. */
export async function chooseSavePath(name: string, extension: string): Promise<string | null> {
  if (!insideTauri()) return null;
  return save({
    defaultPath: `${name}.${extension}`,
    filters: [{ name: extension, extensions: [extension] }],
  });
}

/** Calls `handler` with the paths of files dropped on the window. */
export function onFilesDropped(handler: (paths: string[]) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "drop") handler(event.payload.paths);
  });
}

export async function listJobs(): Promise<JobInfo[]> {
  return insideTauri() ? invoke<JobInfo[]>("list_jobs") : [];
}

/** Queues files; rejects when a file cannot be opened or the model is not installed. */
export async function addJobs(
  files: string[],
  model: string,
  language: string | null,
  translate: boolean,
): Promise<JobInfo[]> {
  return insideTauri() ? invoke<JobInfo[]>("add_jobs", { files, model, language, translate }) : [];
}

export async function cancelJob(id: number): Promise<void> {
  if (insideTauri()) await invoke("cancel_job", { id });
}

export async function removeJob(id: number): Promise<void> {
  if (insideTauri()) await invoke("remove_job", { id });
}

export async function clearFinishedJobs(): Promise<void> {
  if (insideTauri()) await invoke("clear_finished_jobs");
}

export async function moveJob(id: number, position: number): Promise<void> {
  if (insideTauri()) await invoke("move_job", { id, position });
}

export async function setQueuePaused(paused: boolean): Promise<void> {
  if (insideTauri()) await invoke("set_queue_paused", { paused });
}

export function onJobUpdated(handler: (job: JobInfo) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<JobInfo>("job-updated", (event) => handler(event.payload));
}

export async function listLibrary(): Promise<LibraryItem[]> {
  return insideTauri() ? invoke<LibraryItem[]>("list_library") : [];
}

export async function searchLibrary(query: string): Promise<SearchHit[]> {
  return insideTauri() ? invoke<SearchHit[]>("search_library", { query }) : [];
}

export async function getTranscript(id: number): Promise<TranscriptView | null> {
  return insideTauri() ? invoke<TranscriptView>("get_transcript", { id }) : null;
}

export async function deleteLibraryItem(id: number): Promise<void> {
  if (insideTauri()) await invoke("delete_library_item", { id });
}

/** The exported text, for the dialog preview and for Copy. */
export async function renderExport(
  id: number,
  format: ExportFormat,
  settings: ExportSettings,
): Promise<string> {
  return insideTauri() ? invoke<string>("render_export", { id, format, settings }) : "";
}

export async function saveExport(
  id: number,
  format: ExportFormat,
  settings: ExportSettings,
  path: string,
): Promise<void> {
  if (insideTauri()) await invoke("save_export", { id, format, settings, path });
}

/** URL the webview can play the original recording of a library item from. Rejects with a
 *  readable reason when the file was moved or deleted. */
export async function playbackUrl(id: number): Promise<string | null> {
  if (!insideTauri()) return null;
  return convertFileSrc(await invoke<string>("prepare_playback", { id }));
}
