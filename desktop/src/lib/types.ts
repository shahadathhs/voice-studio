/** Mirrors the serde output of the Tauri commands in desktop/src-tauri/src/commands.rs. */

export interface AppInfo {
  name: string;
  version: string;
  coreVersion: string;
  dataDir: string;
}

export type ModelKind = "stt" | "tts" | "diarization" | "text" | "vad" | "clone";

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
  /** The language code the voice speaks, e.g. `es`; empty if unknown. */
  language: string;
  /** A blend the user made; it can be deleted. */
  custom: boolean;
  /** A voice cloned from a recording; it can be deleted. */
  cloned: boolean;
}

/** One voice of a blend, with its weight (the weights are scaled to sum to 1). */
export interface VoicePart {
  voice: string;
  weight: number;
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
  /** What was spoken, when the text was translated first. */
  text: string | null;
}

/** Event `text-finished`, emitted once when a summary or translation ends. */
export interface TextFinished {
  outcome: "done" | "cancelled" | "failed";
  task: TextTask;
  /** The library item a translation was stored as. */
  item_id: number | null;
  message: string | null;
}

export type TextTask = "summary" | "chapters" | "actions" | "translation";

/** What a language model wrote about a transcript. Same fields as `Note` in vtt-core. */
export interface Note {
  id: number;
  item_id: number;
  kind: "summary" | "chapters" | "actions";
  language: string | null;
  model: string;
  /** For chapters, a JSON list of `Chapter`. */
  text: string;
  created_at: number;
}

export interface Chapter {
  start_ms: number;
  title: string;
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

/** A microphone. Same fields as `InputDevice` in vtt-core. */
export interface InputDevice {
  name: string;
  is_default: boolean;
}

/** The `recording-level` event, about 15 times a second while recording. */
export interface RecordingLevel {
  /** Recent peak, 0 to 1. */
  level: number;
  elapsed_ms: number;
  /** Set when the microphone stopped working. */
  error: string | null;
}

/** The `recording-captions` event: `final` will not change, `draft` may. */
export interface RecordingCaptions {
  final: string;
  draft: string;
  /** Set when live captions could not run. */
  error: string | null;
}

/** A finished recording saved in the data directory. */
export interface RecordingResult {
  path: string;
  duration_ms: number;
}

/** Event `dictation-state`: what the floating dictation window shows. */
export interface DictationState {
  state: "listening" | "working" | "done" | "failed";
  text: string | null;
  message: string | null;
}
