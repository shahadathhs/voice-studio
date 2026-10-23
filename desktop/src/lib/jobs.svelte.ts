import {
  addJobs,
  cancelJob,
  clearFinishedJobs,
  listJobs,
  moveJob,
  onJobUpdated,
  removeJob,
  setQueuePaused,
} from "./api";
import { library } from "./library.svelte";
import type { JobInfo } from "./types";

const messageOf = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** How a batch of files is transcribed. */
export interface JobSettings {
  model: string;
  language: string | null;
  translate: boolean;
  identifySpeakers: boolean;
}

const isActive = (job: JobInfo): boolean => job.state === "waiting" || job.state === "running";

/** The transcription queue. Lives for the whole session, so jobs keep reporting while another
 *  page is open. */
class JobsStore {
  items = $state<JobInfo[]>([]);
  paused = $state(false);
  error = $state<string | null>(null);
  readonly active = $derived(this.items.filter(isActive));
  #started = false;

  async start(): Promise<void> {
    if (this.#started) return;
    this.#started = true;
    await Promise.all([this.#reload(), onJobUpdated((job) => this.#update(job))]);
  }

  /** Queues files with the given settings. A rejected batch shows its reason in `error`. */
  async add(settings: JobSettings, files: string[]): Promise<void> {
    this.error = null;
    try {
      await addJobs(
        files,
        settings.model,
        settings.language,
        settings.translate,
        settings.identifySpeakers,
      );
      await this.#reload();
    } catch (failure) {
      this.error = messageOf(failure);
    }
  }

  async cancel(id: number): Promise<void> {
    await cancelJob(id);
  }

  async remove(id: number): Promise<void> {
    await removeJob(id);
    await this.#reload();
  }

  async clearFinished(): Promise<void> {
    await clearFinishedJobs();
    await this.#reload();
  }

  /** Moves a waiting job one place up the queue. */
  async moveUp(id: number): Promise<void> {
    const index = this.items.findIndex((job) => job.id === id);
    if (index > 0) await moveJob(id, index - 1);
    await this.#reload();
  }

  async setPaused(paused: boolean): Promise<void> {
    await setQueuePaused(paused);
    this.paused = paused;
  }

  async #reload(): Promise<void> {
    this.items = await listJobs();
  }

  #update(job: JobInfo): void {
    const index = this.items.findIndex((existing) => existing.id === job.id);
    if (index >= 0) this.items[index] = job;
    else this.items.push(job);
    if (job.state === "done") void library.load();
  }
}

export const jobs = new JobsStore();
