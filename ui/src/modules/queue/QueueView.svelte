<script>
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { pica } from "../../pica.js";
  
  import QueueTracks from "./QueueTracks.svelte";
  import QueueHud from "./QueueHud.svelte";

  // -- Data State --
  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(activeId ? library.getAlbumCoverUrl(activeId) : "");
  
  // -- Dimensions & Layout Logic --
  let innerHeight = $state(0);
  
  const HUD_HEIGHT = 96; 
  const PADDING = 32;
  const MARGIN = 32;

  let availableHeight = $derived(Math.max(0, innerHeight - HUD_HEIGHT));
  let squareModuleSize = $derived(availableHeight);
  let coverSize = $derived(Math.max(0, squareModuleSize - (PADDING * 2)));

  // -- Canvas / Pica Logic --
  let canvasEl;
  let isCanvasReady = $state(false);
  let lastRenderKey = "";

  async function renderCover(url, size) {
    if (!url || !size || !canvasEl || size <= 0) return;

    const renderKey = `${url}-${size}`;
    if (renderKey === lastRenderKey) return;
    lastRenderKey = renderKey;
    isCanvasReady = false;

    try {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.src = url;
      await img.decode();

      const dpr = window.devicePixelRatio || 1;
      canvasEl.width = size * dpr;
      canvasEl.height = size * dpr;

      await pica.resize(img, canvasEl, {
        quality: 3,
        alpha: false,
        unsharpAmount: 0, 
      });
      isCanvasReady = true;
    } catch (err) {
      console.error(err);
      isCanvasReady = false; 
    }
  }

  $effect(() => {
    renderCover(coverUrl, coverSize);
  });
</script>

<svelte:window bind:innerHeight />

<div class="queue-view-container">
  <QueueHud>
    <div class="main-panel-layout" style="margin: 0 {MARGIN}px;">
      
      <div 
        class="module-left" 
        style="
            width: {squareModuleSize}px; 
            height: {squareModuleSize}px;
            padding: {PADDING}px;
        "
      >
        <div class="cover-container" style="width: 100%; height: 100%;">
          {#if coverUrl}
            <img 
              src={coverUrl} 
              class="backing-img" 
              class:visible={!isCanvasReady}
              alt="" 
            />
            <canvas 
              bind:this={canvasEl} 
              class="pica-canvas"
              class:visible={isCanvasReady}
              style="width: 100%; height: 100%;"
            ></canvas>
          {:else}
            <div class="empty-state">
              <span class="empty-text">NO SIGNAL</span>
            </div>
          {/if}
        </div>
      </div>

      <div 
        class="module-right"
        style="
            padding: {PADDING}px;
            height: {availableHeight}px;
        "
      >
        <QueueTracks />
      </div>

    </div>
  </QueueHud>
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    background-color: #1F1F1F;
    position: relative;
    overflow: hidden;
  }

  .main-panel-layout {
    display: flex;
    flex-direction: row;
    align-items: center;
    height: 100%;
    pointer-events: auto;
    box-sizing: border-box;
  }

  .module-left {
    flex-shrink: 0;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover-container {
    position: relative;
    background-color: #000;
    /* Updated Centered Triple-Layer Shadow */
    box-shadow: 
      0 0 24px rgba(0, 0, 0, 0.3), 
      0 0 16px rgba(0, 0, 0, 0.3), 
      0 0 8px rgba(0, 0, 0, 0.3);
    flex-shrink: 0;
  }

  .backing-img, .pica-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .backing-img.visible, .pica-canvas.visible {
    opacity: 1;
  }

  .empty-state {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #1a1a1a;
  }

  .empty-text {
    font-family: var(--font-mono);
    color: #333;
    letter-spacing: 2px;
    font-size: 12px;
  }

  .module-right {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    box-sizing: border-box;
    overflow: hidden; 
  }
</style>
