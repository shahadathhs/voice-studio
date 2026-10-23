<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import ThemeSwitcher from "../components/ThemeSwitcher.svelte";
  import { info } from "../lib/app-info.svelte";
  import { i18n, LOCALES, t } from "../lib/i18n.svelte";
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
    </dl>
  </div>
</section>

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
