<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import Notice from "../components/Notice.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";
  import SpeechPlayer from "../components/SpeechPlayer.svelte";
  import VoiceList from "../components/VoiceList.svelte";
  import { percent } from "../lib/format";
  import { languageName } from "../lib/languages";
  import { models } from "../lib/models.svelte";
  import { nav } from "../lib/nav.svelte";
  import { speech } from "../lib/speech.svelte";

  const installed = $derived(models.installed.some((model) => model.kind === "tts"));
  const working = $derived(speech.status === "working");
  const translating = $derived(working && speech.progress?.total === 0);
  const hasTextModel = $derived(models.installed.some((model) => model.kind === "text"));
  const done = $derived(speech.progress ? percent(speech.progress.done, speech.progress.total) : 0);

  // Voices appear once the voice model has finished installing.
  $effect(() => {
    if (installed) void speech.loadVoices();
  });
  $effect(() => () => speech.stopPlayback());
</script>

<section>
  <PageHeader title={t("speak.title")} subtitle={t("speak.subtitle")}>
    {#snippet actions()}
      <button class="btn" onclick={() => speech.openFile()} disabled={working}>
        <Icon name="folder" />{t("speak.openFile")}
      </button>
    {/snippet}
  </PageHeader>

  {#if models.items !== null && !installed}
    <EmptyState title={t("speak.needModel.title")} text={t("speak.needModel.text")} />
    <div class="center">
      <button class="btn primary" onclick={() => nav.go("models")}>{t("speak.openModels")}</button>
    </div>
  {:else}
    <div class="layout">
      <div class="main">
        <div class="editor">
          <textarea
            aria-label={t("speak.textLabel")}
            placeholder={t("speak.placeholder")}
            bind:value={speech.text}
            disabled={working}></textarea>
          <div class="stats muted">
            <span>{t("speak.count", { characters: speech.text.length, words: speech.words })}</span>
            {#if translating}
              <span>{t("speak.translating")}</span>
            {:else if speech.progress && working}
              <span
                >{t("speak.sentence", {
                  done: speech.progress.done,
                  total: speech.progress.total,
                })}</span
              >
            {/if}
          </div>
        </div>
        {#if working}
          <ProgressBar value={done} label={t("speak.speaking")} />
        {/if}
        {#if speech.error}<p class="error" role="alert">{speech.error}</p>{/if}
        {#if speech.status === "ready"}<SpeechPlayer />{/if}
        {#if speech.spoken}
          <div class="spoken">
            <div class="label">{t("speak.spoken")}</div>
            <p>{speech.spoken}</p>
          </div>
        {/if}
      </div>

      <aside>
        <h2>{t("speak.voice")}</h2>
        {#if speech.voices.length === 0}
          <p class="muted">{t("speak.loadingVoices")}</p>
        {:else}
          <VoiceList />
        {/if}

        {#if hasTextModel && speech.voice}
          <label class="check">
            <input type="checkbox" bind:checked={speech.translate} disabled={working} />
            {t("speak.translateFirst", { language: languageName(speech.voice.language || null) })}
          </label>
        {/if}

        <h2>{t("speak.speed")}</h2>
        <label class="speed">
          <input
            type="range"
            min="0.5"
            max="2"
            step="0.1"
            bind:value={speech.speed}
            aria-label={t("speak.speed")}
            disabled={working}
          />
          <span class="mono">{speech.speed.toFixed(1)}×</span>
        </label>

        {#if working}
          <button class="btn go" onclick={() => speech.cancel()}>{t("speak.cancel")}</button>
        {:else}
          <button
            class="btn primary go"
            onclick={() => speech.generate()}
            disabled={!speech.text.trim() || !speech.voice}
          >
            <Icon name="play" />{t("speak.generate")}
          </button>
        {/if}
      </aside>
    </div>
    <Notice>{t("speak.privacy")}</Notice>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 28px 36px;
  }

  .center {
    display: flex;
    justify-content: center;
  }

  .layout {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 28px;
    align-items: start;
  }

  .main {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }

  .editor {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface);
  }

  textarea {
    min-height: 320px;
    padding: 20px 22px;
    border: 0;
    border-radius: 10px 10px 0 0;
    outline: 0;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 15px;
    line-height: 1.6;
    resize: vertical;
    user-select: text;
  }

  .stats {
    display: flex;
    justify-content: space-between;
    padding: 10px 22px;
    border-top: 1px solid var(--line);
    font-size: 12px;
  }

  aside {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  aside h2:not(:first-child) {
    margin-top: 12px;
  }

  .speed {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .speed input {
    flex: 1;
    accent-color: var(--accent);
  }

  .go {
    justify-content: center;
    height: 40px;
    margin-top: 10px;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }

  .check {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    color: var(--text2);
  }

  input[type="checkbox"] {
    margin-top: 3px;
    accent-color: var(--accent);
  }

  .spoken {
    padding: 12px 14px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
  }

  .spoken p {
    margin-top: 4px;
    white-space: pre-wrap;
    user-select: text;
  }
</style>
