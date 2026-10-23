import {
  cancelInstall,
  installModel,
  listModels,
  onModelFinished,
  onModelProgress,
  removeModel,
} from "./api";
import type { FinishedEvent, ModelStatus, ProgressEvent } from "./types";

const messageOf = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** Model catalog with install state and live download progress. Lives for the whole session, so
 *  an install keeps reporting while another page is open. */
class ModelsStore {
  items = $state<ModelStatus[] | null>(null);
  progress = $state<Record<string, ProgressEvent>>({});
  errors = $state<Record<string, string>>({});
  readonly installed = $derived((this.items ?? []).filter((model) => model.installed));
  readonly installedBytes = $derived(
    this.installed.reduce((total, model) => total + model.size_bytes, 0),
  );
  #started = false;

  /** Loads the catalog and starts listening for install events. Safe to call more than once. */
  async start(): Promise<void> {
    if (this.#started) return;
    this.#started = true;
    await Promise.all([
      this.load(),
      onModelProgress((event) => (this.progress[event.id] = event)),
      onModelFinished((event) => void this.#finished(event)),
    ]);
  }

  async load(): Promise<void> {
    this.items = await listModels();
  }

  async install(model: ModelStatus): Promise<void> {
    delete this.errors[model.id];
    this.progress[model.id] = { id: model.id, downloaded: 0, total: model.size_bytes };
    try {
      await installModel(model.id);
    } catch (error) {
      delete this.progress[model.id];
      this.errors[model.id] = messageOf(error);
    }
  }

  async cancel(id: string): Promise<void> {
    await cancelInstall(id);
  }

  async remove(id: string): Promise<void> {
    delete this.errors[id];
    try {
      await removeModel(id);
    } catch (error) {
      this.errors[id] = messageOf(error);
    }
    await this.load();
  }

  async #finished(event: FinishedEvent): Promise<void> {
    delete this.progress[event.id];
    if (event.outcome === "failed" && event.message) this.errors[event.id] = event.message;
    await this.load();
  }
}

export const models = new ModelsStore();
