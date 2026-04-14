<script>
  import { theme } from "../../../theme.svelte.js";
  import { library } from "../../../library.svelte.js";
  import AlbumText from "./AlbumText.svelte";

  let { album, active, onclick, scrollY = 0, rowY = 0 } = $props();

  let originalUrl = $derived(library.getThumbnailUrl(album));
  let coverBitmap = $derived(library.pinnedTextures.get(originalUrl));

  const coverSize = $derived(theme.albumGrid["cover-size"]);

  let coverCanvas = $state();

  $effect(() => {
    if (coverCanvas && coverBitmap) {
      const ctx = coverCanvas.getContext('2d', { alpha: false });
      const dpr = window.devicePixelRatio || 1;
      coverCanvas.width = coverSize * dpr;
      coverCanvas.height = (coverSize + 2) * dpr;
      
      ctx.scale(dpr, dpr);
      ctx.translate(0, 1);
      
      const bgHex = "#292929";
      ctx.fillStyle = bgHex;
      ctx.fillRect(0, -1, coverSize, coverSize + 2);
      
      ctx.drawImage(coverBitmap, 0, 0, coverSize, coverSize);
    }
  });
</script>

<div class="album-unit">
  <button 
    class="album-cover" 
    class:active
    style="z-index: 10;"
    {onclick}
    aria-label="Select album {album.title}"
  >
    {#if originalUrl}
      <img 
        src={originalUrl} 
        alt="" 
        decoding="sync"
        draggable="false"
        style="opacity: {coverBitmap ? 0 : 1}; position: absolute; inset: 0;"
      />
    {/if}
    <canvas 
        bind:this={coverCanvas}
        style="opacity: {coverBitmap ? 1 : 0}; width: 100%; height: calc(100% + 2px); position: absolute; top: -1px; left: 0; display: block; pointer-events: none;"
    ></canvas>
  </button>
  
  <AlbumText {album} {scrollY} {rowY} />
</div>

<style>
  .album-unit {
    display: flex;
    flex-direction: column;
    flex-shrink: 0; 
    width: var(--cover-size);
    padding-top: var(--gap-y);
    position: relative;
  }

  .album-cover {
    border: none;
    padding: 0;
    cursor: pointer;
    display: block;
    outline: none !important;
    width: var(--cover-size);
    height: var(--cover-size);
    margin-bottom: var(--text-gap-main);
    position: relative;
    background-color: #323232;
    border-radius: 0px;
    box-shadow: var(--album-cover-shadow);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    pointer-events: auto;
    overflow: visible;
  }

  .album-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
