<script lang="ts">
  import { languageLabel, formatBytes, percent } from "../lib/format";
  import { models } from "../lib/models.svelte";
  import type { ModelStatus } from "../lib/types";
  import Icon from "./Icon.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { model }: { model: ModelStatus } = $props();

  const progress = $derived(models.progress[model.id]);
  const error = $derived(models.errors[model.id]);
  const done = $derived(progress ? percent(progress.downloaded, progress.total) : 0);
</script>

<tr>
  <td>
    <strong>{model.name}</strong>
    <div class="muted">{model.description}</div>
  </td>
  <td class="mono">{formatBytes(model.size_bytes)}</td>
  <td>{languageLabel(model.languages)}</td>
  <td>{model.license}</td>
  <td class="action">
    {#if progress}
      <div class="downloading">
        <ProgressBar value={done} label="Downloading {model.name}" />
        <span class="mono muted">{done}%</span>
        <button class="btn" onclick={() => models.cancel(model.id)}>Cancel</button>
      </div>
      <div class="muted small">
        {formatBytes(progress.downloaded)} of {formatBytes(progress.total)}
      </div>
    {:else if model.installed}
      <span class="installed">Installed</span>
      <button class="btn remove" onclick={() => models.remove(model.id)}>Remove</button>
    {:else}
      <button class="btn" onclick={() => models.install(model)}>
        <Icon name="down" />Install
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
