<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import LibraryRow from "../components/LibraryRow.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { library } from "../lib/library.svelte";

  const count = $derived(library.items?.length ?? 0);
</script>

<section>
  <PageHeader
    title="Library"
    subtitle={count === 1 ? "1 transcript" : `${count} transcripts, searchable by what was said`}
  >
    {#snippet actions()}
      <label class="search">
        <Icon name="search" />
        <input
          type="search"
          placeholder="Search transcripts"
          aria-label="Search transcripts"
          value={library.query}
          oninput={(event) => library.search(event.currentTarget.value)}
        />
      </label>
    {/snippet}
  </PageHeader>

  {#if library.items === null}
    <p class="muted">Loading…</p>
  {:else if library.hits !== null}
    {#if library.hits.length === 0}
      <EmptyState title="No matches" text="No transcript contains all of those words." />
    {:else}
      <ul class="list" aria-label="Search results">
        {#each library.hits as hit (hit.item.id)}
          <LibraryRow item={hit.item} snippet={hit.snippet} />
        {/each}
      </ul>
    {/if}
  {:else if library.items.length === 0}
    <EmptyState title="Nothing here yet" text="Transcribe a file and it will be saved here." />
  {:else}
    <ul class="list" aria-label="Transcripts">
      {#each library.items as item (item.id)}<LibraryRow {item} />{/each}
    </ul>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 28px 36px;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text3);
  }

  input {
    width: 220px;
    border: 0;
    outline: 0;
    background: none;
    font: inherit;
    color: var(--text);
    user-select: text;
  }

  .list {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
    overflow: hidden;
  }
</style>
