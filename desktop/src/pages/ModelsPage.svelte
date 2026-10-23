<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import ModelRow from "../components/ModelRow.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { info } from "../lib/app-info.svelte";
  import { formatBytes } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { groupModels } from "../lib/model-groups";
  import { models } from "../lib/models.svelte";
  import { nav } from "../lib/nav.svelte";
</script>

<section>
  <PageHeader title={t("models.title")} subtitle={t("models.subtitle")}>
    {#snippet actions()}
      {#if models.items}
        <div class="usage">
          <strong
            >{t("models.installedCount", {
              installed: models.installed.length,
              total: models.items.length,
            })}</strong
          >
          <div class="muted">
            {t("models.onDisk", { size: formatBytes(models.installedBytes) })}
          </div>
        </div>
      {/if}
    {/snippet}
  </PageHeader>

  {#if models.items === null}
    <p class="muted">{t("common.loading")}</p>
  {:else if models.items.length === 0}
    <EmptyState title={t("models.none.title")} text={t("models.none.text")} />
  {:else}
    {#each groupModels(models.items) as { group, models: members } (group.kind)}
      <div class="group">
        <div class="group-head">
          <div>
            <h2>{t(group.title)}</h2>
            <p class="muted">{t(group.purpose)}</p>
          </div>
          <button class="btn" onclick={() => nav.go(group.page)}>{t(group.pageLabel)}</button>
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
    <p class="muted folder">{t("models.storedIn", { path: info.value?.dataDir ?? "…" })}</p>
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

  .folder {
    user-select: text;
  }
</style>
