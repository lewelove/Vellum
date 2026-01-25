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
  <!-- Persistent nav sits on top of everything at the fixed width -->
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
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--background-main);
  }

  .nav-anchor {
    position: absolute;
    top: 0;
    left: 0;
    width: 150px;
    height: 80px; 
    z-index: 100;
  }

  .sidebar-pane {
    position: absolute;
    top: 80px; 
    left: 0;
    width: 150px;
    bottom: 0;
    border-right: 1px solid var(--border-muted);
    z-index: 10;
    background-color: var(--background-drawer);
    overflow-y: auto;
  }

  .content-pane {
    position: absolute;
    top: 0;
    left: 150px;
    right: 0;
    bottom: 0;
    overflow: hidden;
  }

  .fullscreen-pane {
    position: absolute;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 5;
  }
</style>
