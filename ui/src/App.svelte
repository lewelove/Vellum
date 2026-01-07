<script>
  import { onMount } from "svelte";
  import { library } from "$lib/state/library.svelte.js";
  import { getThemeVariables } from "./theme.svelte.js";
  import AlbumGrid from "$lib/modules/album-grid/AlbumGrid.svelte";

  let themeStyles = $derived(getThemeVariables());

  onMount(() => {
    library.load();
  });
</script>

<main style={themeStyles}>

  <!-- <aside class="sidebar"> -->
  <!--   <!-- Navigation/Filters will go here -->
  <!-- </aside> -->

  <AlbumGrid />

  <!-- <aside class="details"> -->
  <!--   <!-- Cover/Tracklist will go here -->
  <!-- </aside> -->

</main>

<style>
  main {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--background-main);
  }

  .sidebar {
    width: 240px;
    height: 100%;
    background-color: var(--palette-100);
    border-right: 1px solid var(--border-muted);
    flex-shrink: 0;
  }

  .content-area {
    flex: 1;
    height: 100%;
    position: relative;
    overflow: hidden;
  }

  .details {
    width: 450px;
    height: 100%;
    background-color: var(--background-main);
    border-left: 1px solid var(--border-muted);
    flex-shrink: 0;
  }
</style>
