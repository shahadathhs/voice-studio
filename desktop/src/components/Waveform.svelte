<script lang="ts">
  import { getWaveform } from "../lib/api";
  import { formatClock } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { player } from "../lib/player.svelte";

  const BARS = 240;
  const STEP_MS = 5000;

  let { itemId, durationMs }: { itemId: number; durationMs: number } = $props();

  let bars = $state<number[]>([]);

  // The outline is only available when the original recording is still on disk.
  $effect(() => {
    bars = [];
    void getWaveform(itemId, BARS)
      .then((outline) => (bars = outline))
      .catch(() => (bars = []));
  });

  const here = $derived(player.itemId === itemId && player.active !== null ? player.position : 0);
  const share = $derived(durationMs > 0 ? Math.min(1, here / durationMs) : 0);

  function seek(event: MouseEvent): void {
    const box = event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
    if (!box) return;
    const { left, width } = box.getBoundingClientRect();
    void player.seek(itemId, Math.round(((event.clientX - left) / width) * durationMs));
  }

  function nudge(event: KeyboardEvent): void {
    const step = event.key === "ArrowRight" ? STEP_MS : event.key === "ArrowLeft" ? -STEP_MS : 0;
    if (step === 0) return;
    event.preventDefault();
    void player.seek(itemId, Math.max(0, Math.min(durationMs, here + step)));
  }
</script>

{#if bars.length > 0}
  <div
    class="wave"
    role="slider"
    tabindex="0"
    aria-label={t("waveform.label")}
    aria-valuemin="0"
    aria-valuemax={durationMs}
    aria-valuenow={Math.round(here)}
    aria-valuetext={formatClock(here)}
    onclick={seek}
    onkeydown={nudge}
  >
    {#each bars as height, index (index)}
      <span
        class="bar"
        class:played={index / bars.length < share}
        style:height="{Math.max(4, height * 100)}%"
      ></span>
    {/each}
  </div>
{/if}

<style>
  .wave {
    display: flex;
    align-items: center;
    gap: 1px;
    height: 56px;
    padding: 6px 32px;
    border-bottom: 1px solid var(--line);
    cursor: pointer;
  }

  .bar {
    flex: 1;
    min-width: 1px;
    border-radius: 1px;
    background: var(--line-strong);
  }

  .bar.played {
    background: var(--accent);
  }
</style>
