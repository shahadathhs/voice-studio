import { groupModels } from "./model-groups";
import { models } from "./models.svelte";
import type { ModelStatus } from "./types";

const STORAGE_KEY = "voice-studio-onboarding-skipped";

function readSkipped(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "yes";
  } catch {
    return false;
  }
}

/** The model to pre-select for transcription: the recommended one, else the smallest. */
function suggestedTranscriber(items: readonly ModelStatus[]): string | undefined {
  const entry = groupModels(items).find(({ group }) => group.kind === "stt");
  return entry?.group.recommended ?? entry?.models[0]?.id;
}

/**
 * First-run setup. It shows until a transcription model is installed or the person skips it, and
 * lets them pick what to download. Only the transcription model starts selected: the others are
 * optional extras they can add later.
 */
class OnboardingStore {
  skipped = $state(readSkipped());
  /** Ids the person has explicitly turned off or on; the suggestion applies until they touch it. */
  #choices = $state<Record<string, boolean>>({});

  readonly needed = $derived(
    models.items !== null && !this.skipped && !models.installed.some((m) => m.kind === "stt"),
  );
  readonly downloading = $derived(Object.keys(models.progress).length > 0);

  isSelected(model: ModelStatus): boolean {
    return this.#choices[model.id] ?? model.id === suggestedTranscriber(models.items ?? []);
  }

  toggle(model: ModelStatus): void {
    this.#choices[model.id] = !this.isSelected(model);
  }

  /** Bytes still to download for the selection. */
  readonly remainingBytes = $derived(
    (models.items ?? [])
      .filter((model) => !model.installed && this.isSelected(model))
      .reduce((total, model) => total + model.size_bytes, 0),
  );

  /** True when a transcription model is part of the selection or already installed. */
  readonly canStart = $derived(
    (models.items ?? []).some(
      (model) => model.kind === "stt" && (model.installed || this.isSelected(model)),
    ) && this.remainingBytes > 0,
  );

  async start(): Promise<void> {
    const wanted = (models.items ?? []).filter((m) => !m.installed && this.isSelected(m));
    await Promise.all(wanted.map((model) => models.install(model)));
  }

  skip(): void {
    this.skipped = true;
    try {
      localStorage.setItem(STORAGE_KEY, "yes");
    } catch {
      // Skipping still applies for this session.
    }
  }
}

export const onboarding = new OnboardingStore();
