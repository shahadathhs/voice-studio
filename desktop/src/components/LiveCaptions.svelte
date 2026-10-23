<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  let { final, draft, waiting }: { final: string; draft: string; waiting: boolean } = $props();
</script>

<section class="captions" aria-label={t("record.captions")} aria-live="polite">
  <header>
    <h2>{t("record.captions")}</h2>
    <span class="muted">{t("record.captionsHint")}</span>
  </header>
  {#if final === "" && draft === ""}
    <p class="muted">{waiting ? t("record.listening") : t("record.noCaptions")}</p>
  {:else}
    <p>
      <span class="final">{final}</span>
      {#if draft !== ""}
        <span class="draft">{draft}</span>
        <span class="badge">{t("record.draft")}</span>
      {/if}
    </p>
  {/if}
</section>

<style>
  .captions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px 20px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface);
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }

  p {
    line-height: 1.6;
    user-select: text;
  }

  .draft {
    color: var(--text3);
    font-style: italic;
  }

  .badge {
    margin-left: 6px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent-bg);
    color: var(--accent);
    font-size: 11px;
    font-style: normal;
    text-transform: uppercase;
  }
</style>
