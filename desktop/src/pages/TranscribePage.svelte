<script lang="ts">
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
      ? "Turn recordings into text. Audio never leaves this computer."
      : `${jobs.items.length} file${jobs.items.length === 1 ? "" : "s"} · ${jobs.active.length} in progress`,
  );

  $effect(() => () => player.stop());
</script>

<section>
  <PageHeader title="Transcribe" subtitle={summary}>
    {#snippet actions()}
      {#if jobs.active.length > 0}
        <button class="btn" onclick={() => jobs.setPaused(!jobs.paused)}>
          {jobs.paused ? "Resume queue" : "Pause queue"}
        </button>
      {/if}
      {#if jobs.items.some((job) => !jobs.active.includes(job))}
        <button class="btn" onclick={() => jobs.clearFinished()}>Clear finished</button>
      {/if}
    {/snippet}
  </PageHeader>

  <DropZone onfiles={(paths) => batch.add(paths)} disabled={batch.model === null} />

  {#if batch.files.length > 0}
    <BatchPanel />
  {/if}

  {#if jobs.items.length > 0}
    <ul class="list" aria-label="Queue">
      {#each jobs.items as job (job.id)}<JobRow {job} />{/each}
    </ul>
  {/if}

  <Notice>Files are read in place and never uploaded.</Notice>

  <h2>Recent</h2>
  {#if player.error}<p class="error" role="alert">{player.error}</p>{/if}
  {#if recent.length === 0}
    <EmptyState title="No transcripts yet" text="Finished transcripts will appear here." />
  {:else}
    <ul class="list" aria-label="Recent transcripts">
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
