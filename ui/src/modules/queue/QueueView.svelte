<script>
  import { player } from "../player.svelte.js";
  import QueueTracks from "./QueueTracks.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? `/api/assets/${encodeURIComponent(activeId)}/cover` : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  // Requirement: maximum possible width set to (window width - window height) / 2
  let sidebarWidth = $derived(Math.max(0, (innerWidth - innerHeight) / 2));
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="queue-view-container">
  <!-- 
    Cover area width is set to 100vw and shifted left by the sidebar-offset.
    This ensures the cover is centered relative to the whole screen even
    if the parent container is pushed to the right.
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
    Tracks area overlayed on the right with the calculated sidebarWidth.
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
    background-color: #121212;
    overflow: hidden;
  }

  .cover-area {
    position: absolute;
    top: 0;
    bottom: 0;
    /* width 100vw ensures the image-centering logic sees the full screen */
    width: 100vw;
    /* margin-left negates the 'left' offset applied to the viewport in App.svelte */
    margin-left: calc(-1 * var(--sidebar-offset, 0px));
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
    overflow: hidden;
    /* box-shadow: -10px 0 30px rgba(0, 0, 0, 0.3); */
  }

  .empty-state {
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    font-weight: 500;
  }
</style>
