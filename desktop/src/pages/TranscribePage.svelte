<script lang="ts">
  import DropZone from "../components/DropZone.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import JobRow from "../components/JobRow.svelte";
  import LibraryRow from "../components/LibraryRow.svelte";
  import Notice from "../components/Notice.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { jobs } from "../lib/jobs.svelte";
  import { LANGUAGES } from "../lib/languages";
  import { library } from "../lib/library.svelte";
  import { models } from "../lib/models.svelte";

  const RECENT_COUNT = 5;

  const speechModels = $derived(models.installed.filter((model) => model.kind === "stt"));
  let chosenModel = $state<string | null>(null);
  let language = $state("");
  let translate = $state(false);
  const model = $derived(
    speechModels.find((m) => m.id === chosenModel)?.id ?? speechModels[0]?.id ?? null,
  );
  const recent = $derived((library.items ?? []).slice(0, RECENT_COUNT));
  const summary = $derived(
    jobs.items.length === 0
      ? "Turn recordings into text. Audio never leaves this computer."
      : `${jobs.items.length} file${jobs.items.length === 1 ? "" : "s"} · ${jobs.active.length} in progress`,
  );

  function queue(paths: string[]): void {
    if (model) void jobs.add(paths, model, language || null, translate);
  }
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

  <DropZone onfiles={queue} disabled={model === null} />

  <div class="options">
    <div>
      <div class="label">Model</div>
      <select class="field" aria-label="Model" bind:value={chosenModel} disabled={!model}>
        {#each speechModels as speech (speech.id)}
          <option value={speech.id} selected={speech.id === model}>{speech.name}</option>
        {:else}
          <option>No models installed</option>
        {/each}
      </select>
    </div>
    <div>
      <div class="label">Language</div>
      <select class="field" aria-label="Language" bind:value={language}>
        <option value="">Detect automatically</option>
        {#each LANGUAGES as option (option.code)}
          <option value={option.code}>{option.label}</option>
        {/each}
      </select>
    </div>
    <label class="check"
      ><input type="checkbox" bind:checked={translate} />Translate to English</label
    >
    <label class="check" title="Speaker identification arrives in phase 3">
      <input type="checkbox" disabled />Identify speakers
    </label>
  </div>

  {#if jobs.error}<p class="error" role="alert">{jobs.error}</p>{/if}

  {#if jobs.items.length > 0}
    <ul class="list" aria-label="Queue">
      {#each jobs.items as job (job.id)}<JobRow {job} />{/each}
    </ul>
  {/if}

  <Notice>Files are read in place and never uploaded.</Notice>

  <h2>Recent</h2>
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

  .options {
    display: flex;
    align-items: flex-end;
    gap: 24px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
  }

  input[type="checkbox"] {
    accent-color: var(--accent);
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
