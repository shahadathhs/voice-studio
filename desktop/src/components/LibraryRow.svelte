<script lang="ts">
  import { formatClock, formatDate } from "../lib/format";
  import { languageName } from "../lib/languages";
  import { nav } from "../lib/nav.svelte";
  import type { LibraryItem } from "../lib/types";

  let { item, snippet }: { item: LibraryItem; snippet?: string } = $props();

  const speakers = $derived(
    item.speaker_count > 0
      ? ` · ${item.speaker_count} speaker${item.speaker_count === 1 ? "" : "s"}`
      : "",
  );
  /** Splits "a [b] c" into plain and matched parts so matches can be marked without HTML. */
  const parts = $derived((snippet ?? "").split(/(\[[^\]]*\])/).filter((part) => part !== ""));
</script>

<li>
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
</li>

<style>
  li {
    list-style: none;
    border-bottom: 1px solid var(--line);
  }

  li:last-child {
    border-bottom: 0;
  }

  .row {
    display: grid;
    gap: 2px;
    width: 100%;
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
