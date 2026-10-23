<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { languageName } from "../lib/languages";
  import { speech } from "../lib/speech.svelte";
  import type { VoiceInfo } from "../lib/types";
  import Icon from "./Icon.svelte";

  /** The group of cloned voices, which have no language of their own. */
  const CLONED = "cloned";

  /** The voices of each language together, in the order the languages first appear. */
  const groups = $derived.by(() => {
    const byLanguage = new Map<string, VoiceInfo[]>();
    for (const voice of speech.voices) {
      const key = voice.cloned ? CLONED : voice.language;
      byLanguage.set(key, [...(byLanguage.get(key) ?? []), voice]);
    }
    return [...byLanguage].map(([language, voices]) => ({ language, voices }));
  });
</script>

<div class="voices" role="radiogroup" aria-label={t("speak.voice")}>
  {#each groups as group (group.language)}
    {#if groups.length > 1}
      <div class="language">
        {group.language === CLONED ? t("clone.group") : languageName(group.language || null)}
      </div>
    {/if}
    {#each group.voices as voice (voice.id)}
      {@const selected = speech.voice?.id === voice.id}
      <div class="row" class:selected>
        <button
          class="voice"
          role="radio"
          aria-checked={selected}
          onclick={() => speech.chooseVoice(voice.id)}
        >
          <span class="dot" aria-hidden="true"></span>
          <span class="text">
            <strong>{voice.name}</strong>
            <span class="muted">{voice.description}</span>
          </span>
          {#if voice.cloned}
            <span class="badge">{t("clone.badge")}</span>
          {:else if voice.custom}
            <span class="badge">{t("voices.custom")}</span>
          {/if}
        </button>
        {#if voice.custom || voice.cloned}
          {@const label = voice.cloned
            ? t("clone.delete", { name: voice.name })
            : t("voices.delete", { name: voice.name })}
          <button
            class="remove"
            title={label}
            aria-label={label}
            onclick={() => speech.removeVoice(voice.id)}
          >
            <Icon name="trash" />
          </button>
        {/if}
      </div>
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

  .row {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--line);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .voice {
    display: flex;
    flex: 1;
    align-items: center;
    gap: 12px;
    min-width: 0;
    padding: 10px 14px;
    text-align: left;
  }

  .badge {
    margin-left: auto;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--accent-bg);
    color: var(--accent);
    font-size: 11.5px;
    font-weight: 600;
  }

  .remove {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    margin-right: 8px;
    border-radius: 6px;
    color: var(--text2);
  }

  .remove:hover {
    color: var(--danger);
  }

  .language {
    padding: 6px 14px;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
    color: var(--text2);
    font-size: 12px;
    font-weight: 600;
  }

  .row:hover {
    background: var(--bg);
  }

  .row.selected {
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
