<script>
  import { onMount } from "svelte";
  import { library } from "../../library.svelte.js";
  import { pica } from "../../pica.js";

  let canvasEl;
  let containerHeight = $state(0);
  let containerWidth = $state(0);

  // We find the active or first available album to provide the cover for the background
  let activeAlbum = $derived(
    library.albums.find(a => a.id === library.expandedAlbumId) || library.albums[0]
  );

  let coverUrl = $derived(
    activeAlbum ? `/api/assets/${encodeURIComponent(activeAlbum.id)}/cover` : ""
  );

  async function renderBackground(url, height) {
    if (!url || !canvasEl || height === 0) return;

    try {
      const response = await fetch(url);
      const blob = await response.blob();
      const bitmap = await createImageBitmap(blob);

      const dpr = window.devicePixelRatio || 1;
      const targetHeight = height * dpr;
      
      // Calculate width maintaining aspect ratio
      const aspect = bitmap.width / bitmap.height;
      const targetWidth = targetHeight * aspect;

      canvasEl.height = targetHeight;
      canvasEl.width = targetWidth;

      await pica.resize(bitmap, canvasEl, {
        quality: 3,
        alpha: false
      });

      bitmap.close();
    } catch (err) {
      console.error("Queue background render failed:", err);
    }
  }

  $effect(() => {
    renderBackground(coverUrl, containerHeight);
  });
</script>

<div 
  class="queue-view-container" 
  bind:clientHeight={containerHeight}
  bind:clientWidth={containerWidth}
>
  {#if coverUrl}
    <canvas bind:this={canvasEl} class="background-canvas"></canvas>
  {/if}
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--background-main);
    overflow: hidden;
  }

  .background-canvas {
    height: 100%;
    width: auto;
    object-fit: contain;
    box-shadow: 0 0 100px rgba(0,0,0,0.5);
  }
</style>
