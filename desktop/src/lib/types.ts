/** Mirrors the serde output of the Tauri commands in desktop/src-tauri/src/commands.rs. */

export interface AppInfo {
  name: string;
  version: string;
  coreVersion: string;
  dataDir: string;
}

export type ModelKind = "stt" | "tts" | "diarization" | "vad";

/** Catalog data plus install state, from `list_models`. Same fields as `ModelStatus` in vtt-core. */
export interface ModelStatus {
  id: string;
  name: string;
  kind: ModelKind;
  description: string;
  languages: string[];
  license: string;
  size_bytes: number;
  installed: boolean;
}

/** Event `model-progress`, emitted while an install runs. */
export interface ProgressEvent {
  id: string;
  downloaded: number;
  total: number;
}

/** Event `model-finished`, emitted once when an install ends. */
export interface FinishedEvent {
  id: string;
  outcome: "installed" | "cancelled" | "failed";
  message: string | null;
}
