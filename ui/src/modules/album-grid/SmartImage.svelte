<script>
  import { onMount, onDestroy } from "svelte";
  import Pica from "pica";

  let { src, width, height } = $props();

  let canvasEl;
  let isLoaded = $state(false);
  let currentSrc = $state("");
  
  const pica = new Pica();

  async function processImage(url) {
    if (!url || url === currentSrc) return;
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
      const targetWidth = width * dpr;
      const targetHeight = height * dpr;

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
  }

  .output-canvas {
    position: absolute;
    top: 0;
    left: 0;
    opacity: 0;
    background-color: #292929;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
    transition: opacity 0.05s ease-in;
    
    /* image-rendering: -webkit-optimize-contrast; */
    /* image-rendering: crisp-edges; */
  }

  .output-canvas.visible {
    opacity: 1;
  }
</style>
