<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { languageName } from "../lib/languages";
  import { speech } from "../lib/speech.svelte";
  import type { VoiceInfo } from "../lib/types";

  /** The group of the user's own voices (blends and clones), which can span languages. */
  const MINE = "mine";
  /** The languages a cloned voice can speak. */
  const CLONE_LANGUAGES = ["en", "es", "fr", "it", "pt", "hi"];

  let { disabled = false }: { disabled?: boolean } = $props();

  const keyOf = (voice: VoiceInfo): string =>
    voice.custom || voice.cloned ? MINE : voice.language;

  const groups = $derived.by(() => {
    const byKey = new Map<string, VoiceInfo[]>();
    for (const voice of speech.voices) {
      byKey.set(keyOf(voice), [...(byKey.get(keyOf(voice)) ?? []), voice]);
    }
    const found = [...byKey].map(([key, voices]) => ({
      key,
      label: key === MINE ? t("voices.mine") : languageName(key || null),
      voices,
    }));
    return found.toSorted((a, b) => (a.key === MINE ? 1 : b.key === MINE ? -1 : 0));
  });
  const current = $derived(speech.voice ? keyOf(speech.voice) : (groups[0]?.key ?? ""));
  const choices = $derived(groups.find((g) => g.key === current)?.voices ?? []);

  function chooseGroup(key: string): void {
    const first = groups.find((g) => g.key === key)?.voices[0];
    if (first) speech.chooseVoice(first.id);
  }

  const describe = (voice: VoiceInfo): string =>
    voice.custom || voice.cloned
      ? voice.name
      : `${voice.name} · ${voice.description.split(",").slice(-1)[0]?.trim() ?? ""}`;
</script>

<div class="picker">
  <label>
    <span class="muted">{t("voices.language")}</span>
    <select
      class="field"
      value={current}
      {disabled}
      onchange={(event) => chooseGroup(event.currentTarget.value)}
    >
      {#each groups as group (group.key)}
        <option value={group.key}>{group.label}</option>
      {/each}
    </select>
  </label>
  <label>
    <span class="muted">{t("speak.voice")}</span>
    <select
      class="field"
      value={speech.voice?.id}
      {disabled}
      onchange={(event) => speech.chooseVoice(event.currentTarget.value)}
    >
      {#each choices as voice (voice.id)}
        <option value={voice.id}>{describe(voice)}</option>
      {/each}
    </select>
  </label>
  {#if speech.voice?.cloned}
    <label>
      <span class="muted">{t("clone.language")}</span>
      <select class="field" bind:value={speech.cloneLanguage} {disabled}>
        {#each CLONE_LANGUAGES as code (code)}
          <option value={code}>{languageName(code)}</option>
        {/each}
      </select>
    </label>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
  }

  select {
    min-width: 170px;
  }
</style>
