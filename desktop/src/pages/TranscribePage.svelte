<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import BatchPanel from "../components/BatchPanel.svelte";
  import DropZone from "../components/DropZone.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import JobRow from "../components/JobRow.svelte";
  import LibraryRow from "../components/LibraryRow.svelte";
  import Notice from "../components/Notice.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { batch } from "../lib/batch.svelte";
  import { jobs } from "../lib/jobs.svelte";
  import { library } from "../lib/library.svelte";
  import { player } from "../lib/player.svelte";

  const RECENT_COUNT = 5;

  const recent = $derived((library.items ?? []).slice(0, RECENT_COUNT));
  const summary = $derived(
    jobs.items.length === 0
      ? t("transcribe.subtitle.idle")
      : t("transcribe.subtitle.queue", { count: jobs.items.length, active: jobs.active.length }),
  );

  $effect(() => () => player.stop());
</script>

<section>
  <PageHeader title={t("transcribe.title")} subtitle={summary}>
    {#snippet actions()}
      {#if jobs.active.length > 0}
        <button class="btn" onclick={() => jobs.setPaused(!jobs.paused)}>
          {jobs.paused ? t("transcribe.resume") : t("transcribe.pause")}
        </button>
      {/if}
      {#if jobs.items.some((job) => !jobs.active.includes(job))}
        <button class="btn" onclick={() => jobs.clearFinished()}
          >{t("transcribe.clearFinished")}</button
        >
      {/if}
    {/snippet}
  </PageHeader>

  <DropZone onfiles={(paths) => batch.add(paths)} disabled={batch.model === null} />

  {#if batch.files.length > 0}
    <BatchPanel />
  {/if}

  {#if jobs.items.length > 0}
    <ul class="list" aria-label={t("transcribe.queue")}>
      {#each jobs.items as job (job.id)}<JobRow {job} />{/each}
    </ul>
  {/if}

  <Notice>{t("transcribe.privacy")}</Notice>

  <h2>{t("transcribe.recent")}</h2>
  {#if player.error}<p class="error" role="alert">{player.error}</p>{/if}
  {#if recent.length === 0}
    <EmptyState title={t("transcribe.noRecent.title")} text={t("transcribe.noRecent.text")} />
  {:else}
    <ul class="list" aria-label={t("transcribe.recentList")}>
      {#each recent as item (item.id)}<LibraryRow {item} />{/each}
    </ul>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 28px 36px;
  }

  .list {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
    overflow: hidden;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
