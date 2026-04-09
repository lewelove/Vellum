<script>
  import { onDestroy } from "svelte";
  import { pica } from "../pica.js";

  let { src, width, height } = $props();

  let canvasEl;
  let isLoaded = $state(false);
  let currentSrc = $state("");
  let lastRenderedDim = "";

  async function processImage(url, w, h) {
    if (!url || !w || !h) {
      isLoaded = false;
      return;
    }
    
    const dimKey = `${url}-${w}-${h}`;
    if (dimKey === lastRenderedDim) return;
    
    lastRenderedDim = dimKey;
    currentSrc = url;
    isLoaded = false;

    try {
      const response = await fetch(url);
      const blob = await response.blob();
      
      const bitmap = await createImageBitmap(blob, {
        colorSpaceConversion: 'default',
        premultiplyAlpha: 'none'
      });

      const dpr = window.devicePixelRatio || 1;
      const targetWidth = w * dpr;
      const targetHeight = h * dpr;

      if (canvasEl) {
        canvasEl.width = targetWidth;
        canvasEl.height = targetHeight;

        await pica.resize(bitmap, canvasEl, {
          quality: 3,
          alpha: false,
          features: ['js', 'wasm', 'ww']
        });

        isLoaded = true;
      }

      bitmap.close(); 
    } catch (err) {
      console.error("High-speed render failed:", err);
    }
  }

  $effect(() => {
    processImage(src, width, height);
  });

  onDestroy(() => {
    currentSrc = "";
    lastRenderedDim = "";
  });
</script>

<div class="clear-cover-wrapper" style="width: {width}px; height: {height}px;">
  {#if src}
    <div class="cover-block" class:visible={isLoaded}>
      <img
        {src}
        class="cover-image"
        alt=""
      />
      <canvas
        bind:this={canvasEl}
        class="output-canvas"
        style="width: {width}px; height: {height}px;"
      ></canvas>
    </div>
  {:else}
    <div class="empty-cover">
      <span>NO SIGNAL</span>
    </div>
  {/if}
</div>

<style>
  .clear-cover-wrapper {
    position: relative;
    overflow: visible;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover-block {
    position: absolute;
    inset: 0;
    opacity: 0;
    transition: opacity 0.4s;
    will-change: opacity;
  }

  .cover-block.visible {
    opacity: 1;
  }

  .cover-image {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .output-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    background-color: transparent;
  }

  .cover-block.visible .output-canvas {
    box-shadow: var(--album-cover-shadow);
  }

  .empty-cover {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
  }

  .empty-cover span {
    font-family: var(--font-mono);
    color: #444;
    font-size: 12px;
    letter-spacing: 2px;
  }
</style>
