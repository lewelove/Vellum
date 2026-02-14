<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";
  import { pica } from "../../pica.js";
  import QueueTracks from "./QueueTracks.svelte";
  import QueueRowTop from "./QueueRowTop.svelte";
  import QueueRowBottom from "./QueueRowBottom.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? library.getAlbumCoverUrl(activeId) : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);
  let canvasEl = $state(null);
  
  // Controls the visibility of the High-Res Canvas
  let isCanvasReady = $state(false);
  // Controls the global fade-in of the album art (backing image + shadow)
  let isAlbumVisible = $state(false);
  
  let lastRenderKey = "";
  let lastRenderUrl = "";

  const coverMargin = 20;

  let boxSize = $derived(
    Math.max(0, Math.min(innerWidth - (coverMargin * 2), innerHeight - (coverMargin * 2)))
  );

  let boxX = $derived(Math.floor((innerWidth - boxSize) / 2));
  let boxY = $derived(Math.floor((innerHeight - boxSize) / 2));
  
  let sidebarWidth = $derived(Math.max(0, (innerWidth - boxSize) / 2));
  let isQueueVisible = $derived(nav.activeTab === "queue");

  async function renderCover(url, size) {
    if (!url || !size || !canvasEl || size <= 0) {
      isAlbumVisible = false;
      isCanvasReady = false;
      return;
    }

    const renderKey = `${url}-${size}`;
    if (renderKey === lastRenderKey) return;
    
    lastRenderKey = renderKey;

    // Only fade out the entire stack if the album actually changed.
    // If just resizing, we keep the backing image visible to prevent flashing.
    if (url !== lastRenderUrl) {
        isAlbumVisible = false;
        isCanvasReady = false;
        lastRenderUrl = url;
    }

    try {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.src = url;
      
      await img.decode();

      // If this is a new album, show the backing elements now that we have the source
      if (!isAlbumVisible) isAlbumVisible = true;

      const dpr = window.devicePixelRatio || 1;
      
      // Resizing the canvas clears it. 
      // Because we have a backing <img>, the user sees that instead of a flash.
      canvasEl.width = size * dpr;
      canvasEl.height = size * dpr;

      await pica.resize(img, canvasEl, {
        quality: 3,
        alpha: false,
        unsharpAmount: 0,
        features: ['js', 'wasm', 'ww']
      });

      // Show the high-quality canvas once ready
      isCanvasReady = true;

    } catch (err) {
      console.error("Pica Queue Render Failed:", err);
      // Fallback: ensure at least the backing image is visible
      isAlbumVisible = true;
    }
  }

  $effect(() => {
    renderCover(coverUrl, boxSize);
  });
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<svg style="position: absolute; width: 0; height: 0;" aria-hidden="true">
  <filter id="dithered-shadow" x="-20%" y="-20%" width="140%" height="140%">
    <feGaussianBlur in="SourceAlpha" stdDeviation="12" result="blur" />
    <feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" result="noise" />
    <feComposite in="noise" in2="blur" operator="in" result="dithered-blur" />
    <feColorMatrix in="dithered-blur" type="matrix" 
      values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1.5 0" />
  </filter>
</svg>

<div class="queue-layout">
  <div class="ui-layer">
    <div class="row-top-wrapper">
      <QueueRowTop />
    </div>

    {#if sidebarWidth > 0}
      <div 
        class="tracks-overlay" 
        style="
          width: {sidebarWidth}px; 
          visibility: {isQueueVisible ? 'visible' : 'hidden'};
          pointer-events: {isQueueVisible ? 'auto' : 'none'};
        "
      >
        <QueueTracks />
      </div>
    {/if}

    <div class="row-bottom-wrapper">
      <QueueRowBottom />
    </div>
  </div>

  <div class="queue-stage">
    {#if coverUrl && boxSize > 0}
      <div 
        class="pixel-stage" 
        style="
          width: {boxSize}px; 
          height: {boxSize}px; 
          left: {boxX}px;
          top: {boxY}px;
        "
      >
        <!-- Layer 1: Dithered Shadow -->
        <!-- Only fades out on Album Change -->
        <div class="hard-shadow" class:visible={isAlbumVisible} aria-hidden="true">
          <img src={coverUrl} alt="" style="width: 100%; height: 100%;" />
        </div>

        <!-- Layer 2: Backing Image (Browser Scaling) -->
        <!-- Prevents flashing when Canvas is cleared during resize -->
        <!-- Only fades out on Album Change -->
        <img 
            src={coverUrl} 
            class="backing-image" 
            class:visible={isAlbumVisible}
            alt="" 
        />

        <!-- Layer 3: High Quality Canvas (Pica) -->
        <!-- Fades in over the backing image when processing is done -->
        <canvas 
          bind:this={canvasEl}
          class="raw-canvas"
          class:visible={isCanvasReady}
          style="width: {boxSize}px; height: {boxSize}px;"
        ></canvas>
      </div>
    {:else if player.state !== 'stop' && library.isLoading}
      <div class="empty-state">
        <span>Syncing Library...</span>
      </div>
    {:else if !activeId || player.state === 'stop'}
      <div class="empty-state">
        <span>Not Playing</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .queue-layout {
    width: 100%;
    height: 100%;
    position: relative;
    background-color: var(--background-drawer);
    overflow: hidden;
    contain: paint;
  }

  .ui-layer {
    position: absolute;
    inset: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    pointer-events: none;
  }

  .row-top-wrapper, .row-bottom-wrapper {
    pointer-events: auto;
    flex-shrink: 0;
  }

  .tracks-overlay {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-self: flex-end;
    pointer-events: auto;
    background-color: transparent;
  }

  .queue-stage {
    position: absolute;
    inset: 0;
    z-index: 20;
    pointer-events: none;
  }

  .pixel-stage {
    position: absolute;
    overflow: visible;
  }

  .hard-shadow {
    position: absolute;
    inset: 0;
    z-index: 1;
    filter: url(#dithered-shadow);
    opacity: 0;
    transition: opacity 0.4s ease;
  }

  .hard-shadow.visible {
    opacity: 1;
  }

  .backing-image {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    z-index: 2;
    object-fit: fill;
    opacity: 0;
    transition: opacity 0.4s ease;
    will-change: opacity;
  }

  .backing-image.visible {
    opacity: 1;
  }

  .raw-canvas {
    position: relative;
    z-index: 3;
    display: block;
    opacity: 0;
    transition: opacity 0.4s ease;
    will-change: opacity;
  }

  .raw-canvas.visible {
    opacity: 1;
  }

  .empty-state {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    pointer-events: none;
  }
</style>
