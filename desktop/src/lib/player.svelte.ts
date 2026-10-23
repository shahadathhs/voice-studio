import { t } from "./i18n.svelte";
import { playbackUrl } from "./api";

const MS = 1000;

/** Plays the original recording of one transcript: the whole file or one span of it. */
class Player {
  /** Which play button is active: a line number, or "all" for the whole file. */
  active = $state<number | "all" | null>(null);
  /** The library item the active button belongs to. */
  itemId = $state<number | null>(null);
  /** Playback position in milliseconds while playing. */
  position = $state(0);
  error = $state<string | null>(null);
  #audio: HTMLAudioElement | null = null;
  #loadedItem: number | null = null;
  #endMs: number | null = null;

  /** Whether the button `key` of item `itemId` is the one playing. */
  isActive(itemId: number, key: number | "all"): boolean {
    return this.active === key && this.itemId === itemId;
  }

  /** Plays from `startMs` to `endMs` (or to the end of the file). Pressing the active button
   *  again pauses. */
  async play(
    itemId: number,
    key: number | "all",
    startMs: number,
    endMs: number | null,
  ): Promise<void> {
    if (this.isActive(itemId, key)) return this.stop();
    return this.#start(itemId, key, startMs, endMs);
  }

  /** Jumps to `ms` in the whole recording: moves the playhead if it is playing, otherwise starts
   *  playing from there. */
  async seek(itemId: number, ms: number): Promise<void> {
    if (!this.isActive(itemId, "all") || !this.#audio) {
      return this.#start(itemId, "all", ms, null);
    }
    this.#audio.currentTime = ms / MS;
    this.position = ms;
  }

  async #start(
    itemId: number,
    key: number | "all",
    startMs: number,
    endMs: number | null,
  ): Promise<void> {
    this.error = null;
    try {
      const audio = await this.#load(itemId);
      this.#endMs = endMs;
      audio.currentTime = startMs / MS;
      this.active = key;
      this.itemId = itemId;
      await audio.play();
    } catch (failure) {
      this.active = null;
      this.error = failure instanceof Error ? failure.message : String(failure);
    }
  }

  stop(): void {
    this.#audio?.pause();
    this.active = null;
  }

  async #load(itemId: number): Promise<HTMLAudioElement> {
    if (this.#audio && this.#loadedItem === itemId) return this.#audio;
    const url = await playbackUrl(itemId);
    if (!url) throw new Error(t("player.onlyDesktop"));
    this.#audio?.pause();
    const audio = new Audio(url);
    audio.addEventListener("timeupdate", () => this.#tick(audio));
    audio.addEventListener("ended", () => (this.active = null));
    audio.addEventListener("error", () => this.#failed());
    this.#audio = audio;
    this.#loadedItem = itemId;
    return audio;
  }

  #tick(audio: HTMLAudioElement): void {
    this.position = audio.currentTime * MS;
    if (this.#endMs !== null && this.position >= this.#endMs) this.stop();
  }

  #failed(): void {
    this.active = null;
    this.error = t("player.cannotPlay");
    this.#audio = null;
    this.#loadedItem = null;
  }
}

export const player = new Player();
