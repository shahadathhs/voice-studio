<script lang="ts">
  import EmptyState from "../components/EmptyState.svelte";
  import PageHeader from "../components/PageHeader.svelte";

  let { title, subtitle, text }: { title: string; subtitle: string; text: string } = $props();
</script>

<section>
  <PageHeader {title} {subtitle} />
  <EmptyState title="Not available yet" {text} />
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: 28px 36px;
  }
</style>
