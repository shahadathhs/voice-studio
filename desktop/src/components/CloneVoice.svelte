<script lang="ts">
  import { chooseRecording } from "../lib/api-clone";
  import { t } from "../lib/i18n.svelte";
  import { models } from "../lib/models.svelte";
  import { nav } from "../lib/nav.svelte";
  import { speech } from "../lib/speech.svelte";

  let name = $state("");
  let file = $state<string | null>(null);
  let consent = $state(false);
  let working = $state(false);
  let error = $state<string | null>(null);

  const installed = $derived(models.installed.some((model) => model.kind === "clone"));
  const ready = $derived(name.trim().length > 0 && file !== null && consent);
  const busy = $derived(working || speech.status === "working");
  const fileName = $derived(file ? (file.split(/[\\/]/).pop() ?? file) : t("clone.noFile"));

  async function pick(): Promise<void> {
    file = (await chooseRecording()) ?? file;
  }

  async function make(): Promise<void> {
    if (!ready || !file) return;
    working = true;
    error = null;
    try {
      await speech.cloneVoice(name, file, consent);
      name = "";
      file = null;
      consent = false;
    } catch (failure) {
      error = failure instanceof Error ? failure.message : String(failure);
    } finally {
      working = false;
    }
  }
</script>

<div class="clone">
  <h2>{t("clone.title")}</h2>
  <p class="muted">{t("clone.hint")}</p>

  {#if !installed}
    <p class="muted">{t("clone.needModel")}</p>
    <button class="btn" onclick={() => nav.go("models")}>{t("clone.openModels")}</button>
  {:else}
    <div class="group">
      <span>{t("clone.file")}</span>
      <div class="file">
        <button class="btn" onclick={pick} disabled={busy}>{t("clone.chooseFile")}</button>
        <span class="muted name" title={file ?? ""}>{fileName}</span>
      </div>
    </div>

    <label class="group">
      <span>{t("clone.name")}</span>
      <input
        type="text"
        maxlength="40"
        placeholder={t("clone.namePlaceholder")}
        bind:value={name}
        disabled={busy}
      />
    </label>

    <label class="consent">
      <input type="checkbox" bind:checked={consent} disabled={busy} />
      {t("clone.consent")}
    </label>

    <p class="muted">{t("clone.disclosure")} {t("clone.slow")}</p>
    {#if working}
      <p class="muted" role="status">{t("clone.working")}</p>
    {:else if !ready}
      <p class="muted">{t("clone.needAll")}</p>
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}

    <button class="btn primary" onclick={make} disabled={busy || !ready}>
      {t("clone.make")}
    </button>
  {/if}
</div>

<style>
  .clone {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 12px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--text2);
    font-size: 12.5px;
  }

  .file {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  input[type="text"] {
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }

  .consent {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    color: var(--text);
  }

  .consent input {
    margin-top: 3px;
    accent-color: var(--accent);
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
