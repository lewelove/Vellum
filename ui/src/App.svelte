<script>
  import { onMount } from "svelte";
  import { library } from "$lib/state/library.svelte.js";
  import AlbumGrid from "$lib/modules/album-grid/AlbumGrid.svelte";

  onMount(() => {
    library.load();
  });
</script>

<main>

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
    background-color: var(--bg-color);
  }

  .sidebar {
    width: 240px;
    height: 100%;
    background-color: var(--0);
    border-right: 1px solid var(--50);
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
    background-color: var(--card-bg);
    border-left: 1px solid var(--50);
    flex-shrink: 0;
  }
</style>
