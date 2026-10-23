<script lang="ts">
  import { i18n, t } from "../lib/i18n.svelte";
  import { languageLabel, formatBytes, percent } from "../lib/format";
  import { models } from "../lib/models.svelte";
  import type { ModelStatus } from "../lib/types";
  import Icon from "./Icon.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { model, recommended = false }: { model: ModelStatus; recommended?: boolean } = $props();

  const progress = $derived(models.progress[model.id]);
  const error = $derived(models.errors[model.id]);
  const done = $derived(progress ? percent(progress.downloaded, progress.total) : 0);
</script>

<tr>
  <td>
    <strong>{model.name}</strong>
    {#if recommended}<span class="badge">{t("models.recommended")}</span>{/if}
    <div class="muted">{i18n.tOr(`modelInfo.${model.id}`, model.description)}</div>
    <div class="muted meta">
      {languageLabel(model.languages)} · {t("models.licence", {
        license: model.license,
      })}{model.can_translate ? ` · ${t("models.canTranslate")}` : ""}
    </div>
  </td>
  <td class="size mono">{formatBytes(model.size_bytes)}</td>
  <td class="action">
    {#if progress}
      <div class="downloading">
        <ProgressBar value={done} label={t("models.downloading", { name: model.name })} />
        <span class="mono muted">{done}%</span>
        <button class="btn" onclick={() => models.cancel(model.id)}>{t("common.cancel")}</button>
      </div>
      <div class="muted small">
        {t("models.downloaded", {
          done: formatBytes(progress.downloaded),
          total: formatBytes(progress.total),
        })}
      </div>
    {:else if model.installed}
      <span class="installed">{t("models.installed")}</span>
      <button class="btn remove" onclick={() => models.remove(model.id)}
        >{t("models.remove")}</button
      >
    {:else}
      <button class="btn" onclick={() => models.install(model)}>
        <Icon name="down" />{t("models.install")}
      </button>
    {/if}
    {#if error}<div class="error small">{error}</div>{/if}
  </td>
</tr>

<style>
  td {
    padding: 10px 14px;
    border-bottom: 1px solid var(--line);
    vertical-align: middle;
  }

  tr:last-child td {
    border-bottom: 0;
  }

  .size {
    width: 90px;
  }

  .badge {
    margin-left: 6px;
    padding: 1px 7px;
    border-radius: 10px;
    background: var(--accent-bg);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }

  .meta {
    margin-top: 2px;
    font-size: 11.5px;
  }

  .action {
    width: 270px;
    text-align: right;
  }

  .downloading {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .downloading :global(.bar) {
    flex: 1;
  }

  .installed {
    display: inline-block;
    margin-right: 8px;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--accent-bg);
    color: var(--accent);
    font-size: 11.5px;
    font-weight: 600;
  }

  .remove {
    color: var(--danger);
  }

  .small {
    margin-top: 3px;
    font-size: 11.5px;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
