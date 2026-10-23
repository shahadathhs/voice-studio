<script lang="ts">
  import type { Snippet } from "svelte";

  import Icon from "./Icon.svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="notice" role="note">
  <Icon name="lock" />
  <p>{@render children()}</p>
</div>

<style>
  .notice {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text2);
  }
</style>
