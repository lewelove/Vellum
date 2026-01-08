<script>
  let { 
    activeAlbum, 
    width, 
    cardSize, 
    gap, 
    activeIndexInRow, 
    height,       
    bandA,        
    bandB,        
    bandCHeight
  } = $props();

  // Calculate the side of the square needed to reach the height of bandB when rotated 45 deg
  // Side = height * sqrt(2)
  let pointerSize = $derived(bandB * Math.SQRT2);
  let pointerOffset = $derived((activeIndexInRow * (cardSize + gap)) + (cardSize / 2));
</script>

<div class="drawer-container" style="width: {width}px; height: {height}px;">
  <!-- Band A: Transparent Gap (12px) -->
  <div class="band-a" style="height: {bandA}px;"></div>

  <!-- Band B: The Transparent Bridge (Chevron Height) -->
  <div class="band-b" style="height: {bandB}px;">
    <div 
        class="pointer" 
        style="left: {pointerOffset}px; --p-size: {pointerSize}px;"
    ></div>
  </div>

  <!-- Band C: The Content Area (Background starts here) -->
  <div class="band-c" style="height: {bandCHeight}px;">
    <div class="drawer-content">
      <div class="header">
        <h2>{activeAlbum.title}</h2>
        <h3>{activeAlbum.artist}</h3>
      </div>
      <ul class="tracklist">
        {#each activeAlbum.tracks as track}
          <li>{track}</li>
        {/each}
      </ul>
    </div>
  </div>
</div>

<style>
  .drawer-container {
    display: flex;
    flex-direction: column;
    margin: 0 auto;
  }

  .band-a {
    background-color: transparent;
  }

  .band-b {
    position: relative;
    background-color: transparent; /* Now transparent to allow bridging */
  }

  .pointer {
    position: absolute;
    /* Anchor to the bottom of Band B / Top of Band C */
    top: 100%; 
    width: var(--p-size);
    height: var(--p-size);
    background-color: var(--background-drawer);
    border-top: 1px solid var(--border-muted);
    border-left: 1px solid var(--border-muted);
    /* Center it and rotate. translate(-50%, -50%) puts the tip exactly bandB pixels up */
    transform: translate(-50%, -50%) rotate(45deg);
    z-index: 2;
  }

  .band-c {
    background-color: var(--background-drawer);
    border-top: 1px solid var(--border-muted); /* Border starts here */
    border-bottom: 1px solid var(--border-muted);
  }

  .drawer-content {
    padding: 0 40px 40px 40px;
  }

  h2 { margin: 0 0 5px 0; color: var(--text-main); font-size: 28px; font-weight: normal; }
  h3 { margin: 0 0 30px 0; color: var(--text-muted); font-size: 16px; font-weight: normal; }
  
  .tracklist { list-style: none; padding: 0; margin: 0; }
  .tracklist li { 
    padding: 12px 0; 
    border-bottom: 1px solid var(--border-muted); 
    font-size: 14px; 
    color: var(--text-main); 
  }
  .tracklist li:last-child { border-bottom: none; }
</style>
