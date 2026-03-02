<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";
  import { pica } from "../../pica.js";
  import QueueTracks from "./QueueTracks.svelte";
  import QueueHud from "./QueueHud.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? library.getAlbumCoverUrl(activeId) : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);
  let canvasEl = $state(null);
  
  let isCanvasReady = $state(false);
  let isAlbumVisible = $state(false);
  
  let lastRenderKey = "";
  let lastRenderUrl = "";

  const barHeight = 48;
  const padding = 24;

  let boxSize = $derived(
    Math.max(0, Math.min(innerWidth - (padding * 2), innerHeight - (barHeight * 2) - (padding * 2)))
  );

  let boxX = $derived(Math.floor((innerWidth - boxSize) / 2));
  let boxY = $derived(Math.floor((innerHeight - (barHeight * 2) - boxSize) / 2));
  
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

      if (!isAlbumVisible) isAlbumVisible = true;

      const dpr = window.devicePixelRatio || 1;
      
      canvasEl.width = size * dpr;
      canvasEl.height = size * dpr;

      await pica.resize(img, canvasEl, {
        quality: 3,
        alpha: false,
        unsharpAmount: 0,
        features: [
          'js',
          'wasm',
          'ww'
        ]
      });

      isCanvasReady = true;

    } catch (err) {
      console.error("Pica Queue Render Failed:", err);
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
  
  <QueueHud>
    <div class="hud-internal-layout">
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
            <div class="hard-shadow" class:visible={isAlbumVisible} aria-hidden="true">
              <img src={coverUrl} alt="" style="width: 100%; height: 100%;" />
            </div>

            <img 
                src={coverUrl} 
                class="backing-image" 
                class:visible={isAlbumVisible}
                alt="" 
            />

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

      {#if sidebarWidth > 0}
        <div 
          class="tracks-sidebar" 
          style="
            width: {sidebarWidth}px; 
            visibility: {isQueueVisible ? 'visible' : 'hidden'};
            pointer-events: {isQueueVisible ? 'auto' : 'none'};
          "
        >
          <QueueTracks />
        </div>
      {/if}
    </div>
  </QueueHud>

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

  .hud-internal-layout {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: flex-end;
    pointer-events: none;
    position: relative;
  }

  .tracks-sidebar {
    height: 100%;
    display: flex;
    flex-direction: column;
    pointer-events: auto;
    background-color: transparent;
    z-index: 10;
  }

  .queue-stage {
    position: absolute;
    inset: 0;
    z-index: 5;
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
    font-size: 14px;
    letter-spacing: 0.4em;
    text-transform: uppercase;
    pointer-events: none;
  }
</style>
