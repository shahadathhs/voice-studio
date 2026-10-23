<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { languageName } from "../lib/languages";
  import { speech } from "../lib/speech.svelte";
  import type { VoiceInfo } from "../lib/types";

  /** The voices of each language together, in the order the languages first appear. */
  const groups = $derived.by(() => {
    const byLanguage = new Map<string, VoiceInfo[]>();
    for (const voice of speech.voices) {
      byLanguage.set(voice.language, [...(byLanguage.get(voice.language) ?? []), voice]);
    }
    return [...byLanguage].map(([language, voices]) => ({ language, voices }));
  });
</script>

<div class="voices" role="radiogroup" aria-label={t("speak.voice")}>
  {#each groups as group (group.language)}
    {#if groups.length > 1}
      <div class="language">{languageName(group.language || null)}</div>
    {/if}
    {#each group.voices as voice (voice.id)}
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

  .language {
    padding: 6px 14px;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
    color: var(--text2);
    font-size: 12px;
    font-weight: 600;
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
