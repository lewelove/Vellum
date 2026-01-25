<script>
  import { player } from "../player.svelte.js";
  import { pica } from "../../pica.js";

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
    if (canvasEl && coverUrl && containerHeight > 0) {
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
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .background-canvas {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    height: 120%; 
    width: auto;
    opacity: 0.3;
    filter: blur(80px);
    z-index: 0;
    pointer-events: none;
  }

  .foreground-content {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 10;
  }

  .now-playing-cover {
    width: 60vh;
    height: 60vh;
    max-width: 80vw;
    object-fit: cover;
    box-shadow: 0 40px 100px rgba(0,0,0,0.7);
    margin-bottom: 40px;
    background-color: #222;
  }

  .track-info {
    text-align: center;
    max-width: 80%;
  }

  .track-title {
    font-size: 32px;
    font-weight: 500;
    color: var(--text-main);
    margin: 0 0 12px 0;
    text-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .track-artist {
    font-size: 22px;
    font-weight: 400;
    color: var(--text-muted);
    margin: 0;
    text-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .empty-state {
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    font-weight: 500;
  }
</style>
