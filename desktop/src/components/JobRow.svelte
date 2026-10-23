<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { formatClock, percent } from "../lib/format";
  import { jobs } from "../lib/jobs.svelte";
  import { nav } from "../lib/nav.svelte";
  import type { MessageKey } from "../locales/en";
  import type { JobInfo, JobState } from "../lib/types";
  import Icon from "./Icon.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { job }: { job: JobInfo } = $props();

  const LABELS: Record<JobState, MessageKey> = {
    waiting: "job.waiting",
    running: "job.running",
    paused: "job.paused",
    done: "job.done",
    failed: "job.failed",
    cancelled: "job.cancelled",
  };

  const finished = $derived(
    job.state === "done" || job.state === "failed" || job.state === "cancelled",
  );
  const done = $derived(percent(job.processed_ms, job.total_ms));
</script>

<li>
  <div class="top">
    <strong class="title">{job.title}</strong>
    <span class="badge {job.state}">{t(LABELS[job.state])}</span>
    <span class="spacer"></span>
    {#if job.state === "waiting"}
      <button
        class="icon"
        aria-label={t("job.moveUp", { title: job.title })}
        onclick={() => jobs.moveUp(job.id)}
      >
        <Icon name="up" />
      </button>
    {/if}
    {#if job.state === "running" || job.state === "waiting"}
      <button class="btn" onclick={() => jobs.cancel(job.id)}>{t("common.cancel")}</button>
    {/if}
    {#if job.state === "done" && job.item_id !== null}
      {@const itemId = job.item_id}
      <button class="btn" onclick={() => nav.open(itemId)}>{t("job.open")}</button>
    {/if}
    {#if finished || job.state === "waiting"}
      <button
        class="icon"
        aria-label={t("job.remove", { title: job.title })}
        onclick={() => jobs.remove(job.id)}
      >
        <Icon name="x" />
      </button>
    {/if}
  </div>
  {#if job.state === "running"}
    <ProgressBar value={done} label={t("job.progressLabel", { title: job.title })} />
    <div class="muted mono small">
      {t("job.processed", {
        done: formatClock(job.processed_ms),
        total: formatClock(job.total_ms),
      })}
    </div>
  {/if}
  {#if job.error}<div class="error small">{job.error}</div>{/if}
</li>

<style>
  li {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--line);
    list-style: none;
  }

  li:last-child {
    border-bottom: 0;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .spacer {
    flex: 1;
  }

  .badge {
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--line);
    color: var(--text2);
    font-size: 11.5px;
    font-weight: 600;
  }

  .badge.running,
  .badge.done {
    background: var(--accent-bg);
    color: var(--accent);
  }

  .badge.failed {
    color: var(--danger);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    color: var(--text2);
  }

  .icon:hover {
    background: var(--line);
  }

  .small {
    font-size: 12px;
  }

  .error {
    color: var(--danger);
    user-select: text;
  }
</style>
