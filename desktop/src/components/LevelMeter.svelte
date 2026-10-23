<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  /** `level` is the recent peak from 0 to 1. */
  let { level }: { level: number } = $props();

  // Speech peaks are small next to full scale; a square root makes the bar readable.
  const width = $derived(Math.round(Math.sqrt(Math.min(1, Math.max(0, level))) * 100));
</script>

<div
  class="meter"
  role="meter"
  aria-label={t("record.level")}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={width}
>
  <div class="bar" style:width="{width}%"></div>
</div>

<style>
  .meter {
    height: 8px;
    border-radius: 4px;
    background: var(--line);
    overflow: hidden;
  }

  .bar {
    height: 100%;
    background: var(--accent);
    transition: width 60ms linear;
  }
</style>
