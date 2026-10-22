<script lang="ts">
  import { baseName, formatClock, percent } from "../lib/format";
  import { languageName } from "../lib/languages";
  import type { TranscriptView } from "../lib/types";

  let { view }: { view: TranscriptView } = $props();

  const total = $derived(view.speakers.reduce((sum, speaker) => sum + speaker.talk_ms, 0));
  const details = $derived([
    ["Source", view.item.source_path ? baseName(view.item.source_path) : "—"],
    ["Model", view.item.model ?? "—"],
    ["Language", languageName(view.item.language)],
    ["Words", view.word_count.toLocaleString()],
  ]);
</script>

<aside>
  {#if view.speakers.length > 0}
    <h2>Speakers</h2>
    <ul>
      {#each view.speakers as speaker, index (speaker.name)}
        <li style:--colour="var(--sp{index % 4})">
          <div class="name"><span class="dot"></span>{speaker.name}</div>
          <div class="bar"><div style:width="{percent(speaker.talk_ms, total)}%"></div></div>
          <div class="muted small mono">
            {formatClock(speaker.talk_ms)} · {percent(speaker.talk_ms, total)}%
          </div>
        </li>
      {/each}
    </ul>
  {/if}
  <h2>Details</h2>
  <dl>
    {#each details as [label, value] (label)}
      <dt class="muted">{label}</dt>
      <dd>{value}</dd>
    {/each}
  </dl>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: 280px;
    padding: 20px 22px;
    border-left: 1px solid var(--line);
    overflow: auto;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 14px;
    list-style: none;
  }

  .name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--colour);
  }

  .dot {
    width: 9px;
    height: 9px;
    border-radius: 2px;
    background: var(--colour);
  }

  .bar {
    height: 5px;
    margin: 6px 0 4px;
    border-radius: 3px;
    background: var(--line);
    overflow: hidden;
  }

  .bar div {
    height: 100%;
    background: var(--colour);
  }

  .small {
    font-size: 12px;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 12px;
  }

  dd {
    text-align: right;
    overflow-wrap: anywhere;
    user-select: text;
  }
</style>
