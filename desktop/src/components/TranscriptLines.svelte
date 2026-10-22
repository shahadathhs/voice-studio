<script lang="ts">
  import { formatClock } from "../lib/format";
  import type { Segment } from "../lib/types";

  let {
    segments,
    query,
    speakerOrder,
  }: { segments: readonly Segment[]; query: string; speakerOrder: readonly string[] } = $props();

  const escape = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

  /** Splits a line into plain and matching parts so matches can be marked without HTML. */
  function split(text: string): { text: string; hit: boolean }[] {
    const words = query.trim().split(/\s+/).filter(Boolean);
    if (words.length === 0) return [{ text, hit: false }];
    const pattern = new RegExp(`(${words.map(escape).join("|")})`, "gi");
    return text
      .split(pattern)
      .filter((part) => part !== "")
      .map((part, index) => ({ text: part, hit: index % 2 === 1 }));
  }

  const colour = (speaker: string): string => {
    const index = speakerOrder.indexOf(speaker);
    return `var(--sp${(index < 0 ? 0 : index) % 4})`;
  };
</script>

<ol>
  {#each segments as segment, index (index)}
    <li>
      <span class="time mono">{formatClock(segment.start_ms)}</span>
      <span class="speaker" style:color={segment.speaker ? colour(segment.speaker) : undefined}>
        {segment.speaker ?? ""}
      </span>
      <p>
        {#each split(segment.text) as part, partIndex (partIndex)}
          {#if part.hit}<mark>{part.text}</mark>{:else}{part.text}{/if}
        {/each}
      </p>
    </li>
  {/each}
</ol>

<style>
  ol {
    list-style: none;
  }

  li {
    display: grid;
    grid-template-columns: 64px 120px 1fr;
    gap: 14px;
    padding: 9px 32px;
  }

  .time {
    color: var(--text3);
  }

  .speaker {
    font-weight: 600;
  }

  p {
    font-size: 14px;
    line-height: 1.5;
    user-select: text;
  }

  mark {
    background: var(--highlight);
    color: inherit;
    border-radius: 2px;
  }
</style>
