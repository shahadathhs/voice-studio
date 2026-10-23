<script lang="ts">
  import { onMount } from "svelte";

  import { onDictationState } from "./lib/api-dictation";
  import { t } from "./lib/i18n.svelte";
  import type { DictationState } from "./lib/types";

  let current = $state<DictationState>({ state: "listening", text: null, message: null });

  const label = $derived(
    current.state === "listening"
      ? t("dictation.listening")
      : current.state === "working"
        ? t("dictation.working")
        : current.state === "done"
          ? t("dictation.done")
          : (current.message ?? t("dictation.failed")),
  );

  onMount(() => {
    let stop: (() => void) | undefined;
    void onDictationState((state) => (current = state)).then((off) => (stop = off));
    return () => stop?.();
  });
</script>

<div class="pill" class:failed={current.state === "failed"} role="status">
  <span class="dot" class:live={current.state === "listening"} aria-hidden="true"></span>
  <span class="text">
    <strong>{label}</strong>
    {#if current.state === "done" && current.text}<span class="muted">{current.text}</span>{/if}
  </span>
</div>

<style>
  .pill {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 100vh;
    padding: 0 16px;
    border: 1px solid var(--line);
    background: var(--surface);
    color: var(--text);
  }

  .dot {
    flex: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--text3);
  }

  .dot.live {
    background: var(--danger);
  }

  .failed strong {
    color: var(--danger);
  }

  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .text span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
