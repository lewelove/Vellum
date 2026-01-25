<script>
  import { player } from "../player.svelte.js";

  let activeId = $derived(player.currentAlbumId);
  
  let coverUrl = $derived(
    activeId ? `/api/assets/${encodeURIComponent(activeId)}/cover` : ""
  );
</script>

<div class="queue-view-container">
  {#if coverUrl}
    <div class="fullscreen-cover">
      <img src={coverUrl} alt="Now Playing" class="now-playing-cover" />
    </div>
  {:else}
    <div class="empty-state">
      <span>Not Playing</span>
    </div>
  {/if}
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    position: relative;
    background-color: var(--background-drawer);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .fullscreen-cover {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--background-drawer);
  }

  .now-playing-cover {
    height: 100vh;
    width: auto;
    max-width: 100vw;
    object-fit: contain;
    display: block;
  }

  .empty-state {
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    font-weight: 500;
  }
</style>
