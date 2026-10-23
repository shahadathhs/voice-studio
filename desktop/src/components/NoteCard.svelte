<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { formatClock } from "../lib/format";
  import { languageName } from "../lib/languages";
  import type { Chapter, Note } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    note,
    onseek,
    ondelete,
  }: {
    note: Note;
    onseek: (ms: number) => void;
    ondelete: () => void;
  } = $props();

  let copied = $state(false);

  const chapters = $derived.by((): Chapter[] => {
    if (note.kind !== "chapters") return [];
    try {
      return JSON.parse(note.text) as Chapter[];
    } catch {
      return [];
    }
  });
  const isEmpty = $derived(note.kind === "actions" && note.text.trim() === "");
  const shownText = $derived(
    note.kind === "chapters"
      ? chapters.map((c) => `${formatClock(c.start_ms)}  ${c.title}`).join("\n")
      : note.text,
  );

  async function copy(): Promise<void> {
    await navigator.clipboard.writeText(shownText);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<section class="card" aria-label={t(`assist.kind.${note.kind}`)}>
  <header>
    <h3>
      {t(`assist.kind.${note.kind}`)}{note.language ? ` · ${languageName(note.language)}` : ""}
    </h3>
    <button class="icon" aria-label={t("assist.copyNote")} onclick={copy}>
      <Icon name={copied ? "check" : "copy"} size={14} />
    </button>
    <button class="icon" aria-label={t("assist.deleteNote")} onclick={ondelete}>
      <Icon name="x" size={14} />
    </button>
  </header>
  {#if note.kind === "chapters"}
    <ol>
      {#each chapters as chapter (chapter.start_ms)}
        <li>
          <button class="time mono" onclick={() => onseek(chapter.start_ms)}>
            {formatClock(chapter.start_ms)}
          </button>
          {chapter.title}
        </li>
      {/each}
    </ol>
  {:else if isEmpty}
    <p class="muted">{t("assist.noActions")}</p>
  {:else}
    <pre>{note.text}</pre>
  {/if}
  <p class="muted meta">{t("assist.madeBy", { model: note.model })}</p>
</section>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    border: 1px solid var(--line);
    border-radius: 8px;
  }

  header {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  h3 {
    flex: 1;
    font-size: 13px;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
  }

  ol {
    display: flex;
    flex-direction: column;
    gap: 4px;
    list-style: none;
  }

  .time {
    padding: 0 6px;
    border-radius: 10px;
    color: var(--accent);
    font-size: 12px;
  }

  .time:hover {
    background: var(--accent-bg);
  }

  pre {
    font: inherit;
    font-size: 13px;
    line-height: 1.55;
    white-space: pre-wrap;
    user-select: text;
  }

  .meta {
    font-size: 11px;
  }
</style>
