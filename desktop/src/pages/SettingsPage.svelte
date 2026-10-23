<script lang="ts">
  import NoticesDialog from "../components/NoticesDialog.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import SettingRow from "../components/SettingRow.svelte";
  import ThemeSwitcher from "../components/ThemeSwitcher.svelte";
  import { info } from "../lib/app-info.svelte";
  import { i18n, LOCALES, t } from "../lib/i18n.svelte";
  import { languageOptions } from "../lib/languages";
  import { models } from "../lib/models.svelte";
  import { settings } from "../lib/settings.svelte";
  import type { Device } from "../lib/types";

  let showingNotices = $state(false);

  const speechModels = $derived(models.installed.filter((model) => model.kind === "stt"));
  const hasSpeakerModel = $derived(models.installed.some((model) => model.kind === "diarization"));
  const DEVICES = [
    ["auto", "settings.deviceAuto"],
    ["cpu", "settings.deviceCpu"],
    ["gpu", "settings.deviceGpu"],
  ] as const;
</script>

<section>
  <PageHeader title={t("settings.title")} />

  <div>
    <h2>{t("settings.general")}</h2>
    <div class="panel">
      <div class="row">
        <div>
          <strong>{t("settings.appearance")}</strong>
          <p class="muted">{t("settings.appearanceHint")}</p>
        </div>
        <ThemeSwitcher />
      </div>
      <div class="row">
        <div>
          <label for="language"><strong>{t("settings.language")}</strong></label>
          <p class="muted">{t("settings.languageHint")}</p>
        </div>
        <select
          id="language"
          class="field"
          value={i18n.preference}
          onchange={(event) => i18n.set(event.currentTarget.value)}
        >
          <option value="system">{t("settings.languageSystem")}</option>
          {#each LOCALES as locale (locale.code)}
            <option value={locale.code}>{locale.name}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <div>
    <h2>{t("settings.transcription")}</h2>
    <div class="panel">
      <SettingRow id="model" label={t("settings.model")} hint={t("settings.modelHint")}>
        <select
          id="model"
          class="field"
          value={settings.value.model ?? ""}
          onchange={(event) => settings.update({ model: event.currentTarget.value || null })}
        >
          <option value="">{t("settings.modelAuto")}</option>
          {#each speechModels as model (model.id)}
            <option value={model.id}>{model.name}</option>
          {/each}
        </select>
      </SettingRow>
      <SettingRow
        id="spoken"
        label={t("settings.transcriptionLanguage")}
        hint={t("settings.transcriptionLanguageHint")}
      >
        <select
          id="spoken"
          class="field"
          value={settings.value.language ?? ""}
          onchange={(event) => settings.update({ language: event.currentTarget.value || null })}
        >
          <option value="">{t("settings.languageDetect")}</option>
          {#each languageOptions() as option (option.code)}
            <option value={option.code}>{option.label}</option>
          {/each}
        </select>
      </SettingRow>
      <SettingRow id="identify" label={t("settings.identify")} hint={t("settings.identifyHint")}>
        <input
          id="identify"
          type="checkbox"
          checked={settings.value.identify_speakers}
          disabled={!hasSpeakerModel}
          onchange={(event) => settings.update({ identify_speakers: event.currentTarget.checked })}
        />
      </SettingRow>
      <SettingRow id="device" label={t("settings.device")} hint={t("settings.deviceHint")}>
        <select
          id="device"
          class="field"
          value={settings.value.device}
          onchange={(event) => settings.update({ device: event.currentTarget.value as Device })}
        >
          {#each DEVICES as [value, label] (value)}
            <option {value}>{t(label)}</option>
          {/each}
        </select>
      </SettingRow>
    </div>
    {#if settings.error}<p class="error" role="alert">{settings.error}</p>{/if}
  </div>

  <div>
    <h2>{t("settings.about")}</h2>
    <dl class="panel">
      <div class="row">
        <dt>{t("settings.version")}</dt>
        <dd>{info.value?.version ?? "—"}</dd>
      </div>
      <div class="row">
        <dt>{t("settings.engine")}</dt>
        <dd>{info.value?.coreVersion ?? "—"}</dd>
      </div>
      <div class="row">
        <dt>{t("settings.dataFolder")}</dt>
        <dd class="mono">{info.value?.dataDir ?? "—"}</dd>
      </div>
      <div class="row">
        <dt>{t("settings.notices")}</dt>
        <dd>
          <button class="btn" onclick={() => (showingNotices = true)}
            >{t("settings.showNotices")}</button
          >
        </dd>
      </div>
    </dl>
  </div>
</section>
{#if showingNotices}
  <NoticesDialog onclose={() => (showingNotices = false)} />
{/if}

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 760px;
    padding: 28px 36px;
  }

  h2 {
    margin-bottom: 8px;
    color: var(--text3);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
  }

  .panel {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 12px 18px;
    border-bottom: 1px solid var(--line);
  }

  .row:last-child {
    border-bottom: 0;
  }

  dd {
    user-select: text;
  }
</style>
