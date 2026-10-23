import {
  cancelRecording,
  discardRecording,
  listInputDevices,
  onRecordingCaptions,
  onRecordingLevel,
  saveRecordingAs,
  startRecording,
  stopRecording,
} from "./api-record";
import { chooseSavePath } from "./api";
import { batch } from "./batch.svelte";
import { jobs } from "./jobs.svelte";
import { nav } from "./nav.svelte";
import type { InputDevice, RecordingResult } from "./types";

const messageOf = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

/** The Record screen: the microphone, the recording in progress and the one just finished.
 *  Lives for the session, so a recording keeps going while another page is open. */
class RecordStore {
  devices = $state<InputDevice[]>([]);
  /** Live captions while recording (needs an installed speech model). */
  live = $state(true);
  status = $state<"idle" | "recording" | "saving" | "done">("idle");
  level = $state(0);
  elapsedMs = $state(0);
  final = $state("");
  draft = $state("");
  error = $state<string | null>(null);
  result = $state<RecordingResult | null>(null);
  #chosen = $state<string | null>(null);
  #started = false;
  readonly device = $derived(
    this.devices.find((d) => d.name === this.#chosen) ??
      this.devices.find((d) => d.is_default) ??
      this.devices[0] ??
      null,
  );
  readonly canCaption = $derived(batch.model !== null);

  async start(): Promise<void> {
    if (this.#started) return;
    this.#started = true;
    await Promise.all([
      this.loadDevices(),
      onRecordingLevel((event) => this.#heard(event.level, event.elapsed_ms, event.error)),
      onRecordingCaptions((event) => {
        this.final = event.final;
        this.draft = event.draft;
        if (event.error) this.error = event.error;
      }),
    ]);
  }

  async loadDevices(): Promise<void> {
    try {
      this.devices = await listInputDevices();
    } catch (failure) {
      this.devices = [];
      this.error = messageOf(failure);
    }
  }

  chooseDevice(name: string): void {
    this.#chosen = name;
  }

  #heard(level: number, elapsedMs: number, error: string | null): void {
    this.level = level;
    this.elapsedMs = elapsedMs;
    if (error && this.status === "recording") {
      this.error = error;
      void this.stop();
    }
  }

  async begin(): Promise<void> {
    this.error = null;
    this.final = "";
    this.draft = "";
    this.level = 0;
    this.elapsedMs = 0;
    this.result = null;
    try {
      await startRecording(
        this.device?.name ?? null,
        this.live && this.canCaption,
        batch.model?.id ?? null,
        batch.language || null,
      );
      this.status = "recording";
    } catch (failure) {
      this.error = messageOf(failure);
    }
  }

  async stop(): Promise<void> {
    if (this.status !== "recording") return;
    this.status = "saving";
    this.level = 0;
    try {
      this.result = await stopRecording();
      this.status = this.result ? "done" : "idle";
    } catch (failure) {
      this.error = messageOf(failure);
      this.status = "idle";
    }
  }

  /** Queues the recording for transcription and shows the queue. */
  async transcribe(): Promise<void> {
    const model = batch.model;
    if (!this.result || !model) return;
    await jobs.add(
      {
        model: model.id,
        language: batch.language || null,
        translate: false,
        identifySpeakers: false,
      },
      [this.result.path],
    );
    if (jobs.error !== null) {
      this.error = jobs.error;
      return;
    }
    this.reset();
    nav.go("transcribe");
  }

  async saveAs(): Promise<void> {
    if (!this.result) return;
    try {
      const destination = await chooseSavePath("recording", ["wav"]);
      if (destination) await saveRecordingAs(this.result.path, destination);
    } catch (failure) {
      this.error = messageOf(failure);
    }
  }

  async discard(): Promise<void> {
    if (this.status === "recording") await cancelRecording();
    else if (this.result) await discardRecording(this.result.path).catch(() => undefined);
    this.reset();
  }

  reset(): void {
    this.status = "idle";
    this.result = null;
    this.final = "";
    this.draft = "";
    this.elapsedMs = 0;
    this.level = 0;
  }
}

export const record = new RecordStore();
