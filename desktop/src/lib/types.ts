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
  can_translate: boolean;
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

export type JobState = "waiting" | "running" | "paused" | "done" | "failed" | "cancelled";

/** Event `job-updated` and the rows of `list_jobs`. Same fields as `JobInfo` in vtt-core. */
export interface JobInfo {
  id: number;
  file: string;
  title: string;
  model: string;
  state: JobState;
  processed_ms: number;
  total_ms: number;
  error: string | null;
  item_id: number | null;
}

/** One stored transcript, from `list_library`. Same fields as `Item` in vtt-core. */
export interface LibraryItem {
  id: number;
  title: string;
  /** Seconds since the Unix epoch. */
  created_at: number;
  duration_ms: number;
  source_path: string | null;
  model: string | null;
  language: string | null;
  segment_count: number;
  speaker_count: number;
}

/** A library item that matched a search, with the matching line. */
export interface SearchHit {
  item: LibraryItem;
  start_ms: number;
  /** The line with matches wrapped in `[` and `]`. */
  snippet: string;
}

export interface Segment {
  start_ms: number;
  end_ms: number;
  speaker?: string;
  text: string;
}

export interface Transcript {
  segments: Segment[];
  language: string | null;
  duration_ms: number;
  model: string | null;
}

/** Speaking time of one speaker. */
export interface SpeakerShare {
  name: string;
  talk_ms: number;
}

/** Result of `get_transcript`. */
export interface TranscriptView {
  item: LibraryItem;
  transcript: Transcript;
  word_count: number;
  /** Longest speaker first. */
  speakers: SpeakerShare[];
}

export type ExportFormat = "txt" | "srt" | "vtt" | "json" | "md" | "docx";

/** Argument of `render_export` and `save_export`; `ExportSettings` in desktop/src-tauri. */
export interface ExportSettings {
  speakerNames: boolean;
  timestamps: boolean;
  maxCharsPerLine: number;
  linesPerCue: number;
}

/** A voice from `list_voices`. Same fields as `VoiceInfo` in vtt-core. */
export interface VoiceInfo {
  id: string;
  name: string;
  description: string;
}

/** Event `speech-progress`: sentences spoken so far. */
export interface SpeechProgress {
  done: number;
  total: number;
}

/** Event `speech-finished`, emitted once when speaking ends. */
export interface SpeechFinished {
  outcome: "done" | "cancelled" | "failed";
  path: string | null;
  duration_ms: number;
  message: string | null;
}

export type Device = "auto" | "cpu" | "gpu";

/** The saved defaults, from `get_settings`. Same fields as `Settings` in vtt-core. */
export interface Settings {
  /** Catalog id of the transcription model to start with; null picks one automatically. */
  model: string | null;
  /** Spoken language code; null detects it for each file. */
  language: string | null;
  device: Device;
  identify_speakers: boolean;
}
