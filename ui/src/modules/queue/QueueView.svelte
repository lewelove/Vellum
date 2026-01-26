<script>
  import { player } from "../player.svelte.js";
  import QueueTracks from "./QueueTracks.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? `/api/assets/${encodeURIComponent(activeId)}/cover` : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  let sidebarWidth = $derived(Math.max(0, (innerWidth - innerHeight) / 2));
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<!-- Hidden SVG Filter Definition -->
<svg style="position: absolute; width: 0; height: 0;" aria-hidden="true">
  <filter id="dithered-shadow">
    <!-- 1. Create the shadow shape from the image's alpha channel -->
    <feGaussianBlur in="SourceAlpha" stdDeviation="12" result="blur" />
    
    <!-- 2. Generate the noise/dither grain -->
    <feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="5" result="noise" />
    
    <!-- 3. Limit the noise only to the blurred shadow area -->
    <feComposite in="noise" in2="blur" operator="in" result="dithered-blur" />
    
    <!-- 4. Adjust the density and color of the dithered shadow -->
    <feColorMatrix in="dithered-blur" type="matrix" 
      values="0 0 0 0 0
              0 0 0 0 0
              0 0 0 0 0
              0 0 0 0.8 0" /> <!-- 0.4 controls shadow opacity -->
    
    <!-- 5. Merge the original image back on top of the dithered shadow -->
    <feMerge>
      <feMergeNode />
      <feMergeNode in="SourceGraphic" />
    </feMerge>
  </filter>
</svg>

<div class="queue-view-container">
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
    background-color: var(--background-main);
    overflow: hidden;
  }

  .cover-area {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 100vw;
    margin-left: calc(-1 * var(--sidebar-offset, 0px));
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
    /* padding: 32px; */
    box-sizing: border-box;
  }

  .fullscreen-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    /* Apply the SVG filter here */
    filter: url(#dithered-shadow);
  }

  .now-playing-cover {
    max-height: 100%;
    max-width: 100%;
    object-fit: contain;
    display: block;
  }

  .tracks-overlay {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    z-index: 10;
    background-color: transparent;
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
