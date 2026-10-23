<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import ModelRow from "../components/ModelRow.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { info } from "../lib/app-info.svelte";
  import { formatBytes } from "../lib/format";
  import { groupModels } from "../lib/model-groups";
  import { models } from "../lib/models.svelte";
  import { nav } from "../lib/nav.svelte";
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
    {#each groupModels(models.items) as { group, models: members } (group.kind)}
      <div class="group">
        <div class="group-head">
          <div>
            <h2>{group.title}</h2>
            <p class="muted">{group.purpose}</p>
          </div>
          <button class="btn" onclick={() => nav.go(group.page)}>{group.pageLabel}</button>
        </div>
        <table>
          <tbody>
            {#each members as model (model.id)}
              <ModelRow {model} recommended={model.id === group.recommended} />
            {/each}
          </tbody>
        </table>
      </div>
    {/each}
    <p class="muted folder">Stored in <span class="mono">{info.value?.dataDir ?? "…"}</span></p>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 26px;
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

  .group {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .group-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }

  .group-head p {
    max-width: 640px;
    margin-top: 2px;
  }

  .folder {
    margin-top: -10px;
    font-size: 12px;
  }

  .folder span {
    user-select: text;
  }
</style>
