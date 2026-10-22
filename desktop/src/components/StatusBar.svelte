<script lang="ts">
  import { info } from "../lib/app-info.svelte";
</script>

<footer>
  <span><span class="dot"></span>Offline — everything runs on this device</span>
  <span class="grow"></span>
  <span>{info.value?.version ?? ""}</span>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    gap: 18px;
    height: 28px;
    flex: none;
    padding: 0 14px;
    border-top: 1px solid var(--line);
    background: var(--sidebar);
    color: var(--text2);
    font-size: 12px;
  }

  .dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-right: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .grow {
    flex: 1;
  }
</style>
