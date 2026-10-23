<script lang="ts">
  let { value, label }: { value: number; label: string } = $props();
</script>

<div
  class="bar"
  role="progressbar"
  aria-label={label}
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={value}
>
  <div class="fill" style:width="{value}%"></div>
</div>

<style>
  .bar {
    height: 6px;
    border-radius: 3px;
    background: var(--line);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    background: var(--accent);
  }
</style>
