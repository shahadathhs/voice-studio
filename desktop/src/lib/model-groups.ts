import type { PageId } from "./nav.svelte";
import type { ModelKind, ModelStatus } from "./types";

/** A section of the Models page: the models that serve one purpose, and what that purpose is. */
export interface ModelGroup {
  kind: ModelKind;
  title: string;
  purpose: string;
  /** The screen that uses these models. */
  page: PageId;
  pageLabel: string;
  /** Model to suggest to someone who does not know which to pick. */
  recommended?: string;
}

const GROUPS: readonly ModelGroup[] = [
  {
    kind: "stt",
    title: "Transcription",
    purpose:
      "Turn audio and video into text. You need one of these; larger models are more accurate but slower. Not sure? Start with Whisper small.",
    page: "transcribe",
    pageLabel: "Go to Transcribe",
    recommended: "whisper-small",
  },
  {
    kind: "diarization",
    title: "Speaker labels",
    purpose:
      "Optional. Adds who said what to a transcript. It works alongside a transcription model, and you switch it on with Identify speakers.",
    page: "transcribe",
    pageLabel: "Go to Transcribe",
  },
  {
    kind: "tts",
    title: "Speech",
    purpose: "Read text aloud with natural voices. The Speak screen needs one of these.",
    page: "speak",
    pageLabel: "Go to Speak",
  },
];

/** The non-empty groups for the given models, each sorted from smallest to largest download. */
export function groupModels(
  items: readonly ModelStatus[],
): { group: ModelGroup; models: ModelStatus[] }[] {
  return GROUPS.map((group) => ({
    group,
    models: items
      .filter((model) => model.kind === group.kind)
      .toSorted((a, b) => a.size_bytes - b.size_bytes),
  })).filter((entry) => entry.models.length > 0);
}
