import { jobs } from "./jobs.svelte";
import { models } from "./models.svelte";
import { settings } from "./settings.svelte";

/** Files chosen for transcription and how they will be transcribed, until the user starts the
 *  batch. Lives for the session, so the settings are remembered between batches. */
class BatchStore {
  files = $state<string[]>([]);
  translate = $state(false);
  // What the person changed for this batch; until then the saved defaults apply.
  #language = $state<string | null>(null);
  #identify = $state<boolean | null>(null);
  #chosenModel = $state<string | null>(null);
  readonly speechModels = $derived(models.installed.filter((m) => m.kind === "stt"));
  readonly model = $derived(
    this.speechModels.find((m) => m.id === (this.#chosenModel ?? settings.value.model)) ??
      this.speechModels[0] ??
      null,
  );
  readonly canTranslate = $derived(this.model?.can_translate ?? true);
  readonly speakerModel = $derived(models.installed.find((m) => m.kind === "diarization") ?? null);

  get language(): string {
    return this.#language ?? settings.value.language ?? "";
  }

  set language(code: string) {
    this.#language = code;
  }

  get identifySpeakers(): boolean {
    return this.#identify ?? settings.value.identify_speakers;
  }

  set identifySpeakers(on: boolean) {
    this.#identify = on;
  }

  chooseModel(id: string): void {
    this.#chosenModel = id;
  }

  /** Adds files to the batch, skipping ones already in it. */
  add(paths: string[]): void {
    this.files.push(...paths.filter((path) => !this.files.includes(path)));
  }

  remove(path: string): void {
    this.files = this.files.filter((file) => file !== path);
  }

  clear(): void {
    this.files = [];
  }

  /** Queues the batch. The files stay in the batch if the queue rejects them, so the user can fix
   *  the problem and try again. */
  async start(): Promise<void> {
    const model = this.model;
    if (!model || this.files.length === 0) return;
    const settings = {
      model: model.id,
      language: this.language || null,
      translate: this.translate && this.canTranslate,
      identifySpeakers: this.identifySpeakers && this.speakerModel !== null,
    };
    await jobs.add(settings, [...this.files]);
    if (jobs.error === null) this.clear();
  }
}

export const batch = new BatchStore();
