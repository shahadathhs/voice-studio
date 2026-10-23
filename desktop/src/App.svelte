<script lang="ts">
  import { onMount } from "svelte";

  import Sidebar from "./components/Sidebar.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import { info } from "./lib/app-info.svelte";
  import { jobs } from "./lib/jobs.svelte";
  import { library } from "./lib/library.svelte";
  import { models } from "./lib/models.svelte";
  import { nav } from "./lib/nav.svelte";
  import { onboarding } from "./lib/onboarding.svelte";
  import { speech } from "./lib/speech.svelte";
  import OnboardingPage from "./pages/OnboardingPage.svelte";
  import LibraryPage from "./pages/LibraryPage.svelte";
  import ModelsPage from "./pages/ModelsPage.svelte";
  import SettingsPage from "./pages/SettingsPage.svelte";
  import SpeakPage from "./pages/SpeakPage.svelte";
  import TranscribePage from "./pages/TranscribePage.svelte";
  import TranscriptPage from "./pages/TranscriptPage.svelte";

  onMount(() => {
    void info.load();
    void models.start();
    void jobs.start();
    void library.load();
    void speech.start();
  });
</script>

<div class="window">
  {#if onboarding.needed}
    <OnboardingPage />
  {:else}
    <div class="body">
      <Sidebar />
      <main>
        {#if nav.current === "transcribe"}
          <TranscribePage />
        {:else if nav.current === "speak"}
          <SpeakPage />
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
  {/if}
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
