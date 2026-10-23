<script lang="ts">
  import CloneVoice from "../components/CloneVoice.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import Segmented, { type SegmentOption } from "../components/Segmented.svelte";
  import VoiceCreator from "../components/VoiceCreator.svelte";
  import { t } from "../lib/i18n.svelte";
  import { models } from "../lib/models.svelte";
  import { nav } from "../lib/nav.svelte";
  import { speech } from "../lib/speech.svelte";

  type Tab = "mine" | "blend" | "clone";

  let tab = $state<Tab>("mine");

  const TABS: readonly SegmentOption<Tab>[] = [
    { value: "mine", label: t("voices.tab.mine") },
    { value: "blend", label: t("voices.tab.blend") },
    { value: "clone", label: t("voices.tab.clone") },
  ];
  const installed = $derived(models.installed.some((model) => model.kind === "tts"));
  const mine = $derived(speech.voices.filter((voice) => voice.custom || voice.cloned));

  $effect(() => {
    void speech.start();
    if (installed) void speech.loadVoices();
  });

  function use(id: string): void {
    speech.chooseVoice(id);
    nav.go("speak");
  }
</script>

<section>
  <PageHeader title={t("voices.page.title")} subtitle={t("voices.page.subtitle")} />

  {#if models.items !== null && !installed}
    <EmptyState title={t("speak.needModel.title")} text={t("speak.needModel.text")} />
    <div class="center">
      <button class="btn primary" onclick={() => nav.go("models")}>{t("speak.openModels")}</button>
    </div>
  {:else}
    <Segmented
      options={TABS}
      value={tab}
      label={t("voices.page.title")}
      onchange={(next) => (tab = next)}
    />
    <div class="panel">
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
              <span class="buttons">
                <button class="btn" onclick={() => use(voice.id)}>
                  <Icon name="play" />{t("voices.use")}
                </button>
                <button
                  class="icon"
                  title={label}
                  aria-label={label}
                  onclick={() => speech.removeVoice(voice.id)}
                >
                  <Icon name="trash" />
                </button>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 640px;
    padding: 28px 36px;
  }

  .center {
    display: flex;
    justify-content: center;
  }

  .panel {
    padding: 4px 0;
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
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
  }
</style>
