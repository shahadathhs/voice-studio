import { playbackUrl } from "./api";

const MS = 1000;

/** Plays the original recording of one transcript: the whole file or one span of it. */
class Player {
  /** Which play button is active: a line number, or "all" for the whole file. */
  active = $state<number | "all" | null>(null);
  /** Playback position in milliseconds while playing. */
  position = $state(0);
  error = $state<string | null>(null);
  #audio: HTMLAudioElement | null = null;
  #itemId: number | null = null;
  #endMs: number | null = null;

  /** Plays from `startMs` to `endMs` (or to the end of the file). Pressing the active button
   *  again pauses. */
  async play(
    itemId: number,
    key: number | "all",
    startMs: number,
    endMs: number | null,
  ): Promise<void> {
    if (this.active === key) return this.stop();
    this.error = null;
    try {
      const audio = await this.#load(itemId);
      this.#endMs = endMs;
      audio.currentTime = startMs / MS;
      this.active = key;
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
    if (this.#audio && this.#itemId === itemId) return this.#audio;
    const url = await playbackUrl(itemId);
    if (!url) throw new Error("Playback is only available in the desktop app.");
    this.#audio?.pause();
    const audio = new Audio(url);
    audio.addEventListener("timeupdate", () => this.#tick(audio));
    audio.addEventListener("ended", () => (this.active = null));
    audio.addEventListener("error", () => this.#failed());
    this.#audio = audio;
    this.#itemId = itemId;
    return audio;
  }

  #tick(audio: HTMLAudioElement): void {
    this.position = audio.currentTime * MS;
    if (this.#endMs !== null && this.position >= this.#endMs) this.stop();
  }

  #failed(): void {
    this.active = null;
    this.error = "This audio format cannot be played here.";
    this.#audio = null;
    this.#itemId = null;
  }
}

export const player = new Player();
