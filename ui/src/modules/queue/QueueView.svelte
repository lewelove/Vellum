<script>
  import { onMount } from "svelte";
  import { player } from "../player.svelte.js";
  import { library } from "../../library.svelte.js";
  import { nav } from "../../navigation.svelte.js";
  import { pica } from "../../pica.js";
  import QueueTracks from "./QueueTracks.svelte";
  import QueueRowTop from "./QueueRowTop.svelte";
  import QueueRowBottom from "./QueueRowBottom.svelte";

  let activeId = $derived(player.currentAlbumId);
  let coverUrl = $derived(
    activeId ? library.getAlbumCoverUrl(activeId) : ""
  );

  let innerWidth = $state(0);
  let innerHeight = $state(0);
  let canvasEl = $state(null);

  let boxSize = $derived(innerHeight);
  let boxX = $derived(Math.floor((innerWidth - boxSize) / 2));
  
  let sidebarWidth = $derived(Math.max(0, (innerWidth - boxSize) / 2));
  let isQueueVisible = $derived(nav.activeTab === "queue");

  async function renderCover(url, size) {
    if (!url || !size || !canvasEl || size <= 0) return;

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
    <div class="row-top-wrapper">
      <QueueRowTop />
    </div>

    {#if sidebarWidth > 0}
      <div 
        class="tracks-overlay" 
        style="
          width: {sidebarWidth}px; 
          visibility: {isQueueVisible ? 'visible' : 'hidden'};
          pointer-events: {isQueueVisible ? 'auto' : 'none'};
        "
      >
        <QueueTracks />
      </div>
    {/if}

    <div class="row-bottom-wrapper">
      <QueueRowBottom />
    </div>
  </div>

  <div class="queue-stage">
    {#if coverUrl && boxSize > 0}
      <div 
        class="pixel-stage" 
        style="
          width: {boxSize}px; 
          height: {boxSize}px; 
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
      <div class="empty-state">
        <span>Not Playing</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .queue-layout {
    width: 100%;
    height: 100%;
    position: relative;
    background-color: var(--background-drawer);
    overflow: hidden;
    contain: paint;
  }

  .ui-layer {
    position: absolute;
    inset: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    pointer-events: none;
  }

  .row-top-wrapper, .row-bottom-wrapper {
    pointer-events: auto;
  }

  .tracks-overlay {
    flex: 1;
    align-self: flex-end;
    pointer-events: auto;
    background-color: transparent;
  }

  .queue-stage {
    position: absolute;
    inset: 0;
    z-index: 20;
    pointer-events: none;
  }

  .pixel-stage {
    position: absolute;
    top: 0;
    bottom: 0;
    overflow: visible;
  }

  .hard-shadow {
    position: absolute;
    inset: 0;
    z-index: 1;
    filter: url(#dithered-shadow);
  }

  .raw-canvas {
    position: relative;
    z-index: 2;
    display: block;
  }

  .empty-state {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 18px;
    letter-spacing: 0.3em;
    text-transform: uppercase;
    pointer-events: none;
  }
</style>
