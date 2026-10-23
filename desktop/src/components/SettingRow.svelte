<script lang="ts">
  import type { Snippet } from "svelte";

  /** One line of the Settings page: what it is, a hint, and the control on the right. */
  let {
    label,
    hint,
    id,
    children,
  }: { label: string; hint: string; id: string; children: Snippet } = $props();
</script>

<div class="row">
  <div>
    <label for={id}><strong>{label}</strong></label>
    <p class="muted">{hint}</p>
  </div>
  {@render children()}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 12px 18px;
    border-bottom: 1px solid var(--line);
  }

  .row:last-child {
    border-bottom: 0;
  }
</style>
