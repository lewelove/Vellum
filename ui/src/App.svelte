<script>
  import { onMount } from "svelte";
  import { library } from "$state/library.svelte.js";
  import { getThemeVariables } from "$core/theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";
  import Sidebar from "$modules/sidebar/Sidebar.svelte";

  let themeStyles = $derived(getThemeVariables());

  onMount(() => {
    // Initialize capabilities, then load library
    library.init();
  });
</script>

<main style={themeStyles}>
  <aside class="sidebar-pane">
    <Sidebar />
  </aside>
  
  <section class="content-pane">
    <AlbumGrid />
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
    flex: 0 0 260px; /* Fixed Width */
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
