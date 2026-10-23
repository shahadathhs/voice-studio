<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import LevelMeter from "../components/LevelMeter.svelte";
  import LiveCaptions from "../components/LiveCaptions.svelte";
  import Notice from "../components/Notice.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import RecordingDone from "../components/RecordingDone.svelte";
  import { batch } from "../lib/batch.svelte";
  import { formatClock } from "../lib/format";
  import { nav } from "../lib/nav.svelte";
  import { record } from "../lib/record.svelte";

  const recording = $derived(record.status === "recording");
  const busy = $derived(record.status === "saving");
  const captioning = $derived(record.live && record.canCaption);
  const showCaptions = $derived(
    captioning && (recording || record.final !== "" || record.draft !== ""),
  );

  $effect(() => {
    void record.loadDevices();
  });
</script>

<section>
  <PageHeader title={t("record.title")} subtitle={t("record.subtitle")} />

  {#if record.devices.length === 0}
    <EmptyState title={t("record.noDevice.title")} text={t("record.noDevice.text")} />
  {:else}
    <div class="panel">
      <div class="row">
        <label class="label" for="record-device">{t("record.device")}</label>
        <select
          id="record-device"
          class="field"
          value={record.device?.name}
          disabled={recording || busy}
          onchange={(event) => record.chooseDevice(event.currentTarget.value)}
        >
          {#each record.devices as device (device.name)}
            <option value={device.name}>
              {device.name}{device.is_default ? ` (${t("record.default")})` : ""}
            </option>
          {/each}
        </select>
      </div>

      <div class="stage">
        <button
          class="big"
          class:on={recording}
          aria-label={recording ? t("record.stop") : t("record.start")}
          disabled={busy}
          onclick={() => (recording ? record.stop() : record.begin())}
        >
          {#if recording}<span class="square"></span>{:else}<Icon name="mic" size={30} />{/if}
        </button>
        <div class="timer mono" aria-live="off">{formatClock(record.elapsedMs)}</div>
        <p class="muted">
          {recording ? t("record.recording") : busy ? t("record.saving") : t("record.ready")}
        </p>
        <div class="meter"><LevelMeter level={record.level} /></div>
      </div>

      <label class="check">
        <input type="checkbox" bind:checked={record.live} disabled={recording || !batch.model} />
        {t("record.live")}
      </label>
      {#if !batch.model}
        <p class="hint muted">
          {t("record.needModel")}
          <button class="link" onclick={() => nav.go("models")}>{t("record.openModels")}</button>
        </p>
      {/if}
    </div>
  {/if}

  {#if record.error}<p class="error" role="alert">{record.error}</p>{/if}

  {#if showCaptions}
    <LiveCaptions final={record.final} draft={record.draft} waiting={recording} />
  {/if}

  <RecordingDone />

  <Notice>{t("record.privacy")}</Notice>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 28px 36px;
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 20px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 12px 0;
  }

  .big {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 72px;
    height: 72px;
    border: 0;
    border-radius: 50%;
    background: var(--accent);
    color: var(--accent-fg);
    cursor: pointer;
  }

  .big.on {
    background: var(--danger);
  }

  .big:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .square {
    width: 22px;
    height: 22px;
    border-radius: 3px;
    background: currentColor;
  }

  .timer {
    font-size: 28px;
  }

  .meter {
    width: min(360px, 100%);
  }

  .hint {
    margin-top: -8px;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
