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
  import { player } from "../lib/player.svelte";
  import { models } from "../lib/models.svelte";

  const RECENT_COUNT = 5;

  const speechModels = $derived(models.installed.filter((model) => model.kind === "stt"));
  let chosenModel = $state<string | null>(null);
  let language = $state("");
  let translate = $state(false);
  let identifySpeakers = $state(false);
  const model = $derived(
    speechModels.find((m) => m.id === chosenModel)?.id ?? speechModels[0]?.id ?? null,
  );
  const chosen = $derived(speechModels.find((m) => m.id === model));
  const canTranslate = $derived(chosen?.can_translate ?? true);
  const speakerModel = $derived(models.installed.find((m) => m.kind === "diarization"));
  const recent = $derived((library.items ?? []).slice(0, RECENT_COUNT));
  const summary = $derived(
    jobs.items.length === 0
      ? "Turn recordings into text. Audio never leaves this computer."
      : `${jobs.items.length} file${jobs.items.length === 1 ? "" : "s"} · ${jobs.active.length} in progress`,
  );

  function queue(paths: string[]): void {
    if (!model) return;
    const settings = {
      model,
      language: language || null,
      translate: translate && canTranslate,
      identifySpeakers: identifySpeakers && speakerModel !== undefined,
    };
    void jobs.add(settings, paths);
  }

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
    <label
      class="check"
      title={canTranslate ? "" : `${chosen?.name} cannot translate. Choose another model.`}
    >
      <input type="checkbox" bind:checked={translate} disabled={!canTranslate} />Translate to
      English
    </label>
    <label class="check" title={speakerModel ? "" : "Install the speaker model in Models first."}>
      <input type="checkbox" bind:checked={identifySpeakers} disabled={!speakerModel} />Identify
      speakers
    </label>
  </div>

  {#if player.error}<p class="error" role="alert">{player.error}</p>{/if}
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
