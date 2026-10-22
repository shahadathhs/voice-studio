<script lang="ts">
  import PageHeader from "../components/PageHeader.svelte";
  import ThemeSwitcher from "../components/ThemeSwitcher.svelte";
  import { info } from "../lib/app-info.svelte";
</script>

<section>
  <PageHeader title="Settings" />

  <div>
    <h2>General</h2>
    <div class="panel">
      <div class="row">
        <div>
          <strong>Appearance</strong>
          <p class="muted">Follows the system unless you choose.</p>
        </div>
        <ThemeSwitcher />
      </div>
    </div>
  </div>

  <div>
    <h2>About</h2>
    <dl class="panel">
      <div class="row">
        <dt>Version</dt>
        <dd>{info.value?.version ?? "—"}</dd>
      </div>
      <div class="row">
        <dt>Core engine</dt>
        <dd>{info.value?.coreVersion ?? "—"}</dd>
      </div>
      <div class="row">
        <dt>Data folder</dt>
        <dd class="mono">{info.value?.dataDir ?? "—"}</dd>
      </div>
    </dl>
  </div>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 760px;
    padding: 28px 36px;
  }

  h2 {
    margin-bottom: 8px;
    color: var(--text3);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
  }

  .panel {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
  }

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

  dd {
    user-select: text;
  }
</style>
