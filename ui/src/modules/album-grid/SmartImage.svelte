<script>
  import { onMount, onDestroy } from "svelte";
  import Pica from "pica";

  let { src, width, height } = $props();

  let canvasEl;
  let isLoaded = $state(false);
  let currentSrc = $state("");
  
  // Singleton Pica instance
  const pica = new Pica();

  async function processImage(url) {
    if (!url || url === currentSrc) return;
    currentSrc = url;
    
    // We keep isLoaded true if it's already showing an image to prevent flickering 
    // unless you want a brief blank state between different albums.
    isLoaded = false;

    try {
      // 1. Parallel Fetch & Decode
      // We fetch the blob and immediately start decoding it into a bitmap
      // This happens off the main thread.
      const response = await fetch(url);
      const blob = await response.blob();
      
      const bitmap = await createImageBitmap(blob, {
        colorSpaceConversion: 'default',
        premultiplyAlpha: 'none'
      });

      // 2. High-DPI Canvas Setup
      const dpr = window.devicePixelRatio || 1;
      const targetWidth = width * dpr;
      const targetHeight = height * dpr;

      if (canvasEl) {
        canvasEl.width = targetWidth;
        canvasEl.height = targetHeight;

        // 3. Pica Resampling
        // Pica can take a ImageBitmap directly, which is faster than an <img> tag
        await pica.resize(bitmap, canvasEl, {
          quality: 3,
          alpha: false,
          features: ['js', 'wasm', 'ww']
        });

        isLoaded = true;
      }

      // 4. Memory Cleanup
      bitmap.close(); 
    } catch (err) {
      console.error("High-speed render failed:", err);
    }
  }

  $effect(() => {
    processImage(src);
  });

  onDestroy(() => {
    currentSrc = "";
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
    overflow: hidden;
    background-color: #121212; /* Neutral dark background for the "pop" */
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }

  .output-canvas {
    position: absolute;
    top: 0;
    left: 0;
    opacity: 0;
    /* Ultra-fast transition for the "pop" effect */
    transition: opacity 0.15s ease-in;
    
    /* Optimize for sharp rendering */
    image-rendering: -webkit-optimize-contrast;
    image-rendering: crisp-edges;
  }

  .output-canvas.visible {
    opacity: 1;
  }
</style>
