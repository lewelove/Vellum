<script>
  import DrawerTracks from "./DrawerTracks.svelte";

  let { 
    activeAlbum, 
    width, 
    height,       
    bandA,        
    bandB,        
    trackCols,
    bandCHeight
  } = $props();

  // High-Res Asset URL - Uses cover_path resolution via server API
  let coverUrl = $derived(`/api/assets/${encodeURIComponent(activeAlbum.id)}/cover`);
</script>

<div class="drawer-container" style="width: {width}px; height: {height}px;">
  <!-- Spacer Band (Alignment) -->
  <div style="height: {bandA + bandB}px;"></div>

  <!-- Content Area -->
  <div class="drawer-content" style="height: {bandCHeight}px;">
      
      <div class="split-layout">
        <!-- LEFT: Cover -->
        <div class="cover-col">
          <img 
            src={coverUrl} 
            alt="Album Cover" 
            class="d-cover"
            loading="lazy" 
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

  /* --- LEFT COLUMN --- */
  .cover-col {
    display: flex;
    flex-direction: column;
    width: var(--drawer-cover-size);
    flex-shrink: 0;
  }

  .d-cover {
    width: var(--drawer-cover-size);
    height: var(--drawer-cover-size);
    object-fit: cover;
    box-shadow: 0 4px 20px rgba(0,0,0,0.5);
    background-color: rgba(255,255,255,0.05);
    animation: fadeIn 0.4s ease-out;
  }

  /* --- RIGHT COLUMN --- */
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
    justify-content: flex-end; /* Align text to bottom of header block if desired, or top */
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
    /* No internal scroll usually needed if layout engine works, but safety fallback */
    overflow-y: hidden; 
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: scale(0.98); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
