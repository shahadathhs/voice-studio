<script lang="ts">
  import Notice from "../components/Notice.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";
  import { formatBytes, percent } from "../lib/format";
  import { i18n, t } from "../lib/i18n.svelte";
  import { groupModels } from "../lib/model-groups";
  import { models } from "../lib/models.svelte";
  import { onboarding } from "../lib/onboarding.svelte";

  const groups = $derived(groupModels(models.items ?? []));
</script>

<main>
  <div class="card">
    <h1>{t("onboarding.title")}</h1>
    <p class="muted lead">{t("onboarding.lead")}</p>

    {#each groups as { group, models: members } (group.kind)}
      <h2>{t(group.title)}</h2>
      <ul class="panel">
        {#each members as model (model.id)}
          {@const progress = models.progress[model.id]}
          <li>
            <label class="option">
              <input
                type="checkbox"
                checked={model.installed || onboarding.isSelected(model)}
                disabled={model.installed || onboarding.downloading}
                onchange={() => onboarding.toggle(model)}
              />
              <span class="text">
                <span class="name">
                  <strong>{model.name}</strong>
                  {#if model.id === group.recommended}
                    <span class="badge">{t("models.recommended")}</span>
                  {/if}
                </span>
                <span class="muted">{i18n.tOr(`modelInfo.${model.id}`, model.description)}</span>
                {#if progress}
                  <ProgressBar
                    value={percent(progress.downloaded, progress.total)}
                    label={t("models.downloading", { name: model.name })}
                  />
                {/if}
              </span>
              <span class="mono muted">
                {model.installed ? t("models.installed") : formatBytes(model.size_bytes)}
              </span>
            </label>
            {#if models.errors[model.id]}
              <p class="error" role="alert">{models.errors[model.id]}</p>
            {/if}
          </li>
        {/each}
      </ul>
    {/each}

    <footer>
      <span class="muted">
        {#if onboarding.downloading}
          {t("onboarding.downloading")}
        {:else}
          {t("onboarding.total", { size: formatBytes(onboarding.remainingBytes) })}
        {/if}
      </span>
      <span class="grow"></span>
      <button class="btn" onclick={() => onboarding.skip()}>{t("onboarding.skip")}</button>
      <button
        class="btn primary"
        disabled={!onboarding.canStart || onboarding.downloading}
        onclick={() => onboarding.start()}
      >
        {t("onboarding.download")}
      </button>
    </footer>
    <Notice>{t("onboarding.privacy")}</Notice>
  </div>
</main>

<style>
  main {
    height: 100%;
    overflow: auto;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 680px;
    margin: 0 auto;
    padding: 44px 24px;
  }

  .lead {
    margin-bottom: 8px;
  }

  h2 {
    margin-top: 10px;
    color: var(--text3);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
  }

  .panel {
    margin: 0;
    padding: 0;
    list-style: none;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
  }

  li {
    border-bottom: 1px solid var(--line);
  }

  li:last-child {
    border-bottom: 0;
  }

  .option {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 16px;
    cursor: pointer;
  }

  .option input {
    margin-top: 3px;
    accent-color: var(--accent);
  }

  .name {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .text {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 3px;
  }

  .badge {
    padding: 1px 7px;
    border-radius: 10px;
    background: var(--accent-bg);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }

  footer {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 10px;
  }

  .grow {
    flex: 1;
  }

  .error {
    margin: 0 16px 10px 44px;
    color: var(--danger);
    user-select: text;
  }
</style>
