<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { baseName, formatClock, percent } from "../lib/format";
  import { i18n } from "../lib/i18n.svelte";
  import { languageName } from "../lib/languages";
  import type { TranscriptView } from "../lib/types";
  import Icon from "./Icon.svelte";

  let { view, onrename }: { view: TranscriptView; onrename: (from: string, to: string) => void } =
    $props();

  let editing = $state<string | null>(null);
  let draft = $state("");

  function edit(name: string): void {
    editing = name;
    draft = name;
  }

  function commit(): void {
    const name = draft.trim();
    if (editing && name && name !== editing) onrename(editing, name);
    editing = null;
  }

  /** Svelte action: focus the element when it appears. */
  function focus(node: HTMLInputElement): void {
    node.focus();
    node.select();
  }

  const total = $derived(view.speakers.reduce((sum, speaker) => sum + speaker.talk_ms, 0));
  const details = $derived([
    [t("details.source"), view.item.source_path ? baseName(view.item.source_path) : "—"],
    [t("details.model"), view.item.model ?? "—"],
    [t("details.language"), languageName(view.item.language)],
    [t("details.words"), view.word_count.toLocaleString(i18n.locale.code)],
  ]);
</script>

<aside>
  {#if view.speakers.length > 0}
    <h2>{t("speakers.title")}</h2>
    <ul>
      {#each view.speakers as speaker, index (speaker.name)}
        <li style:--colour="var(--sp{index % 4})">
          <div class="name">
            <span class="dot"></span>
            {#if editing === speaker.name}
              <input
                aria-label={t("speakers.newName", { name: speaker.name })}
                bind:value={draft}
                onblur={commit}
                onkeydown={(event) => {
                  if (event.key === "Enter") commit();
                  if (event.key === "Escape") editing = null;
                }}
                use:focus
              />
            {:else}
              <span class="label-text">{speaker.name}</span>
              <button
                class="edit"
                aria-label={t("speakers.rename", { name: speaker.name })}
                title={t("speakers.renameHint")}
                onclick={() => edit(speaker.name)}
              >
                <Icon name="edit" size={14} />
              </button>
            {/if}
          </div>
          <div class="bar"><div style:width="{percent(speaker.talk_ms, total)}%"></div></div>
          <div class="muted small mono">
            {formatClock(speaker.talk_ms)} · {percent(speaker.talk_ms, total)}%
          </div>
        </li>
      {/each}
    </ul>
  {/if}
  <h2>{t("details.title")}</h2>
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

  .label-text {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .edit {
    margin-left: auto;
    color: var(--text3);
  }

  .edit:hover {
    color: var(--text);
  }

  input {
    flex: 1;
    min-width: 0;
    height: 24px;
    padding: 0 6px;
    border: 1px solid var(--line-strong);
    border-radius: 4px;
    background: var(--surface);
    font: inherit;
    color: var(--text);
    user-select: text;
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
