<script lang="ts">
  let { title, text }: { title: string; text: string } = $props();
</script>

<div class="empty">
  <strong>{title}</strong>
  <p class="muted">{text}</p>
</div>

<style>
  .empty {
    padding: 36px 20px;
    text-align: center;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
  }

  p {
    margin-top: 4px;
  }
</style>
