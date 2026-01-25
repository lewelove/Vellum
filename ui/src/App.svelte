<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";
  import QueueView from "$modules/queue/QueueView.svelte";
  import NavTabs from "$modules/navigation/NavTabs.svelte";

  let themeStyles = $derived(getThemeVariables());

  onMount(() => {
    library.init();
  });
</script>

<main style={themeStyles}>
  <!-- Fixed Navigation Top-Left -->
  <nav class="nav-anchor">
    <NavTabs />
  </nav>

  {#if nav.activeTab === 'home'}
    <aside class="sidebar-pane">
      <Sidebar />
    </aside>
    
    <section class="content-pane">
      <AlbumGrid />
    </section>
    
  {:else if nav.activeTab === 'queue'}
    <section class="fullscreen-pane">
      <QueueView />
    </section>
  {/if}
</main>

<style>
  main {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    position: relative;
    background-color: var(--background-main);
  }

  .nav-anchor {
    position: absolute;
    top: 0;
    left: 0;
    width: 150px;
    height: 80px; 
    z-index: 50; /* Above all content */
  }

  .sidebar-pane {
    position: absolute;
    top: 80px; /* Below nav-anchor */
    left: 0;
    width: 150px;
    bottom: 0;
    border-right: 1px solid var(--border-muted);
    z-index: 40;
    background-color: var(--background-drawer);
  }

  .content-pane {
    position: absolute;
    top: 0;
    left: 150px;
    right: 0;
    bottom: 0;
    overflow: hidden;
    z-index: 1;
  }

  .fullscreen-pane {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 0;
  }
</style>
