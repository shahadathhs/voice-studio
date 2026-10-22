<script lang="ts">
  import { baseName } from "../lib/format";
  import { LANGUAGES } from "../lib/languages";
  import { batch } from "../lib/batch.svelte";
  import { jobs } from "../lib/jobs.svelte";
  import { nav } from "../lib/nav.svelte";
  import Icon from "./Icon.svelte";

  const count = $derived(batch.files.length);
  const noun = $derived(count === 1 ? "file" : "files");
</script>

<section class="panel" aria-label="Files ready to transcribe">
  <header>
    <div>
      <h2>{count} {noun} ready</h2>
      <p class="muted">Choose how to transcribe {count === 1 ? "it" : "them"}, then start.</p>
    </div>
    <button class="btn" onclick={() => batch.clear()}>Cancel</button>
  </header>

  <ul class="files" aria-label="Files">
    {#each batch.files as path (path)}
      <li>
        <Icon name="doc" size={14} />
        <span class="name" title={path}>{baseName(path)}</span>
        <button
          class="icon"
          aria-label="Remove {baseName(path)}"
          onclick={() => batch.remove(path)}
        >
          <Icon name="x" size={14} />
        </button>
      </li>
    {/each}
  </ul>

  <div class="options">
    <div class="option">
      <label class="label" for="batch-model">Model</label>
      <select
        id="batch-model"
        class="field"
        value={batch.model?.id}
        onchange={(event) => batch.chooseModel(event.currentTarget.value)}
      >
        {#each batch.speechModels as speech (speech.id)}
          <option value={speech.id}>{speech.name}</option>
        {/each}
      </select>
    </div>

    <div class="option">
      <label class="label" for="batch-language">Language</label>
      <select id="batch-language" class="field" bind:value={batch.language}>
        <option value="">Detect automatically</option>
        {#each LANGUAGES as option (option.code)}
          <option value={option.code}>{option.label}</option>
        {/each}
      </select>
    </div>

    <div class="option">
      <label class="check">
        <input type="checkbox" bind:checked={batch.translate} disabled={!batch.canTranslate} />
        Translate to English
      </label>
      {#if !batch.canTranslate}
        <p class="hint muted">{batch.model?.name} cannot translate. Choose another model.</p>
      {/if}
    </div>

    <div class="option">
      <label class="check">
        <input
          type="checkbox"
          bind:checked={batch.identifySpeakers}
          disabled={batch.speakerModel === null}
        />
        Identify who is speaking
      </label>
      {#if batch.speakerModel === null}
        <p class="hint muted">
          Needs the speaker model.
          <button class="link" onclick={() => nav.go("models")}>Install it in Models</button>
        </p>
      {:else}
        <p class="hint muted">
          Labels each line SPEAKER_00, SPEAKER_01, … You can rename them later.
        </p>
      {/if}
    </div>
  </div>

  {#if jobs.error}<p class="error" role="alert">{jobs.error}</p>{/if}

  <footer>
    <button class="btn primary" onclick={() => batch.start()}>
      Transcribe {count}
      {noun}
    </button>
  </footer>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 18px 20px;
    border: 1px solid var(--accent);
    border-radius: 10px;
    background: var(--surface);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  h2 {
    font-size: 15px;
  }

  .files {
    display: flex;
    flex-direction: column;
    max-height: 132px;
    overflow: auto;
    border: 1px solid var(--line);
    border-radius: 8px;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--line);
    color: var(--text2);
  }

  li:last-child {
    border-bottom: 0;
  }

  .name {
    flex: 1;
    overflow: hidden;
    color: var(--text);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 5px;
    color: var(--text3);
  }

  .icon:hover {
    background: var(--line);
    color: var(--text);
  }

  .options {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px 24px;
  }

  .option .field {
    width: 100%;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 30px;
    font-weight: 550;
  }

  input[type="checkbox"] {
    accent-color: var(--accent);
  }

  .hint {
    margin-top: 2px;
    font-size: 12px;
  }

  .link {
    color: var(--accent);
    text-decoration: underline;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }

  footer {
    display: flex;
    justify-content: flex-end;
  }
</style>
