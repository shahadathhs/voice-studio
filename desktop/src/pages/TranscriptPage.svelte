<script lang="ts">
  import ExportDialog from "../components/ExportDialog.svelte";
  import Icon from "../components/Icon.svelte";
  import SpeakerPanel from "../components/SpeakerPanel.svelte";
  import TranscriptLines from "../components/TranscriptLines.svelte";
  import { getTranscript, renameSpeaker, renderExport } from "../lib/api";
  import { formatClock } from "../lib/format";
  import { languageName } from "../lib/languages";
  import { library } from "../lib/library.svelte";
  import { player } from "../lib/player.svelte";
  import { nav } from "../lib/nav.svelte";
  import type { TranscriptView } from "../lib/types";

  const PLAIN_TEXT = { speakerNames: true, timestamps: false, maxCharsPerLine: 42, linesPerCue: 2 };

  let { id }: { id: number } = $props();
  let view = $state<TranscriptView | null>(null);
  let query = $state("");
  let exporting = $state(false);
  let copied = $state(false);
  let confirmingDelete = $state(false);

  const speakerOrder = $derived(view?.speakers.map((speaker) => speaker.name) ?? []);

  $effect(() => {
    void getTranscript(id).then((loaded) => (view = loaded));
    return () => player.stop();
  });

  async function copy(): Promise<void> {
    await navigator.clipboard.writeText(await renderExport(id, "txt", PLAIN_TEXT));
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  async function rename(from: string, to: string): Promise<void> {
    await renameSpeaker(id, from, to);
    view = await getTranscript(id);
    await library.load();
  }

  async function remove(): Promise<void> {
    await library.remove(id);
    nav.go("library");
  }
</script>

{#if view}
  <div class="page">
    <div class="main">
      <header>
        <button class="btn" aria-label="Back to the library" onclick={() => nav.go("library")}>
          <Icon name="back" />
        </button>
        <div class="heading">
          <h1>{view.item.title}</h1>
          <p class="muted">
            {formatClock(view.item.duration_ms)} · {languageName(view.item.language)}
            {view.item.speaker_count > 0 ? ` · ${view.item.speaker_count} speakers` : ""}
          </p>
        </div>
        <label class="search">
          <Icon name="search" />
          <input
            type="search"
            placeholder="Search transcript"
            aria-label="Search transcript"
            bind:value={query}
          />
        </label>
        <button class="btn" onclick={() => player.play(id, "all", 0, null)}>
          <Icon name={player.isActive(id, "all") ? "pause" : "play"} />
          {player.isActive(id, "all") ? "Pause" : "Play"}
        </button>
        <button class="btn" onclick={copy}><Icon name="copy" />{copied ? "Copied" : "Copy"}</button>
        <button class="btn primary" onclick={() => (exporting = true)}>
          <Icon name="down" />Export
        </button>
        {#if confirmingDelete}
          <button class="btn danger" onclick={remove}>Delete for good</button>
        {:else}
          <button class="btn" onclick={() => (confirmingDelete = true)}>Delete</button>
        {/if}
      </header>
      {#if player.error}<p class="error" role="alert">{player.error}</p>{/if}
      <div class="lines">
        {#if view.transcript.segments.length === 0}
          <p class="muted empty">No speech was found in this recording.</p>
        {:else}
          <TranscriptLines itemId={id} segments={view.transcript.segments} {query} {speakerOrder} />
        {/if}
      </div>
    </div>
    <SpeakerPanel {view} onrename={rename} />
  </div>
  {#if exporting}
    <ExportDialog itemId={id} title={view.item.title} onclose={() => (exporting = false)} />
  {/if}
{:else}
  <p class="muted loading">Loading…</p>
{/if}

<style>
  .page {
    display: flex;
    height: 100%;
  }

  .main {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 16px 32px;
    border-bottom: 1px solid var(--line);
  }

  .heading {
    flex: 1;
    min-width: 0;
  }

  h1 {
    overflow: hidden;
    font-size: 17px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text3);
  }

  input {
    width: 170px;
    border: 0;
    outline: 0;
    background: none;
    font: inherit;
    color: var(--text);
    user-select: text;
  }

  .error {
    padding: 8px 32px;
    color: var(--danger);
    user-select: text;
  }

  .danger {
    color: var(--danger);
  }

  .lines {
    flex: 1;
    padding: 12px 0;
    overflow: auto;
    background: var(--surface);
  }

  .empty,
  .loading {
    padding: 28px 36px;
  }
</style>
