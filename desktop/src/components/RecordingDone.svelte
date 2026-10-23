<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { formatClock } from "../lib/format";
  import { batch } from "../lib/batch.svelte";
  import { record } from "../lib/record.svelte";
  import Icon from "./Icon.svelte";

  const result = $derived(record.result);
</script>

{#if result}
  <section class="done" aria-label={t("record.done")}>
    <div>
      <h2>{t("record.done")}</h2>
      <p class="muted">{t("record.length", { time: formatClock(result.duration_ms) })}</p>
    </div>
    <div class="buttons">
      <button class="btn primary" onclick={() => record.transcribe()} disabled={!batch.model}>
        {t("record.transcribe")}
      </button>
      <button class="btn" onclick={() => record.saveAs()}>
        <Icon name="down" />{t("record.saveAs")}
      </button>
      <button class="btn" onclick={() => record.discard()}>
        <Icon name="trash" />{t("record.discard")}
      </button>
    </div>
  </section>
{/if}

<style>
  .done {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 20px;
    border: 1px solid var(--accent);
    border-radius: 10px;
    background: var(--surface);
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
