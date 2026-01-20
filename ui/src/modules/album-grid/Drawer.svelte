<script>
  import DrawerTracks from "./DrawerTracks.svelte";

  let { 
    activeAlbum, 
    width, 
    gap, 
    height,       
    bandA,        
    bandB,        
    trackCols,
    bandCHeight
  } = $props();

  // Simplified Drawer: Only content is rendered, relying on layout engine for spacing/heights
</script>

<div class="drawer-container" style="width: {width}px; height: {height}px;">
  <!-- Spacer Band to match GridController geometry -->
  <div style="height: {bandA + bandB}px;"></div>

  <!-- Content Area -->
  <div class="drawer-content" style="height: {bandCHeight}px;">
      <header class="drawer-header">
        <div class="header-left">
          <h2 class="d-title">{activeAlbum.title}</h2>
          <h3 class="d-artist">{activeAlbum.artist}</h3>
        </div>
        <div class="header-right">
          <span class="d-info">{activeAlbum.tracks.length} Tracks</span>
        </div>
      </header>
      
      <DrawerTracks tracks={activeAlbum.tracks} cols={trackCols} />
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

  .drawer-content {
    background-color: var(--background-drawer);
    border: 1px solid var(--border-muted);
    box-sizing: border-box;
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
    font-weight: 400;
  }
  
  .d-artist { 
    margin: 4px 0 0 0; 
    color: var(--text-muted); 
    font-size: var(--drawer-font-size-artist); 
    font-weight: 300;
  }

  .d-info {
    font-size: 14px;
    color: var(--text-muted);
    line-height: 1.4;
  }
</style>
