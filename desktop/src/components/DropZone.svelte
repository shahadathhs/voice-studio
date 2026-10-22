<script lang="ts">
  import { chooseFiles, onFilesDropped } from "../lib/api";
  import Icon from "./Icon.svelte";

  let { onfiles, disabled }: { onfiles: (paths: string[]) => void; disabled: boolean } = $props();

  async function choose(): Promise<void> {
    const paths = await chooseFiles();
    if (paths.length > 0) onfiles(paths);
  }

  $effect(() => {
    const unlisten = onFilesDropped((paths) => {
      if (!disabled) onfiles(paths);
    });
    return () => void unlisten.then((stop) => stop());
  });
</script>

<div class="drop">
  <span class="faint"><Icon name="down" size={20} /></span>
  <strong>Drop audio or video files here</strong>
  <p class="muted">WAV, MP3, M4A, FLAC, OGG, AAC, MP4, MOV, MKV</p>
  <button class="btn primary" {disabled} onclick={choose}
    ><Icon name="folder" />Choose files…</button
  >
  {#if disabled}<p class="muted small">Install a speech model in Models first.</p>{/if}
</div>

<style>
  .drop {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 200px;
    border: 1.5px dashed var(--line-strong);
    border-radius: 10px;
    background: var(--surface);
  }

  .drop strong {
    font-size: 15px;
  }

  .faint {
    color: var(--text3);
  }

  .small {
    font-size: 12px;
  }
</style>
