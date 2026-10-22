<script lang="ts">
  import { speech } from "../lib/speech.svelte";
</script>

<div class="voices" role="radiogroup" aria-label="Voice">
  {#each speech.voices as voice (voice.id)}
    {@const selected = speech.voice?.id === voice.id}
    <button
      class="voice"
      class:selected
      role="radio"
      aria-checked={selected}
      onclick={() => speech.chooseVoice(voice.id)}
    >
      <span class="dot" aria-hidden="true"></span>
      <span class="text">
        <strong>{voice.name}</strong>
        <span class="muted">{voice.description}</span>
      </span>
    </button>
  {/each}
</div>

<style>
  .voices {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
    overflow: hidden;
  }

  .voice {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--line);
    text-align: left;
  }

  .voice:last-child {
    border-bottom: 0;
  }

  .voice:hover {
    background: var(--bg);
  }

  .voice.selected {
    background: var(--accent-bg);
  }

  .dot {
    flex: none;
    width: 16px;
    height: 16px;
    border: 1.5px solid var(--line-strong);
    border-radius: 50%;
  }

  .selected .dot {
    border: 5px solid var(--accent);
    background: var(--surface);
  }

  .text {
    display: flex;
    flex-direction: column;
  }
</style>
