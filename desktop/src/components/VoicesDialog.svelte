<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { speech } from "../lib/speech.svelte";
  import CloneVoice from "./CloneVoice.svelte";
  import Icon from "./Icon.svelte";
  import Segmented, { type SegmentOption } from "./Segmented.svelte";
  import VoiceCreator from "./VoiceCreator.svelte";

  type Tab = "blend" | "clone" | "mine";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement | undefined = $state();
  let tab = $state<Tab>("blend");

  const TABS: readonly SegmentOption<Tab>[] = [
    { value: "blend", label: t("voices.tab.blend") },
    { value: "clone", label: t("voices.tab.clone") },
    { value: "mine", label: t("voices.tab.mine") },
  ];
  const mine = $derived(speech.voices.filter((voice) => voice.custom || voice.cloned));

  $effect(() => dialog?.showModal());
</script>

<dialog bind:this={dialog} aria-labelledby="voices-title" {onclose} oncancel={onclose}>
  <header>
    <h2 id="voices-title">{t("voices.manage.title")}</h2>
    <button class="icon" aria-label={t("common.close")} onclick={onclose}><Icon name="x" /></button>
  </header>
  <div class="tabs">
    <Segmented
      options={TABS}
      value={tab}
      label={t("voices.manage.title")}
      onchange={(next) => (tab = next)}
    />
  </div>
  <div class="body">
    {#if tab === "blend"}
      <VoiceCreator />
    {:else if tab === "clone"}
      <CloneVoice />
    {:else if mine.length === 0}
      <p class="muted">{t("voices.noneYet")}</p>
    {:else}
      <ul>
        {#each mine as voice (voice.id)}
          {@const label = voice.cloned
            ? t("clone.delete", { name: voice.name })
            : t("voices.delete", { name: voice.name })}
          <li>
            <span>
              <strong>{voice.name}</strong>
              <span class="muted">{voice.cloned ? t("clone.badge") : t("voices.custom")}</span>
            </span>
            <button
              class="icon"
              title={label}
              aria-label={label}
              onclick={() => speech.removeVoice(voice.id)}
            >
              <Icon name="trash" />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</dialog>

<style>
  dialog {
    width: 520px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    margin: auto;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface);
    color: var(--text);
  }

  dialog::backdrop {
    background: rgb(0 0 0 / 0.45);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 22px;
    border-bottom: 1px solid var(--line);
  }

  .tabs {
    padding: 14px 22px 0;
  }

  .body {
    padding: 14px 22px 22px;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
  }

  ul {
    display: flex;
    flex-direction: column;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 0;
    border-bottom: 1px solid var(--line);
  }

  li:last-child {
    border-bottom: 0;
  }
</style>
