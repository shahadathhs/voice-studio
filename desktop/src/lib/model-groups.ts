import type { MessageKey } from "../locales/en";
import type { PageId } from "./nav.svelte";
import type { ModelKind, ModelStatus } from "./types";

/** A section of the Models page: the models that serve one purpose, and what that purpose is. */
export interface ModelGroup {
  kind: ModelKind;
  title: MessageKey;
  purpose: MessageKey;
  /** The screen that uses these models. */
  page: PageId;
  pageLabel: MessageKey;
  /** Model to suggest to someone who does not know which to pick. */
  recommended?: string;
}

const GROUPS: readonly ModelGroup[] = [
  {
    kind: "stt",
    title: "models.group.stt.title",
    purpose: "models.group.stt.purpose",
    page: "transcribe",
    pageLabel: "models.goTranscribe",
    recommended: "whisper-small",
  },
  {
    kind: "diarization",
    title: "models.group.diarization.title",
    purpose: "models.group.diarization.purpose",
    page: "transcribe",
    pageLabel: "models.goTranscribe",
  },
  {
    kind: "tts",
    title: "models.group.tts.title",
    purpose: "models.group.tts.purpose",
    page: "speak",
    pageLabel: "models.goSpeak",
  },
  {
    kind: "text",
    title: "models.group.text.title",
    purpose: "models.group.text.purpose",
    page: "library",
    pageLabel: "models.goLibrary",
    recommended: "qwen2.5-1.5b",
  },
  {
    kind: "clone",
    title: "models.group.clone.title",
    purpose: "models.group.clone.purpose",
    page: "speak",
    pageLabel: "models.goSpeak",
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
