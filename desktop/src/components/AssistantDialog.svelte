<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { assistant } from "../lib/assistant.svelte";
  import { nav } from "../lib/nav.svelte";
  import { player } from "../lib/player.svelte";
  import Icon from "./Icon.svelte";
  import NoteCard from "./NoteCard.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { itemId, onclose }: { itemId: number; onclose: () => void } = $props();

  let dialog: HTMLDialogElement | undefined = $state();

  const working = $derived(assistant.status === "working");

  $effect(() => dialog?.showModal());
  $effect(() => {
    void assistant.start();
    void assistant.open(itemId);
  });

  function seek(ms: number): void {
    onclose();
    player.play(itemId, "all", ms, null);
  }

  function openModels(): void {
    onclose();
    nav.go("models");
  }

  function openTranslation(): void {
    const id = assistant.translated;
    onclose();
    if (id !== null) nav.open(id);
  }
</script>

<dialog bind:this={dialog} aria-labelledby="assist-title" {onclose} oncancel={onclose}>
  <header>
    <h2 id="assist-title">{t("assist.title")}</h2>
    <button class="icon" aria-label={t("common.close")} onclick={onclose}><Icon name="x" /></button>
  </header>
  <div class="body">
    {#if !assistant.model}
      <p>{t("assist.needModel")}</p>
      <div><button class="btn primary" onclick={openModels}>{t("speak.openModels")}</button></div>
    {:else}
      <label class="row">
        {t("assist.language")}
        <select class="field" bind:value={assistant.language} disabled={working}>
          {#each assistant.options as option (option.code)}
            <option value={option.code}>{option.label}</option>
          {/each}
        </select>
      </label>
      {#if assistant.installed.length > 1}
        <label class="row">
          {t("assist.model")}
          <select
            class="field"
            value={assistant.model.id}
            disabled={working}
            onchange={(event) => assistant.chooseModel(event.currentTarget.value)}
          >
            {#each assistant.installed as model (model.id)}
              <option value={model.id}>{model.name}</option>
            {/each}
          </select>
        </label>
      {/if}
      <div class="actions">
        <button
          class="btn primary"
          disabled={working}
          onclick={() => assistant.run(itemId, "summary")}>{t("assist.summarize")}</button
        >
        <button class="btn" disabled={working} onclick={() => assistant.run(itemId, "chapters")}
          >{t("assist.chapters")}</button
        >
        <button class="btn" disabled={working} onclick={() => assistant.run(itemId, "actions")}
          >{t("assist.actions")}</button
        >
        <button class="btn" disabled={working} onclick={() => assistant.run(itemId, "translation")}
          >{t("assist.translate")}</button
        >
        {#if working}
          <button class="btn" onclick={() => assistant.cancel()}>{t("common.cancel")}</button>
        {/if}
      </div>
      {#if working}
        <ProgressBar value={assistant.fraction * 100} label={t("assist.working")} />
      {/if}
      {#if assistant.error}<p class="error" role="alert">{assistant.error}</p>{/if}
      {#each assistant.notes as note (note.id)}
        <NoteCard {note} onseek={seek} ondelete={() => assistant.remove(note)} />
      {/each}
      {#if assistant.translated !== null}
        <section aria-label={t("assist.translation")}>
          <p>{t("assist.translationSaved")}</p>
          <button class="btn primary" onclick={openTranslation}
            >{t("assist.openTranslation")}</button
          >
        </section>
      {/if}
      <p class="muted hint">{t("assist.warning")}</p>
    {/if}
  </div>
</dialog>

<style>
  dialog {
    width: 600px;
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

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 16px 22px;
    border-bottom: 1px solid var(--line);
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

  .actions {
    flex-wrap: wrap;
  }

  .row,
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .row {
    justify-content: space-between;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
