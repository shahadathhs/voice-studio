<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { thirdPartyNotices } from "../lib/api-text";
  import Icon from "./Icon.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement | undefined = $state();
  let text = $state("");

  $effect(() => dialog?.showModal());
  $effect(() => {
    void thirdPartyNotices().then((notices) => (text = notices));
  });
</script>

<dialog bind:this={dialog} aria-labelledby="notices-title" {onclose} oncancel={onclose}>
  <header>
    <h2 id="notices-title">{t("settings.notices")}</h2>
    <button class="icon" aria-label={t("common.close")} onclick={onclose}><Icon name="x" /></button>
  </header>
  <pre>{text || t("common.loading")}</pre>
</dialog>

<style>
  dialog {
    width: 760px;
    max-width: calc(100vw - 48px);
    margin: auto;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface);
    color: var(--text);
  }

  dialog::backdrop {
    background: rgb(0 0 0 / 0.45);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 22px;
    border-bottom: 1px solid var(--line);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
  }

  pre {
    max-height: 60vh;
    padding: 16px 22px;
    overflow: auto;
    font: 12px/1.5 var(--mono);
    white-space: pre-wrap;
    user-select: text;
  }
</style>
