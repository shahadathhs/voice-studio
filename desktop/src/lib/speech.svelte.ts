import { t } from "./i18n.svelte";
import {
  cancelSpeech,
  chooseSavePath,
  chooseTextFile,
  listVoices,
  onSpeechFinished,
  onSpeechProgress,
  readTextFile,
  saveSpeech,
  speechUrl,
  startSpeech,
} from "./api";
import type { SpeechFinished, SpeechProgress, VoiceInfo } from "./types";

const MS = 1000;

const messageOf = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** The Speak screen: the text, voice and speed, the speech being generated and its playback.
 *  Lives for the session, so a long generation keeps going while another page is open. */
class SpeechStore {
  text = $state("");
  speed = $state(1);
  /** Translate the text into the voice's language before speaking it. */
  translate = $state(false);
  /** What was spoken, when it was translated first. */
  spoken = $state<string | null>(null);
  voices = $state<VoiceInfo[]>([]);
  status = $state<"idle" | "working" | "ready">("idle");
  progress = $state<SpeechProgress | null>(null);
  error = $state<string | null>(null);
  durationMs = $state(0);
  positionMs = $state(0);
  playing = $state(false);
  #chosenVoice = $state<string | null>(null);
  #audio: HTMLAudioElement | null = null;
  #started = false;
  readonly voice = $derived(
    this.voices.find((v) => v.id === this.#chosenVoice) ?? this.voices[0] ?? null,
  );
  readonly words = $derived(this.text.split(/\s+/).filter(Boolean).length);

  async start(): Promise<void> {
    if (this.#started) return;
    this.#started = true;
    await Promise.all([
      this.loadVoices(),
      onSpeechProgress((event) => (this.progress = event)),
      onSpeechFinished((event) => this.#finished(event)),
    ]);
  }

  async loadVoices(): Promise<void> {
    this.voices = await listVoices();
  }

  chooseVoice(id: string): void {
    this.#chosenVoice = id;
  }

  /** Fills the text box from a text file the user picks. */
  async openFile(): Promise<void> {
    this.error = null;
    try {
      const path = await chooseTextFile();
      if (path) this.text = await readTextFile(path);
    } catch (failure) {
      this.error = messageOf(failure);
    }
  }

  async generate(): Promise<void> {
    if (!this.voice || this.status === "working") return;
    this.error = null;
    this.#discardAudio();
    this.spoken = null;
    this.status = "working";
    this.progress = null;
    try {
      await startSpeech(this.text, this.voice.id, this.speed, this.translate);
    } catch (failure) {
      this.status = "idle";
      this.error = messageOf(failure);
    }
  }

  async cancel(): Promise<void> {
    await cancelSpeech();
  }

  /** Plays or pauses the generated speech. */
  async togglePlay(): Promise<void> {
    if (!this.#audio) return;
    if (this.#audio.paused) await this.#audio.play();
    else this.#audio.pause();
  }

  seek(ms: number): void {
    if (this.#audio) this.#audio.currentTime = ms / MS;
  }

  stopPlayback(): void {
    this.#audio?.pause();
  }

  async save(): Promise<void> {
    this.error = null;
    try {
      const path = await chooseSavePath("speech", ["wav", "flac"]);
      if (path) await saveSpeech(path);
    } catch (failure) {
      this.error = messageOf(failure);
    }
  }

  #discardAudio(): void {
    this.#audio?.pause();
    this.#audio = null;
    this.playing = false;
    this.positionMs = 0;
  }

  #finished(event: SpeechFinished): void {
    if (event.outcome === "failed") this.error = event.message ?? t("speak.failed");
    this.spoken = event.text;
    if (event.outcome !== "done" || !event.path) {
      this.status = "idle";
      return;
    }
    const audio = new Audio(speechUrl(event.path));
    audio.addEventListener("play", () => (this.playing = true));
    audio.addEventListener("pause", () => (this.playing = false));
    audio.addEventListener("timeupdate", () => (this.positionMs = audio.currentTime * MS));
    audio.addEventListener("error", () => (this.error = t("player.cannotPlay")));
    this.#audio = audio;
    this.durationMs = event.duration_ms;
    this.status = "ready";
    void audio.play().catch(() => (this.playing = false));
  }
}

export const speech = new SpeechStore();
