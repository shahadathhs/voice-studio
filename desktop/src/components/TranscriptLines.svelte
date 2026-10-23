<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { formatClock } from "../lib/format";
  import { player } from "../lib/player.svelte";
  import type { Segment } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    itemId,
    segments,
    query,
    speakerOrder,
  }: {
    itemId: number;
    segments: readonly Segment[];
    query: string;
    speakerOrder: readonly string[];
  } = $props();

  const isPlaying = (index: number): boolean => player.isActive(itemId, index);
  /** The line being heard while the whole file plays. */
  const isCurrent = (segment: Segment): boolean =>
    player.isActive(itemId, "all") &&
    player.position >= segment.start_ms &&
    player.position < segment.end_ms;

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
    <li class:current={isPlaying(index) || isCurrent(segment)}>
      <button
        class="time mono"
        aria-label={t(isPlaying(index) ? "transcript.pauseFrom" : "transcript.playFrom", {
          start: formatClock(segment.start_ms),
          end: formatClock(segment.end_ms),
        })}
        onclick={() => player.play(itemId, index, segment.start_ms, segment.end_ms)}
      >
        <Icon name={isPlaying(index) ? "pause" : "play"} size={14} />
        {formatClock(segment.start_ms)}–{formatClock(segment.end_ms)}
      </button>
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
    grid-template-columns: 124px 120px 1fr;
    gap: 14px;
    padding: 9px 32px;
  }

  li.current {
    background: var(--highlight);
  }

  .time {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 8px 0 4px;
    border-radius: 12px;
    color: var(--accent);
    font-size: 12px;
    white-space: nowrap;
  }

  .time:hover {
    background: var(--accent-bg);
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
