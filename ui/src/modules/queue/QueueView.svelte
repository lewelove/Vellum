<script>
  import { player } from "../player.svelte.js";
  import { pica } from "../../pica.js";

  // Fix: Explicit state for canvas element reference to satisfy Svelte 5 strictness
  let canvasEl = $state(null);
  let containerHeight = $state(0);
  
  let activeId = $derived(player.currentAlbumId);
  
  let coverUrl = $derived(
    activeId ? `/api/assets/${encodeURIComponent(activeId)}/cover` : ""
  );

  async function renderBackground(url, height) {
    if (!url || !canvasEl || height === 0) return;

    try {
      const response = await fetch(url);
      const blob = await response.blob();
      const bitmap = await createImageBitmap(blob);

      const dpr = window.devicePixelRatio || 1;
      const targetHeight = height * dpr;
      
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
    // Svelte 5 effect tracks dependencies. 
    // canvasEl is now a state, so changes to it (mounting) will trigger this.
    if (canvasEl) {
      renderBackground(coverUrl, containerHeight);
    }
  });
</script>

<div 
  class="queue-view-container" 
  bind:clientHeight={containerHeight}
>
  {#if coverUrl}
    <canvas bind:this={canvasEl} class="background-canvas"></canvas>
    
    <div class="foreground-content">
      <img src={coverUrl} alt="Now Playing" class="now-playing-cover" />
      <div class="track-info">
        {#if player.title}<h1 class="track-title">{player.title}</h1>{/if}
        {#if player.artist}<h2 class="track-artist">{player.artist}</h2>{/if}
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <span>Not Playing</span>
    </div>
  {/if}
</div>

<style>
  .queue-view-container {
    width: 100%;
    height: 100%;
    position: relative;
    background-color: var(--background-main);
    overflow: hidden;
  }

  .background-canvas {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    height: 110%; 
    width: auto;
    opacity: 0.4;
    filter: blur(60px);
    z-index: 0;
  }

  .foreground-content {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 10;
  }

  .now-playing-cover {
    width: 40vh;
    height: 40vh;
    object-fit: cover;
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
    margin-bottom: 32px;
  }

  .track-info {
    text-align: center;
    max-width: 600px;
  }

  .track-title {
    font-size: 24px;
    font-weight: 500;
    color: var(--text-main);
    margin: 0 0 8px 0;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
  }

  .track-artist {
    font-size: 18px;
    font-weight: 400;
    color: var(--text-muted);
    margin: 0;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 14px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
</style>
