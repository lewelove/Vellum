<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { pica } from "../../pica.js";
  import QueueTracks from "./QueueTracks.svelte";
  import QueueRowTop from "./QueueRowTop.svelte";
  import QueueRowBottom from "./QueueRowBottom.svelte";

  const PADDING = 0;
  const ROW_HEIGHT = 36;

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? library.getAlbumCoverUrl(activeId) : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);
  let canvasEl = $state(null);

  let boxSize = $derived(Math.max(0, Math.min(innerWidth - (PADDING * 2), innerHeight - (PADDING * 2))));
  let boxX = $derived(Math.floor((innerWidth - boxSize) / 2));
  let boxY = $derived(Math.floor((innerHeight - boxSize) / 2));

  let sidebarWidth = $derived(Math.max(0, (innerWidth - boxSize) / 2));

  async function renderCover(url, size) {
    if (!url || !size || !canvasEl) return;

    try {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.src = url;
      
      await img.decode();

      canvasEl.width = size;
      canvasEl.height = size;

      await pica.resize(img, canvasEl, {
        quality: 3,
        alpha: false,
        unsharpAmount: 0,
        features: ['js', 'wasm', 'ww']
      });

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

<div class="queue-layout">
  
  <div class="ui-layer">
    <QueueRowTop />
    <div class="spacer"></div>
    <QueueRowBottom />
  </div>

  {#if sidebarWidth > 0}
    <div 
      class="tracks-layer" 
      style="
        width: {sidebarWidth}px; 
        top: {ROW_HEIGHT}px; 
        bottom: {ROW_HEIGHT}px;
      "
    >
      <QueueTracks />
    </div>
  {/if}

  {#if coverUrl && boxSize > 0}
    <div 
      class="cover-layer" 
      style="
        width: {boxSize}px; 
        height: {boxSize}px; 
        top: {boxY}px;
        left: {boxX}px;
      "
    >
      <div class="hard-shadow" aria-hidden="true">
        <img src={coverUrl} alt="" style="width: 100%; height: 100%;" />
      </div>

      <canvas 
        bind:this={canvasEl}
        class="raw-canvas"
        style="width: {boxSize}px; height: {boxSize}px;"
      ></canvas>
    </div>
  {:else if !coverUrl}
    <div class="empty-layer">
      <span>Not Playing</span>
    </div>
  {/if}

</div>

<style>
  .queue-layout {
    width: 100%;
    height: 100%;
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: 1fr;
    background-color: var(--background-drawer);
    overflow: hidden;
  }

  .ui-layer {
    grid-area: 1 / 1;
    display: flex;
    flex-direction: column;
    z-index: 1;
  }

  .spacer {
    flex: 1;
  }

  .tracks-layer {
    grid-area: 1 / 1;
    justify-self: end;
    position: absolute;
    right: 0;
    z-index: 2;
    overflow: hidden;
    pointer-events: auto;
  }

  .cover-layer {
    grid-area: 1 / 1;
    position: absolute;
    z-index: 3;
    pointer-events: none;
  }

  .empty-layer {
    grid-area: 1 / 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    z-index: 1;
  }

  .hard-shadow {
    position: absolute;
    opacity: 0.5;
    inset: 0;
    z-index: 1;
    filter: url(#dithered-shadow);
  }

  .raw-canvas {
    position: relative;
    z-index: 2;
    display: block;
  }
</style>
