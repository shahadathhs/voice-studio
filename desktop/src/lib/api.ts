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
  Settings,
  SpeechFinished,
  SpeechProgress,
  TranscriptView,
  VoiceInfo,
} from "./types";

/** True inside the Tauri window; false when the UI is opened in a plain browser (pnpm dev). */
const insideTauri = (): boolean => "__TAURI_INTERNALS__" in window;

const BROWSER_PREVIEW: AppInfo = {
  name: "Voice Studio",
  version: "browser preview",
  coreVersion: "unavailable",
  dataDir: "unavailable",
};

const DEFAULT_SETTINGS: Settings = {
  model: null,
  language: null,
  device: "auto",
  identify_speakers: false,
};

const noop = async (): Promise<void> => {};

export async function appInfo(): Promise<AppInfo> {
  return insideTauri() ? invoke<AppInfo>("app_info") : BROWSER_PREVIEW;
}

export async function getSettings(): Promise<Settings> {
  return insideTauri() ? invoke<Settings>("get_settings") : DEFAULT_SETTINGS;
}

/** Stores the settings and returns what was stored. */
export async function saveSettings(settings: Settings): Promise<Settings> {
  return insideTauri() ? invoke<Settings>("save_settings", { settings }) : settings;
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
  identifySpeakers: boolean,
): Promise<JobInfo[]> {
  return insideTauri()
    ? invoke<JobInfo[]>("add_jobs", { files, model, language, translate, identifySpeakers })
    : [];
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

/** Renames a speaker in one transcript; a name another speaker has merges the two. */
export async function renameSpeaker(id: number, from: string, to: string): Promise<void> {
  if (insideTauri()) await invoke("rename_speaker", { id, from, to });
}

/** Asks the user for a plain-text file; null when they cancel or outside the desktop app. */
export async function chooseTextFile(): Promise<string | null> {
  if (!insideTauri()) return null;
  const picked = await open({
    multiple: false,
    filters: [{ name: "Text", extensions: ["txt", "md", "text"] }],
  });
  return picked ?? null;
}

export async function readTextFile(path: string): Promise<string> {
  return insideTauri() ? invoke<string>("read_text_file", { path }) : "";
}

/** The installed voices; empty until the voice model is installed. */
export async function listVoices(): Promise<VoiceInfo[]> {
  return insideTauri() ? invoke<VoiceInfo[]>("list_voices") : [];
}

/** Starts speaking in the background; follow it with `onSpeechProgress` / `onSpeechFinished`. */
export async function startSpeech(text: string, voice: string, speed: number): Promise<void> {
  if (insideTauri()) await invoke("start_speech", { text, voice, speed });
}

export async function cancelSpeech(): Promise<void> {
  if (insideTauri()) await invoke("cancel_speech");
}

/** Copies the latest speech to a file. */
export async function saveSpeech(path: string): Promise<void> {
  if (insideTauri()) await invoke("save_speech", { path });
}

/** URL the webview can play a generated speech file from. */
export const speechUrl = (path: string): string => convertFileSrc(path);

export function onSpeechProgress(handler: (event: SpeechProgress) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<SpeechProgress>("speech-progress", (event) => handler(event.payload));
}

export function onSpeechFinished(handler: (event: SpeechFinished) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<SpeechFinished>("speech-finished", (event) => handler(event.payload));
}
