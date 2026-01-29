<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import QueueTracks from "./QueueTracks.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? library.getAlbumCoverUrl(activeId) : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  let sidebarWidth = $derived(Math.max(0, (innerWidth - innerHeight) / 2));
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<!-- Hidden SVG Filter Definition -->
<svg style="position: absolute; width: 0; height: 0;" aria-hidden="true">
  <filter id="dithered-shadow" x="-50%" y="-50%" width="200%" height="200%">
    <!-- Process Alpha for Shadow -->
    <feGaussianBlur in="SourceAlpha" stdDeviation="16" result="blur" />
    
    <!-- Generate Noise -->
    <feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="4" result="noise" />
    
    <!-- Composite Noise into Blur Shape -->
    <feComposite in="noise" in2="blur" operator="in" result="dithered-blur" />
    
    <!-- Color and Opacity Control -->
    <feColorMatrix in="dithered-blur" type="matrix" 
      values="0 0 0 0 0
              0 0 0 0 0
              0 0 0 0 0
              0 0 0 1.2 0" />
    
    <!-- 
      SourceGraphic is explicitly omitted from the filter chain 
      to prevent color space interference during rendering.
    -->
  </filter>
</svg>

<div class="queue-view-container">
  <div class="cover-area">
    {#if coverUrl}
      <div class="fullscreen-cover">
        <!-- 
          Filtered Shadow Layer 
          Isolating the filter prevents the browser from passing the 
          primary image through the SVG color management pipeline.
        -->
        <div class="shadow-backdrop" aria-hidden="true">
          <img src={coverUrl} alt="" />
        </div>

        <!-- Clean Image Layer -->
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
    background-color: var(--background-drawer);
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
    box-sizing: border-box;
  }

  .fullscreen-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .shadow-backdrop {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
    filter: url(#dithered-shadow);
    pointer-events: none;
  }

  .shadow-backdrop img {
    max-height: 100%;
    max-width: 100%;
    object-fit: contain;
    display: block;
    opacity: 1;
  }

  .now-playing-cover {
    position: relative;
    z-index: 2;
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
