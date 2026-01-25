<script>
  import { onMount } from "svelte";
  import { library } from "./library.svelte.js";
  import { nav } from "./navigation.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";
  import QueueView from "$modules/queue/QueueView.svelte";

  let themeStyles = $derived(getThemeVariables());

  onMount(() => {
    library.init();
  });
</script>

<main style={themeStyles}>
  <aside class="sidebar-pane">
    <Sidebar />
  </aside>
  
  <section class="content-pane">
    {#if nav.activeTab === 'home'}
      <AlbumGrid />
    {:else if nav.activeTab === 'queue'}
      <QueueView />
    {/if}
  </section>
</main>

<style>
  main {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--background-main);
  }

  .sidebar-pane {
    flex: 0 0 150px; 
    height: 100%;
    border-right: 1px solid var(--border-muted);
    z-index: 10;
  }

  .content-pane {
    flex: 1;
    height: 100%;
    position: relative;
    overflow: hidden;
  }
</style>
