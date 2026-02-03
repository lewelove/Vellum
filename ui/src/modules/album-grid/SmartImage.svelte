<script>
  import { onMount, onDestroy } from "svelte";
  import { pica } from "../../pica.js";

  let { src, width, height } = $props();

  let canvasEl;
  let isLoaded = $state(false);
  let currentSrc = $state("");
  let lastRenderedDim = "";

  async function processImage(url, w, h) {
    if (!url || !w || !h) return;
    
    // Check if we actually need to redraw
    const dimKey = `${url}-${w}-${h}`;
    if (dimKey === lastRenderedDim) return;
    
    lastRenderedDim = dimKey;
    currentSrc = url;

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

  // React to URL, width, or height changes
  $effect(() => {
    processImage(src, width, height);
  });

  onDestroy(() => {
    currentSrc = "";
    lastRenderedDim = "";
  });
</script>

<div class="smart-image-wrapper" style="width: {width}px; height: {height}px;">
  <canvas 
    bind:this={canvasEl} 
    class="output-canvas" 
    class:visible={isLoaded}
    style="width: {width}px; height: {height}px;"
  ></canvas>
</div>

<style>
  .smart-image-wrapper {
    position: relative;
    overflow: hidden; /* Ensure no bleed during resize */
  }

  .output-canvas {
    position: absolute;
    top: 0;
    left: 0;
    opacity: 0;
    background-color: var(--background-drawer);
    transition: opacity 0.4s;
  }

  .output-canvas.visible {
    opacity: 1;
  }
</style>
