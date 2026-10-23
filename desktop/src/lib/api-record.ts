import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { insideTauri, noop } from "./api";
import type { InputDevice, RecordingCaptions, RecordingLevel, RecordingResult } from "./types";

/** The microphones, the default one flagged. Empty in a browser, where there is no microphone. */
export async function listInputDevices(): Promise<InputDevice[]> {
  return insideTauri() ? invoke<InputDevice[]>("list_input_devices") : [];
}

/** Starts recording; follow it with `onRecordingLevel` / `onRecordingCaptions`. Rejects with a
 *  readable reason when there is no microphone or access was denied. */
export async function startRecording(
  device: string | null,
  live: boolean,
  model: string | null,
  language: string | null,
): Promise<void> {
  if (insideTauri()) await invoke("start_recording", { device, live, model, language });
}

/** Stops and saves; null in a browser, where nothing was recorded. */
export async function stopRecording(): Promise<RecordingResult | null> {
  return insideTauri() ? invoke<RecordingResult>("stop_recording") : null;
}

/** Stops and throws the audio away. */
export async function cancelRecording(): Promise<void> {
  if (insideTauri()) await invoke("cancel_recording");
}

/** Copies a finished recording to a file the user chose. */
export async function saveRecordingAs(file: string, destination: string): Promise<void> {
  if (insideTauri()) await invoke("save_recording_as", { file, destination });
}

export async function discardRecording(file: string): Promise<void> {
  if (insideTauri()) await invoke("discard_recording", { file });
}

export function onRecordingLevel(handler: (event: RecordingLevel) => void): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<RecordingLevel>("recording-level", (event) => handler(event.payload));
}

export function onRecordingCaptions(
  handler: (event: RecordingCaptions) => void,
): Promise<UnlistenFn> {
  if (!insideTauri()) return Promise.resolve(noop);
  return listen<RecordingCaptions>("recording-captions", (event) => handler(event.payload));
}
