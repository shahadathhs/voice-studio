<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { chooseSavePath, renderExport, saveExport } from "../lib/api";
  import type { MessageKey } from "../locales/en";
  import type { ExportFormat, ExportSettings } from "../lib/types";
  import Icon from "./Icon.svelte";
  import Segmented, { type SegmentOption } from "./Segmented.svelte";

  let { itemId, title, onclose }: { itemId: number; title: string; onclose: () => void } = $props();

  const FORMATS: readonly SegmentOption<ExportFormat>[] = [
    { value: "txt", label: "TXT" },
    { value: "srt", label: "SRT" },
    { value: "vtt", label: "VTT" },
    { value: "json", label: "JSON" },
    { value: "md", label: "Markdown" },
  ];
  const DESCRIPTIONS: Record<ExportFormat, MessageKey> = {
    txt: "export.txt",
    srt: "export.srt",
    vtt: "export.vtt",
    json: "export.json",
    md: "export.md",
  };
  const PREVIEW_CHARS = 700;

  let format = $state<ExportFormat>("srt");
  let settings = $state<ExportSettings>({
    speakerNames: true,
    timestamps: false,
    maxCharsPerLine: 42,
    linesPerCue: 2,
  });
  let preview = $state("");
  let error = $state<string | null>(null);
  let dialog: HTMLDialogElement | undefined = $state();

  const subtitles = $derived(format === "srt" || format === "vtt");

  $effect(() => dialog?.showModal());

  $effect(() => {
    const wanted = { ...settings };
    renderExport(itemId, format, wanted)
      .then((text) => (preview = text.slice(0, PREVIEW_CHARS)))
      .catch((failure: unknown) => (error = String(failure)));
  });

  async function save(): Promise<void> {
    error = null;
    try {
      const path = await chooseSavePath(title, format);
      if (!path) return;
      await saveExport(itemId, format, { ...settings }, path);
      onclose();
    } catch (failure) {
      error = String(failure);
    }
  }
</script>

<dialog bind:this={dialog} aria-labelledby="export-title" {onclose} oncancel={onclose}>
  <header>
    <h2 id="export-title">{t("export.title")}</h2>
    <button class="icon" aria-label={t("common.close")} onclick={onclose}><Icon name="x" /></button>
  </header>
  <div class="body">
    <div>
      <div class="label">{t("export.format")}</div>
      <Segmented
        options={FORMATS}
        value={format}
        label={t("export.format")}
        onchange={(f) => (format = f)}
      />
      <p class="muted hint">{t(DESCRIPTIONS[format])}</p>
    </div>
    <label class="check">
      <input type="checkbox" bind:checked={settings.speakerNames} />{t("export.speakerNames")}
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={settings.timestamps} />{t("export.timestamps")}
    </label>
    {#if subtitles}
      <label class="row">
        {t("export.maxChars")}
        <select class="field" bind:value={settings.maxCharsPerLine}>
          {#each [32, 42, 60] as count (count)}<option value={count}>{count}</option>{/each}
        </select>
      </label>
      <label class="row">
        {t("export.linesPerCue")}
        <select class="field" bind:value={settings.linesPerCue}>
          {#each [1, 2, 3] as count (count)}<option value={count}>{count}</option>{/each}
        </select>
      </label>
    {/if}
    <div>
      <div class="label">{t("export.preview")}</div>
      <pre>{preview}</pre>
    </div>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>
  <footer>
    <button class="btn" onclick={onclose}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={save}>{t("export.action")}</button>
  </footer>
</dialog>

<style>
  dialog {
    width: 560px;
    max-width: calc(100vw - 48px);
    margin: auto;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface);
    color: var(--text);
  }

  dialog::backdrop {
    background: rgb(0 0 0 / 0.45);
  }

  header,
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 16px 22px;
  }

  header {
    border-bottom: 1px solid var(--line);
  }

  footer {
    justify-content: flex-end;
    border-top: 1px solid var(--line);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 22px;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
  }

  .hint {
    margin-top: 6px;
  }

  .check,
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .row {
    justify-content: space-between;
  }

  input[type="checkbox"] {
    accent-color: var(--accent);
  }

  pre {
    max-height: 150px;
    padding: 12px;
    overflow: auto;
    border-radius: 8px;
    background: var(--bg);
    font: 12px/1.5 var(--mono);
    white-space: pre-wrap;
    user-select: text;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
