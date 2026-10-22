<script lang="ts">
  import { nav } from "../lib/nav.svelte";
  import type { PageEntry } from "../lib/pages";
  import Icon from "./Icon.svelte";

  let { page }: { page: PageEntry } = $props();
  const active = $derived(nav.current === page.id);
</script>

<button
  class="item"
  class:on={active}
  aria-current={active ? "page" : undefined}
  onclick={() => nav.go(page.id)}
>
  <Icon name={page.icon} />
  {page.label}
</button>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    margin-bottom: 2px;
    border-radius: 6px;
    color: var(--text2);
    font-weight: 500;
    text-align: left;
  }

  .item.on {
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 0 0 1px var(--line);
  }
</style>
