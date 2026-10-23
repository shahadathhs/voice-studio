<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import EmptyState from "../components/EmptyState.svelte";
  import Icon from "../components/Icon.svelte";
  import LibraryRow from "../components/LibraryRow.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import { library } from "../lib/library.svelte";
  import { player } from "../lib/player.svelte";

  const count = $derived(library.items?.length ?? 0);

  $effect(() => () => player.stop());
</script>

<section>
  <PageHeader title={t("library.title")} subtitle={t("library.subtitle", { count })}>
    {#snippet actions()}
      <label class="search">
        <Icon name="search" />
        <input
          type="search"
          placeholder={t("library.search")}
          aria-label={t("library.search")}
          value={library.query}
          oninput={(event) => library.search(event.currentTarget.value)}
        />
      </label>
    {/snippet}
  </PageHeader>

  {#if player.error}<p class="error" role="alert">{player.error}</p>{/if}

  {#if library.items === null}
    <p class="muted">{t("common.loading")}</p>
  {:else if library.hits !== null}
    {#if library.hits.length === 0}
      <EmptyState title={t("library.noMatches.title")} text={t("library.noMatches.text")} />
    {:else}
      <ul class="list" aria-label={t("library.results")}>
        {#each library.hits as hit (hit.item.id)}
          <LibraryRow item={hit.item} snippet={hit.snippet} startMs={hit.start_ms} />
        {/each}
      </ul>
    {/if}
  {:else if library.items.length === 0}
    <EmptyState title={t("library.empty.title")} text={t("library.empty.text")} />
  {:else}
    <ul class="list" aria-label={t("library.list")}>
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

  .error {
    color: var(--danger);
    user-select: text;
  }

  .list {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
    overflow: hidden;
  }
</style>
