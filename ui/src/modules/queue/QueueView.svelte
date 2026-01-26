<script>
  import { player } from "../player.svelte.js";
  import QueueTracks from "./QueueTracks.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? `/api/assets/${encodeURIComponent(activeId)}/cover` : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  // Constant Width based on window dimensions
  let sidebarWidth = $derived(Math.max(0, (innerWidth - innerHeight) / 2));
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="queue-view-container">
  <!-- 
    Cover area is the background layer, covering the full viewport.
    This ensures the image centers against the screen, not the remaining space.
  -->
  <div class="cover-area">
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

  <!-- 
    Tracks area overlayed on the right with fixed calculated width.
    The 1px border is removed.
  -->
  {#if sidebarWidth > 0}
    <div class="tracks-overlay" style="width: {sidebarWidth}px">
      <QueueTracks />
    </div>
  {/if}
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    position: relative;
    background-color: #242424;
    overflow: hidden;
  }

  .cover-area {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
  }

  .fullscreen-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .now-playing-cover {
    height: 100%;
    width: 100%;
    object-fit: contain;
    display: block;
  }

  .tracks-overlay {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    z-index: 10;
    background-color: var(--background-drawer);
    /* Shadow provides separation since the border was removed */
    /* box-shadow: -10px 0 30px rgba(0, 0, 0, 0.5); */
    overflow: hidden;
  }

  .empty-state {
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    font-weight: 500;
  }
</style>
