<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { pica } from "../../pica.js";
  import QueueTracks from "./QueueTracks.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? library.getAlbumCoverUrl(activeId) : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);
  let canvasEl = $state(null);

  // INTEGER SNAP
  let boxSize = $derived(Math.floor(Math.min(innerWidth, innerHeight)));
  let boxX = $derived(Math.floor((innerWidth - boxSize) / 2));
  let boxY = $derived(Math.floor((innerHeight - boxSize) / 2));

  let sidebarWidth = $derived(Math.max(0, (innerWidth - innerHeight) / 2));

  // High-precision render loop
  async function renderCover(url, size) {
    if (!url || !size || !canvasEl) return;

    try {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.src = url;
      
      await img.decode();

      // Set canvas to physical integer pixels
      canvasEl.width = size;
      canvasEl.height = size;

      await pica.resize(img, canvasEl, {
        quality: 3,
        alpha: false,
        unsharpAmount: 0,
        features: ['js', 'wasm', 'ww']
      });

      // Strictly disable any remaining browser interpolation
      const ctx = canvasEl.getContext('2d');
      ctx.imageSmoothingEnabled = false;
    } catch (err) {
      console.error("Pica Queue Render Failed:", err);
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

<div class="queue-view-container">
  
  {#if coverUrl && boxSize > 0}
    <div 
      class="pixel-stage" 
      style="
        width: {boxSize}px; 
        height: {boxSize}px; 
        top: {boxY}px;
        left: {boxX}px;
      "
    >
      <!-- Shadow Layer (Still using image for the blur shape) -->
      <div class="hard-shadow" aria-hidden="true">
        <img src={coverUrl} alt="" style="width: 100%; height: 100%;" />
      </div>

      <!-- Canvas foreground: Bypass <img> tags entirely -->
      <canvas 
        bind:this={canvasEl}
        class="raw-canvas"
        style="width: {boxSize}px; height: {boxSize}px;"
      ></canvas>
    </div>

  {:else if !coverUrl}
    <div class="empty-state">
      <span>Not Playing</span>
    </div>
  {/if}

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

  .pixel-stage {
    position: absolute;
    z-index: 1;
    overflow: visible;
  }

  .hard-shadow {
    position: absolute;
    inset: 0;
    z-index: 1;
    filter: url(#dithered-shadow);
    /* Prevent shadow bleed-through at edges */
    clip-path: inset(1px); 
  }

  .raw-canvas {
    position: relative;
    z-index: 2;
    display: block;
    /* Eliminate all browser-level smoothing */
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    image-rendering: -moz-crisp-edges;
    /* Force Hardware Clipping to 0px boundary */
    clip-path: inset(0);
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
  }

  .tracks-overlay {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    z-index: 10;
    background-color: transparent;
  }
</style>
