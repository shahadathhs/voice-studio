<script lang="ts">
  import { onMount } from "svelte";

  import Sidebar from "./components/Sidebar.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import { info } from "./lib/app-info.svelte";
  import { jobs } from "./lib/jobs.svelte";
  import { library } from "./lib/library.svelte";
  import { models } from "./lib/models.svelte";
  import { nav } from "./lib/nav.svelte";
  import ComingSoonPage from "./pages/ComingSoonPage.svelte";
  import LibraryPage from "./pages/LibraryPage.svelte";
  import ModelsPage from "./pages/ModelsPage.svelte";
  import SettingsPage from "./pages/SettingsPage.svelte";
  import TranscribePage from "./pages/TranscribePage.svelte";
  import TranscriptPage from "./pages/TranscriptPage.svelte";

  onMount(() => {
    void info.load();
    void models.start();
    void jobs.start();
    void library.load();
  });
</script>

<div class="window">
  <div class="body">
    <Sidebar />
    <main>
      {#if nav.current === "transcribe"}
        <TranscribePage />
      {:else if nav.current === "speak"}
        <ComingSoonPage
          title="Speak"
          subtitle="Type or paste text, pick a voice, and listen or save it."
          text="Text to speech arrives in phase 4 of the roadmap."
        />
      {:else if nav.current === "library"}
        {#if nav.viewing !== null}
          <TranscriptPage id={nav.viewing} />
        {:else}
          <LibraryPage />
        {/if}
      {:else if nav.current === "models"}
        <ModelsPage />
      {:else}
        <SettingsPage />
      {/if}
    </main>
  </div>
  <StatusBar />
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  main {
    flex: 1;
    min-width: 0;
    overflow: auto;
  }
</style>
