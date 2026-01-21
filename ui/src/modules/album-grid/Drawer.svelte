<script>
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
            <h2 class="d-title">{activeAlbum.title}</h2>
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

  .d-title { 
    margin: 0; 
    color: var(--text-main); 
    font-size: var(--drawer-font-size-album); 
    line-height: 1.1;
    font-weight: 600;
    letter-spacing: -0.01em;
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
