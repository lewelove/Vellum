<script>
  import { playAlbum } from "$core/api.js";
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
    gap
  } = $props();

  // High-Res Asset URL
  let coverUrl = $derived(`/api/assets/${encodeURIComponent(activeAlbum.id)}/cover`);

  // Calculate horizontal center of the active album relative to the drawer's width
  let chevronLeft = $derived((activeIndexInRow * (cardSize + gap)) + (cardSize / 2));

  async function handlePlay() {
    try {
      await playAlbum(activeAlbum.id);
    } catch (err) {
      console.error("Failed to play album:", err);
    }
  }
</script>

<div class="drawer-container" style="width: {width}px; height: {height}px;">
  <!-- Spacer Band (Alignment + Pointer) -->
  <div class="pointer-band" style="height: {bandA + bandB}px;">
    <div 
      class="chevron-pointer" 
      style="left: {chevronLeft}px; border-bottom-width: {bandB}px; border-left-width: {bandB}px; border-right-width: {bandB}px;"
    ></div>
  </div>

  <!-- Content Area -->
  <div class="drawer-content" style="height: {bandCHeight}px;">
      
      <div class="split-layout">
        <!-- LEFT: High-Fidelity Cover Rendering -->
        <div class="cover-col">
          <SmartImage 
            src={coverUrl} 
            width={464} 
            height={464} 
          />
        </div>

        <!-- RIGHT: Header + Tracks -->
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
          
          <div class="tracks-wrapper">
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
  }

  .pointer-band {
    position: relative;
    width: 100%;
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
    z-index: 2;
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
    background-color: var(--background-drawer);
    border: 1px solid var(--border-muted);
    box-sizing: border-box;
    padding: var(--drawer-padding-y) var(--drawer-padding-x);
    overflow: hidden;
  }

  .split-layout {
    display: flex;
    flex-direction: row;
    height: 100%;
    gap: var(--drawer-split-gap);
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
  }

  .play-button:hover {
    color: var(--text-main);
    background-color: rgba(255, 255, 255, 0.05);
    border-color: var(--text-muted);
  }
  
  .d-artist { 
    margin: 8px 0 0 0; 
    color: var(--text-muted); 
    font-size: var(--drawer-font-size-artist); 
    font-weight: 400;
  }

  .tracks-wrapper {
    flex: 1;
    overflow-y: hidden; 
  }
</style>
