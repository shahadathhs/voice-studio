<script lang="ts">
  import type { Snippet } from "svelte";

  import { t } from "../lib/i18n.svelte";
  import Icon from "./Icon.svelte";

  /**
   * Shows `children`, with a pencil that swaps it for a text field. Enter or leaving the field
   * saves, Escape cancels. `suggestions` makes the field offer those values (for picking an
   * existing speaker or typing a new one).
   */
  let {
    value,
    label,
    suggestions = [],
    multiline = false,
    onsave,
    children,
  }: {
    value: string;
    label: string;
    suggestions?: readonly string[];
    multiline?: boolean;
    onsave: (value: string) => void;
    children: Snippet;
  } = $props();

  let editing = $state(false);
  let draft = $state("");
  const listId = $props.id();

  function begin(): void {
    draft = value;
    editing = true;
  }

  function finish(save: boolean): void {
    const next = draft.trim();
    editing = false;
    if (save && next && next !== value) onsave(next);
  }

  /** Svelte action: focus the field when it appears. */
  function focus(node: HTMLInputElement | HTMLTextAreaElement): void {
    node.focus();
    node.select();
  }

  function keys(event: KeyboardEvent): void {
    if (event.key === "Escape") finish(false);
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      finish(true);
    }
  }
</script>

{#if editing}
  {#if multiline}
    <textarea
      class="field"
      rows="2"
      aria-label={label}
      bind:value={draft}
      onkeydown={keys}
      onblur={() => finish(true)}
      use:focus></textarea>
  {:else}
    <input
      class="field"
      aria-label={label}
      list={suggestions.length > 0 ? listId : undefined}
      bind:value={draft}
      onkeydown={keys}
      onblur={() => finish(true)}
      use:focus
    />
    {#if suggestions.length > 0}
      <datalist id={listId}>
        {#each suggestions as suggestion (suggestion)}<option value={suggestion}></option>{/each}
      </datalist>
    {/if}
  {/if}
{:else}
  <span class="shown">
    {@render children()}
    <button class="edit" aria-label={t("edit.label", { what: label })} onclick={begin}>
      <Icon name="edit" size={14} />
    </button>
  </span>
{/if}

<style>
  .shown {
    display: inline;
  }

  .edit {
    display: inline-flex;
    vertical-align: middle;
    margin-left: 6px;
    padding: 2px;
    border-radius: 4px;
    color: var(--text3);
    opacity: 0;
  }

  .shown:hover .edit,
  .edit:focus-visible {
    opacity: 1;
  }

  .edit:hover {
    background: var(--accent-bg);
    color: var(--accent);
  }

  textarea.field {
    width: 100%;
    height: auto;
    padding: 6px 10px;
    resize: vertical;
    line-height: 1.5;
  }
</style>
