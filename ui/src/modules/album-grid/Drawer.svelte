<script>
  import { playAlbum } from "../../api.js";
  import { library } from "../../library.svelte.js";
  import DrawerTracks from "./DrawerTracks.svelte";
  import SmartImage from "./SmartImage.svelte";

  let { 
    activeAlbum, 
    activeIndexInRow,
    width, 
    height,       
    bandA,        
    bandB,        
    trackCols,
    bandCHeight,
    cardSize,
    gap,
    drawerCoverSize
  } = $props();

  let tracksContainer = $state(null);
  let hasOverflow = $state(false);
  let isHoveringScrollbar = $state(false);

  let coverUrl = $derived(library.getAlbumCoverUrl(activeAlbum.id));
  let chevronLeft = $derived((activeIndexInRow * (cardSize + gap)) + (cardSize / 2));

  function checkOverflow() {
    if (tracksContainer) {
      hasOverflow = tracksContainer.scrollHeight > tracksContainer.clientHeight;
    }
  }

  $effect(() => {
    if (activeAlbum) checkOverflow();
  });

  async function handlePlay() {
    try {
      await playAlbum(activeAlbum.id);
    } catch (err) {
      console.error("Failed to play album:", err);
    }
  }

  function handleMouseMove(e) {
    if (!hasOverflow) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const dist = rect.right - e.clientX;
    isHoveringScrollbar = dist <= 24;
  }

  function handleMouseLeave() {
    isHoveringScrollbar = false;
  }

  function handleWheel(e) {
    if (!hasOverflow) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const dist = rect.right - e.clientX;
    if (dist <= 24) {
      e.stopPropagation();
    } else {
      e.preventDefault(); 
    }
  }
</script>

<div class="drawer-container" style="width: {width}px; height: {height}px;">
  <div class="pointer-band" style="height: {bandA + bandB}px;">
    <div 
      class="chevron-pointer" 
      style="left: {chevronLeft}px; border-bottom-width: {bandB}px; border-left-width: {bandB}px; border-right-width: {bandB}px; z-index: 10;"
    ></div>
  </div>

  <div 
    class="drawer-content" 
    style="height: {bandCHeight}px;"
  >
      <div class="split-layout">
        <div class="cover-col">
          <SmartImage 
            src={coverUrl} 
            width={drawerCoverSize} 
            height={drawerCoverSize} 
          />
        </div>

        <div class="info-col">
          <div class="header-text">
            <div class="title-row">
              <h2 class="d-title">{activeAlbum.title}</h2>
              <button class="play-button" onclick={handlePlay} title="Replace queue and play album">
                PLAY ALBUM
              </button>
            </div>
            <h3 class="d-artist">{activeAlbum.artist}</h3>
          </div>
          
          <div 
            bind:this={tracksContainer}
            class="tracks-wrapper" 
            class:has-overflow={hasOverflow}
            onwheel={handleWheel}
            onmousemove={handleMouseMove}
            onmouseleave={handleMouseLeave}
            role="region"
            aria-label="Track list"
          >
            {#if hasOverflow}
              <div class="scrollbar-zone" class:active={isHoveringScrollbar}></div>
            {/if}
            <DrawerTracks tracks={activeAlbum.tracks} cols={trackCols} />
          </div>
        </div>
      </div>
  </div>
</div>

<style>
  .drawer-container {
    display: flex;
    flex-direction: column;
    margin: 0 auto;
    box-sizing: border-box;
    overflow: hidden;
    position: relative;
    -webkit-font-smoothing: subpixel-antialiased;
  }

  .pointer-band {
    position: relative;
    width: 100%;
    flex-shrink: 0;
  }

  .chevron-pointer {
    position: absolute;
    bottom: 0;
    width: 0;
    height: 0;
    border-style: solid;
    border-top-color: transparent;
    border-left-color: transparent;
    border-right-color: transparent;
    border-bottom-color: var(--border-muted);
    transform: translateX(-50%);
  }

  .chevron-pointer::after {
    content: '';
    position: absolute;
    top: 2px;
    left: -12px;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 0 12px 12px 12px;
    border-color: transparent transparent var(--background-drawer) transparent;
  }

  .drawer-content {
    box-sizing: border-box;
    padding: var(--drawer-padding-y) var(--drawer-padding-x);
    overflow: hidden;
    background-color: var(--background-drawer);
    border: 1px solid var(--border-muted);
    z-index: 1;
  }

  .split-layout {
    display: flex;
    flex-direction: row;
    height: 100%;
    gap: var(--drawer-split-gap);
    max-width: var(--drawer-contents-x-max);
    margin: 0 auto;
  }

  .cover-col {
    display: flex;
    flex-direction: column;
    width: var(--drawer-cover-size);
    flex-shrink: 0;
  }

  .info-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header-text {
    margin-bottom: 24px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    flex-shrink: 0;
  }

  .title-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .d-title { 
    margin: 0; 
    color: var(--text-main); 
    font-size: var(--drawer-font-size-album); 
    font-weight: 400;
  }

  .play-button {
    background: none;
    border: 1px solid var(--border-muted);
    color: var(--text-muted);
    padding: 4px 12px;
    font-family: var(--font-stack);
    font-size: 11px;
    letter-spacing: 0.1em;
    cursor: pointer;
    transition: all 0.1s ease;
    pointer-events: auto;
  }

  .play-button:hover {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.05);
    border-color: var(--text-muted);
  }
  
  .d-artist { 
    margin: 4px 0 0 0; 
    color: var(--text-muted); 
    font-size: var(--drawer-font-size-artist); 
    font-weight: 400;
  }

  .tracks-wrapper {
    flex: 1;
    min-height: 0;
    overflow-y: auto; 
    pointer-events: auto;
    padding-right: 0;
    position: relative;
  }

  .tracks-wrapper.has-overflow {
    padding-right: 24px;
  }

  .scrollbar-zone {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 24px;
    background-color: transparent;
    pointer-events: none;
    transition: background-color 0.2s;
    z-index: 10;
  }

  .scrollbar-zone.active {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .tracks-wrapper::-webkit-scrollbar {
    width: 6px;
  }

  .tracks-wrapper::-webkit-scrollbar-track {
    background: transparent;
  }

  .tracks-wrapper::-webkit-scrollbar-thumb {
    background-color: var(--palette-300);
    border-radius: 3px;
  }

  .tracks-wrapper::-webkit-scrollbar-thumb:hover {
    background-color: var(--palette-400);
  }
</style>
