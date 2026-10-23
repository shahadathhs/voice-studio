import { t } from "./i18n.svelte";
import {
  cancelTextTask,
  onTextFinished,
  onTextProgress,
  startTextTask,
  textLanguages,
} from "./api";
import { languageName } from "./languages";
import { library } from "./library.svelte";
import { models } from "./models.svelte";
import type { TextFinished, TextTask } from "./types";

const messageOf = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** Summaries and translations by a language model. Lives for the session, so a long task keeps
 *  going while the dialog is closed. */
class Assistant {
  languages = $state<string[]>([]);
  language = $state("en");
  status = $state<"idle" | "working">("idle");
  task = $state<TextTask | null>(null);
  fraction = $state(0);
  summary = $state<string | null>(null);
  /** The library item of the last finished translation. */
  translated = $state<number | null>(null);
  error = $state<string | null>(null);
  #started = false;
  #chosenModel = $state<string | null>(null);
  /** The installed language models, largest first. */
  readonly installed = $derived(
    models.installed
      .filter((m) => m.kind === "text")
      .toSorted((a, b) => b.size_bytes - a.size_bytes),
  );
  readonly model = $derived(
    this.installed.find((m) => m.id === this.#chosenModel) ?? this.installed[0] ?? null,
  );
  readonly options = $derived(
    this.languages
      .map((code) => ({ code, label: languageName(code) }))
      .toSorted((a, b) => a.label.localeCompare(b.label)),
  );

  async start(): Promise<void> {
    if (this.#started) return;
    this.#started = true;
    this.languages = await textLanguages();
    await Promise.all([
      onTextProgress((fraction) => (this.fraction = fraction)),
      onTextFinished((event) => this.#finished(event)),
    ]);
  }

  chooseModel(id: string): void {
    this.#chosenModel = id;
  }

  async run(itemId: number, task: TextTask): Promise<void> {
    if (!this.model || this.status === "working") return;
    this.error = null;
    this.summary = null;
    this.translated = null;
    this.fraction = 0;
    this.task = task;
    this.status = "working";
    try {
      await startTextTask(itemId, task, this.language, this.model.id);
    } catch (failure) {
      this.status = "idle";
      this.error = messageOf(failure);
    }
  }

  async cancel(): Promise<void> {
    await cancelTextTask();
  }

  /** Forget the last result, e.g. when another transcript is opened. */
  clear(): void {
    if (this.status === "working") return;
    this.summary = null;
    this.translated = null;
    this.error = null;
  }

  #finished(event: TextFinished): void {
    this.status = "idle";
    if (event.outcome === "failed") this.error = event.message ?? t("assist.failed");
    if (event.outcome !== "done") return;
    this.summary = event.text;
    this.translated = event.item_id;
    if (event.item_id !== null) void library.load();
  }
}

export const assistant = new Assistant();
