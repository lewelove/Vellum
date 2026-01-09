<script>
  import DrawerTracks from "./DrawerTracks.svelte";

  let { 
    activeAlbum, 
    width, 
    cardSize, 
    gap, 
    activeIndexInRow, 
    height,       
    bandA,        
    bandB,        
    trackCols,
    chevronWidth,
    bandCHeight
  } = $props();

  let pointerOffset = $derived((activeIndexInRow * (cardSize + gap)) + (cardSize / 2));
</script>

<div class="drawer-container" style="width: {width}px; height: {height}px;">
  <!-- Band A: Top Spacer -->
  <div class="band-a" style="height: {bandA}px;"></div>

  <!-- Band B: The Chevron Bridge -->
  <div class="band-b" style="height: {bandB}px;">
    <div class="pointer-wrapper" style="left: {pointerOffset}px; width: {chevronWidth}px; height: {bandB}px;">
      <svg 
        width={chevronWidth} 
        height={bandB + 1} 
        viewBox="0 0 {chevronWidth} {bandB + 1}" 
        fill="none" 
        xmlns="http://www.w3.org/2000/svg"
      >
        <path 
          d="M0 {bandB + 1} L{chevronWidth / 2} 1 L{chevronWidth} {bandB + 1} L0 {bandB + 1}Z" 
          fill="var(--background-drawer)"
        />
        <path 
          d="M0 {bandB + 1} L{chevronWidth / 2} 1 L{chevronWidth} {bandB + 1}" 
          stroke="var(--border-muted)" 
          stroke-width="1.2"
        />
      </svg>
    </div>
  </div>

  <!-- Band C: Content Area -->
  <div class="band-c" style="height: {bandCHeight}px;">
    <div class="drawer-content">
      <header class="drawer-header">
        <div class="header-left">
          <h2 class="d-title">{activeAlbum.title}</h2>
          <h3 class="d-artist">{activeAlbum.artist}</h3>
        </div>
        <div class="header-right">
          <span class="d-info">45:12</span>
          <span class="d-genre">Electronic / Downtempo</span>
        </div>
      </header>
      
      <DrawerTracks tracks={activeAlbum.tracks} cols={trackCols} />
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
  }

  .band-a { background-color: transparent; }

  .band-b {
    position: relative;
    z-index: 2;
  }

  .pointer-wrapper {
    position: absolute;
    bottom: -1px;
    transform: translateX(-50%);
    display: flex;
    align-items: flex-end;
  }

  .band-c {
    position: relative;
    z-index: 1;
    background-color: var(--background-drawer);
    border: 1px solid var(--border-muted);
    box-sizing: border-box;
  }

  .drawer-content {
    padding: var(--drawer-padding-y) var(--drawer-padding-x);
  }

  .drawer-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 24px;
    border-bottom: 1px solid var(--border-muted);
    padding-bottom: 8px;
  }

  .header-left {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header-right {
    display: flex;
    flex-direction: column;
    text-align: right;
    flex-shrink: 0;
  }

  .d-title { 
    margin: 0; 
    color: var(--text-main); 
    font-size: var(--drawer-font-size-album); 
    line-height: 1.2;
    font-weight: 500;
    font-style: italic;
  }
  
  .d-artist { 
    margin: 4px 0 0 0; 
    color: var(--text-muted); 
    font-size: var(--drawer-font-size-artist); 
    font-weight: 400;
  }

  .d-info, .d-genre {
    font-size: 14px;
    color: var(--text-muted);
    line-height: 1.4;
  }
</style>
