<script>
  import { onMount } from "svelte";
  import { library } from "$state/library.svelte.js";
  import { getThemeVariables } from "$core/theme.svelte.js";
  import AlbumGrid from "$modules/album-grid/AlbumGrid.svelte";

  let themeStyles = $derived(getThemeVariables());

  onMount(() => {
    library.load();
  });
</script>

<main style={themeStyles}>
  <AlbumGrid />
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
