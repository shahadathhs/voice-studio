<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { formatClock, formatDate } from "../lib/format";
  import { languageName } from "../lib/languages";
  import { library } from "../lib/library.svelte";
  import { nav } from "../lib/nav.svelte";
  import { player } from "../lib/player.svelte";
  import type { LibraryItem } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    item,
    snippet,
    startMs = 0,
  }: { item: LibraryItem; snippet?: string; startMs?: number } = $props();

  const speakers = $derived(
    item.speaker_count > 0 ? ` · ${t("row.speakers", { count: item.speaker_count })}` : "",
  );
  let confirming = $state(false);

  async function remove(): Promise<void> {
    if (player.itemId === item.id) player.stop();
    await library.remove(item.id);
  }

  const playing = $derived(player.isActive(item.id, "all"));
  /** Splits "a [b] c" into plain and matched parts so matches can be marked without HTML. */
  const parts = $derived((snippet ?? "").split(/(\[[^\]]*\])/).filter((part) => part !== ""));
</script>

<li>
  <button
    class="play"
    aria-label={t(playing ? "row.pause" : "row.play", { title: item.title })}
    onclick={() => player.play(item.id, "all", startMs, null)}
  >
    <Icon name={playing ? "pause" : "play"} />
  </button>
  <button class="row" onclick={() => nav.open(item.id)}>
    <span class="title">{item.title}</span>
    <span class="meta muted">
      {formatClock(item.duration_ms)} · {languageName(item.language)}{speakers} · {formatDate(
        item.created_at,
      )}
    </span>
    {#if snippet}
      <span class="snippet muted">
        {#each parts as part, index (index)}
          {#if part.startsWith("[")}<mark>{part.slice(1, -1)}</mark>{:else}{part}{/if}
        {/each}
      </span>
    {/if}
  </button>
  <div class="actions">
    {#if confirming}
      <button class="btn danger" onclick={remove}>{t("row.delete")}</button>
      <button class="btn" onclick={() => (confirming = false)}>{t("row.keep")}</button>
    {:else}
      <button
        class="trash"
        aria-label={t("row.deleteLabel", { title: item.title })}
        onclick={() => (confirming = true)}
      >
        <Icon name="trash" />
      </button>
    {/if}
  </div>
</li>

<style>
  li {
    display: flex;
    align-items: center;
    padding-left: 12px;
    list-style: none;
    border-bottom: 1px solid var(--line);
  }

  li:last-child {
    border-bottom: 0;
  }

  .play {
    display: grid;
    flex: none;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--accent-bg);
    color: var(--accent);
  }

  .actions {
    display: flex;
    flex: none;
    gap: 6px;
    padding: 0 12px;
  }

  .trash {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    color: var(--text3);
  }

  .trash:hover {
    background: var(--line);
    color: var(--danger);
  }

  .danger {
    color: var(--danger);
  }

  .row {
    display: grid;
    flex: 1;
    gap: 2px;
    min-width: 0;
    padding: 11px 16px;
    text-align: left;
  }

  .row:hover {
    background: var(--bg);
  }

  .title {
    font-weight: 600;
  }

  .meta,
  .snippet {
    font-size: 12px;
  }

  mark {
    background: var(--highlight);
    color: var(--text);
    border-radius: 2px;
  }
</style>
