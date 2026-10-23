<script module lang="ts">
  import type { IconName } from "../lib/icons";

  export interface SegmentOption<V extends string> {
    value: V;
    label: string;
    icon?: IconName;
  }
</script>

<script lang="ts" generics="T extends string">
  import Icon from "./Icon.svelte";

  /**
   * Single-choice control. Default: labelled segments. `compact`: icon-only circular stops with a
   * circular thumb that slides to the selected one (labels stay available as tooltips and to
   * screen readers).
   */
  let {
    options,
    value,
    label,
    compact = false,
    onchange,
  }: {
    options: readonly SegmentOption<T>[];
    value: T;
    label: string;
    compact?: boolean;
    onchange: (value: T) => void;
  } = $props();

  const selectedIndex = $derived(options.findIndex((option) => option.value === value));
</script>

<div
  class="seg"
  class:compact
  role="radiogroup"
  aria-label={label}
  style:--count={options.length}
  style:--index={selectedIndex}
>
  {#if compact}<span class="thumb" aria-hidden="true"></span>{/if}
  {#each options as option (option.value)}
    <button
      role="radio"
      aria-checked={value === option.value}
      aria-label={compact ? option.label : undefined}
      title={compact ? option.label : undefined}
      class:on={value === option.value}
      onclick={() => onchange(option.value)}
    >
      {#if option.icon}<Icon name={option.icon} size={compact ? 14 : 16} />{/if}
      {#if !compact}{option.label}{/if}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--line-strong);
    border-radius: 7px;
    background: var(--surface);
  }

  button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 26px;
    padding: 0 12px;
    border-radius: 5px;
    color: var(--text2);
    font-weight: 550;
  }

  button.on {
    background: var(--text);
    color: var(--bg);
  }

  /* Compact: a pill track with N circular stops and one circular thumb. */
  .compact {
    --stop: 28px;
    --pad: 3px;
    position: relative;
    display: inline-grid;
    grid-template-columns: repeat(var(--count), var(--stop));
    gap: 0;
    padding: var(--pad);
    border-radius: 999px;
  }

  .compact button {
    position: relative;
    z-index: 1;
    width: var(--stop);
    height: var(--stop);
    padding: 0;
    border-radius: 50%;
    transition: color 150ms ease;
  }

  .compact button.on {
    background: none;
    color: var(--bg);
  }

  .thumb {
    position: absolute;
    top: var(--pad);
    left: var(--pad);
    width: var(--stop);
    height: var(--stop);
    border-radius: 50%;
    background: var(--text);
    transform: translateX(calc(var(--index) * var(--stop)));
    transition: transform 160ms ease;
  }

  @media (prefers-reduced-motion: reduce) {
    .thumb,
    .compact button {
      transition: none;
    }
  }
</style>
