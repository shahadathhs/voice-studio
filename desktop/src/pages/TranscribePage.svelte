<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import Notice from "../components/Notice.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { models } from "../lib/models.svelte";

  const speechModels = $derived(models.installed.filter((model) => model.kind === "stt"));
</script>

<section>
  <PageHeader
    title="Transcribe"
    subtitle="Turn recordings into text. Audio never leaves this computer."
  />

  <div class="notice">
    <Notice>
      Transcription arrives in phase 2 of the roadmap. You can already install models in Models.
    </Notice>
  </div>

  <div class="drop">
    <span class="faint"><Icon name="down" size={20} /></span>
    <strong>Drop audio or video files here</strong>
    <p class="muted">WAV, MP3, M4A, FLAC, OGG, AAC, MP4, MOV, MKV</p>
    <div class="buttons">
      <button class="btn primary" disabled><Icon name="folder" />Choose files…</button>
      <button class="btn" disabled><Icon name="mic" />Record</button>
    </div>
  </div>

  <div class="options">
    <div>
      <div class="label">Model</div>
      <select class="field" disabled aria-label="Model">
        {#each speechModels as model (model.id)}
          <option>{model.name}</option>
        {:else}
          <option>No models installed</option>
        {/each}
      </select>
    </div>
    <div>
      <div class="label">Language</div>
      <select class="field" disabled aria-label="Language">
        <option>Detect automatically</option>
      </select>
    </div>
    <label class="check"><input type="checkbox" disabled checked />Identify speakers</label>
    <label class="check"><input type="checkbox" disabled />Translate to English</label>
  </div>

  <h2>Recent</h2>
  <EmptyState title="No transcripts yet" text="Finished transcripts will appear here." />
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 28px 36px;
  }

  .drop {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 230px;
    border: 1.5px dashed var(--line-strong);
    border-radius: 10px;
    background: var(--surface);
  }

  .drop strong {
    font-size: 15px;
  }

  .faint {
    color: var(--text3);
  }

  .buttons,
  .options {
    display: flex;
    align-items: flex-end;
    gap: 8px;
  }

  .options {
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
</style>
