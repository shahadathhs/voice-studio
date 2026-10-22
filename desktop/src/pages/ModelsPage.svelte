<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import ModelRow from "../components/ModelRow.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { info } from "../lib/app-info.svelte";
  import { formatBytes } from "../lib/format";
  import { models } from "../lib/models.svelte";
</script>

<section>
  <PageHeader
    title="Models"
    subtitle="Downloaded once, then used offline. Remove any you don't need."
  >
    {#snippet actions()}
      {#if models.items}
        <div class="usage">
          <strong>{models.installed.length} of {models.items.length} installed</strong>
          <div class="muted">{formatBytes(models.installedBytes)} on disk</div>
        </div>
      {/if}
    {/snippet}
  </PageHeader>

  {#if models.items === null}
    <p class="muted">Loading…</p>
  {:else if models.items.length === 0}
    <EmptyState title="No models to show" text="Models are only available in the desktop app." />
  {:else}
    <table>
      <thead>
        <tr>
          <th>Model</th>
          <th>Size</th>
          <th>Languages</th>
          <th>License</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each models.items as model (model.id)}
          <ModelRow {model} />
        {/each}
      </tbody>
    </table>
    <p class="muted folder">Stored in <span class="mono">{info.value?.dataDir ?? "…"}</span></p>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: 28px 36px;
  }

  .usage {
    text-align: right;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
  }

  th {
    padding: 10px 14px;
    border-bottom: 1px solid var(--line);
    font-size: 12px;
    text-align: left;
    color: var(--text2);
  }

  .folder {
    margin-top: -10px;
    font-size: 12px;
  }

  .folder span {
    user-select: text;
  }
</style>
