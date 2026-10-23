<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { speech } from "../lib/speech.svelte";
  import type { VoiceInfo, VoicePart } from "../lib/types";
  import Icon from "./Icon.svelte";

  const MAX_PARTS = 4;
  const START_WEIGHT = 50;

  interface Accent {
    letter: string;
    label: string;
    voices: VoiceInfo[];
  }

  let name = $state("");
  let letter = $state("");
  /** Percent weight of each ticked voice, by voice id. */
  let weights = $state<Record<string, number>>({});

  /** Shipped voices grouped by the language letter of their id (American and British English
   *  are different groups). */
  const accents = $derived.by(() => {
    const groups = new Map<string, Accent>();
    for (const voice of speech.voices.filter((v) => !v.custom && !v.cloned)) {
      const key = voice.id.charAt(0);
      const group = groups.get(key) ?? {
        letter: key,
        label: voice.description.split(",")[0] ?? key,
        voices: [],
      };
      group.voices.push(voice);
      groups.set(key, group);
    }
    return [...groups.values()];
  });
  const accent = $derived(accents.find((a) => a.letter === letter) ?? accents[0]);
  const chosen = $derived(accent ? accent.voices.filter((v) => v.id in weights) : []);
  const parts = $derived<VoicePart[]>(chosen.map((v) => ({ voice: v.id, weight: weights[v.id] })));
  const ready = $derived(name.trim().length > 0 && chosen.length >= 2);
  const busy = $derived(speech.status === "working");

  function toggle(voice: VoiceInfo, on: boolean): void {
    if (on) weights[voice.id] = START_WEIGHT;
    else delete weights[voice.id];
  }

  function changeLanguage(next: string): void {
    letter = next;
    weights = {};
  }

  async function save(): Promise<void> {
    if (await speech.createVoice(name, parts)) {
      name = "";
      weights = {};
    }
  }
</script>

<div class="creator">
  <h2>{t("voices.create.title")}</h2>
  <p class="muted">{t("voices.create.hint")}</p>

  <label class="group">
    <span>{t("voices.name")}</span>
    <input
      type="text"
      maxlength="40"
      placeholder={t("voices.namePlaceholder")}
      bind:value={name}
      disabled={busy}
    />
  </label>

  {#if accents.length > 1}
    <label class="group">
      <span>{t("voices.language")}</span>
      <select
        value={accent?.letter}
        onchange={(e) => changeLanguage(e.currentTarget.value)}
        disabled={busy}
      >
        {#each accents as option (option.letter)}
          <option value={option.letter}>{option.label}</option>
        {/each}
      </select>
    </label>
  {/if}

  <fieldset>
    <legend>{t("voices.mix")}</legend>
    {#each accent?.voices ?? [] as voice (voice.id)}
      {@const on = voice.id in weights}
      <div class="mix">
        <label class="pick">
          <input
            type="checkbox"
            checked={on}
            disabled={busy || (!on && chosen.length >= MAX_PARTS)}
            onchange={(e) => toggle(voice, e.currentTarget.checked)}
          />
          {voice.name}
        </label>
        {#if on}
          <input
            type="range"
            min="5"
            max="100"
            step="5"
            bind:value={weights[voice.id]}
            aria-label={t("voices.weight", { voice: voice.name })}
            disabled={busy}
          />
          <span class="mono">{weights[voice.id]}</span>
        {/if}
      </div>
    {/each}
  </fieldset>

  {#if chosen.length >= MAX_PARTS}
    <p class="muted">{t("voices.maxFour")}</p>
  {:else if !ready}
    <p class="muted">{t("voices.needTwo")}</p>
  {/if}

  <div class="actions">
    <button
      class="btn"
      onclick={() => speech.preview(name || "Preview", parts)}
      disabled={busy || chosen.length < 2}
    >
      <Icon name="play" />{t("voices.preview")}
    </button>
    <button class="btn primary" onclick={save} disabled={busy || !ready}>
      {t("voices.save")}
    </button>
  </div>
</div>

<style>
  .creator {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--text2);
    font-size: 12px;
  }

  .group input,
  .group select {
    padding: 6px 8px;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font: inherit;
  }

  fieldset {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }

  legend {
    padding: 0;
    margin-bottom: 4px;
    color: var(--text2);
    font-size: 12px;
  }

  .mix {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 24px;
  }

  .pick {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 90px;
  }

  .mix input[type="range"] {
    flex: 1;
    min-width: 0;
    accent-color: var(--accent);
  }

  input[type="checkbox"] {
    accent-color: var(--accent);
  }

  .mono {
    min-width: 28px;
    text-align: right;
  }

  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
</style>
