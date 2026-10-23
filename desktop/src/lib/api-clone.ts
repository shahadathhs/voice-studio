import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { insideTauri } from "./api";
import type { VoiceInfo } from "./types";

/** Asks the user for a recording to clone a voice from; null when they cancel. */
export async function chooseRecording(): Promise<string | null> {
  if (!insideTauri()) return null;
  const picked = await open({
    multiple: false,
    filters: [{ name: "Audio", extensions: ["wav", "mp3", "m4a", "aac", "flac", "ogg"] }],
  });
  return picked ?? null;
}

/** Makes a cloned voice from a recording. `consent` must be true: the person's confirmation. */
export async function cloneVoice(
  name: string,
  path: string,
  consent: boolean,
): Promise<VoiceInfo | null> {
  return insideTauri() ? invoke<VoiceInfo>("clone_voice", { name, path, consent }) : null;
}

export async function deleteClone(id: string): Promise<void> {
  if (insideTauri()) await invoke("delete_clone", { id });
}
