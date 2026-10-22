<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, subtitle, actions }: { title: string; subtitle?: string; actions?: Snippet } =
    $props();
</script>

<header>
  <div>
    <h1>{title}</h1>
    {#if subtitle}<p class="muted">{subtitle}</p>{/if}
  </div>
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  p {
    margin-top: 2px;
  }
</style>
