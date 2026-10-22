<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import Notice from "../components/Notice.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";
  import SpeechPlayer from "../components/SpeechPlayer.svelte";
  import VoiceList from "../components/VoiceList.svelte";
  import { percent } from "../lib/format";
  import { models } from "../lib/models.svelte";
  import { nav } from "../lib/nav.svelte";
  import { speech } from "../lib/speech.svelte";

  const installed = $derived(models.installed.some((model) => model.kind === "tts"));
  const working = $derived(speech.status === "working");
  const done = $derived(speech.progress ? percent(speech.progress.done, speech.progress.total) : 0);

  // Voices appear once the voice model has finished installing.
  $effect(() => {
    if (installed) void speech.loadVoices();
  });
  $effect(() => () => speech.stopPlayback());
</script>

<section>
  <PageHeader title="Speak" subtitle="Type or paste text, pick a voice, and listen or save it.">
    {#snippet actions()}
      <button class="btn" onclick={() => speech.openFile()} disabled={working}>
        <Icon name="folder" />Open text file…
      </button>
    {/snippet}
  </PageHeader>

  {#if models.items !== null && !installed}
    <EmptyState
      title="Install the voice model first"
      text="Voices are downloaded once (about 100 MB) and then work offline."
    />
    <div class="center">
      <button class="btn primary" onclick={() => nav.go("models")}>Open Models</button>
    </div>
  {:else}
    <div class="layout">
      <div class="main">
        <div class="editor">
          <textarea
            aria-label="Text to speak"
            placeholder="Type or paste the text you want to hear."
            bind:value={speech.text}
            disabled={working}></textarea>
          <div class="stats muted">
            <span>{speech.text.length} characters · {speech.words} words</span>
            {#if speech.progress && working}
              <span>Sentence {speech.progress.done} of {speech.progress.total}</span>
            {/if}
          </div>
        </div>
        {#if working}
          <ProgressBar value={done} label="Speaking" />
        {/if}
        {#if speech.error}<p class="error" role="alert">{speech.error}</p>{/if}
        {#if speech.status === "ready"}<SpeechPlayer />{/if}
      </div>

      <aside>
        <h2>Voice</h2>
        {#if speech.voices.length === 0}
          <p class="muted">Loading voices…</p>
        {:else}
          <VoiceList />
        {/if}

        <h2>Speed</h2>
        <label class="speed">
          <input
            type="range"
            min="0.5"
            max="2"
            step="0.1"
            bind:value={speech.speed}
            aria-label="Speed"
            disabled={working}
          />
          <span class="mono">{speech.speed.toFixed(1)}×</span>
        </label>

        {#if working}
          <button class="btn go" onclick={() => speech.cancel()}>Cancel</button>
        {:else}
          <button
            class="btn primary go"
            onclick={() => speech.generate()}
            disabled={!speech.text.trim() || !speech.voice}
          >
            <Icon name="play" />Generate &amp; play
          </button>
        {/if}
      </aside>
    </div>
    <Notice>Speech is made on this computer. Nothing is sent anywhere.</Notice>
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
</style>
