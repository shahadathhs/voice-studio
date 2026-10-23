import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { insideTauri, noop } from "./api";
import type { Note, TextFinished, TextTask } from "./types";

/** The language codes a summary or translation can be written in. */
export async function textLanguages(): Promise<string[]> {
  return insideTauri() ? invoke<string[]>("text_languages") : [];
}

/** Starts a summary or translation; follow it with `onTextProgress` / `onTextFinished`. */
export async function startTextTask(
  itemId: number,
  task: TextTask,
  language: string | null,
  model: string,
): Promise<void> {
  if (insideTauri()) await invoke("start_text_task", { itemId, task, language, model });
}

export async function cancelTextTask(): Promise<void> {
  if (insideTauri()) await invoke("cancel_text_task");
}

/** Progress of the running task, from 0 to 1. */
export function onTextProgress(handler: (fraction: number) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<number>("text-progress", (event) => handler(event.payload));
}

export function onTextFinished(handler: (event: TextFinished) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<TextFinished>("text-finished", (event) => handler(event.payload));
}

export async function listNotes(itemId: number): Promise<Note[]> {
  return insideTauri() ? invoke<Note[]>("list_notes", { itemId }) : [];
}

export async function deleteNote(noteId: number): Promise<void> {
  if (insideTauri()) await invoke("delete_note", { noteId });
}
