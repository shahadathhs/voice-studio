<script lang="ts">
  import { MAIN_PAGES, SETTINGS_PAGE } from "../lib/pages";
  import NavItem from "./NavItem.svelte";
  import ThemeSwitcher from "./ThemeSwitcher.svelte";
</script>

<aside>
  <div class="brand">Voice Studio</div>
  <nav aria-label="Main">
    {#each MAIN_PAGES as page (page.id)}
      <NavItem {page} />
    {/each}
  </nav>
  <div class="grow"></div>
  <div class="theme"><ThemeSwitcher compact /></div>
  <NavItem page={SETTINGS_PAGE} />
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    width: 208px;
    flex: none;
    padding: 14px 10px;
    background: var(--sidebar);
    border-right: 1px solid var(--line);
  }

  .brand {
    padding: 2px 10px 14px;
    font-weight: 700;
  }

  .grow {
    flex: 1;
  }

  .theme {
    padding: 0 10px;
    margin-bottom: 10px;
  }
</style>
