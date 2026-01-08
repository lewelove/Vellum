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
    chevronWidth,
    bandCHeight
  } = $props();

  // Calculate the horizontal center of the active album cover to align the chevron tip
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
        <!-- 1. Fill first: This masks the border-top of band-c that passes underneath -->
        <path 
          d="M0 {bandB + 1} L{chevronWidth / 2} 1 L{chevronWidth} {bandB + 1} L0 {bandB + 1}Z" 
          fill="var(--background-drawer)"
        />
        <!-- 2. Stroke second: Drawn on top of the fill. 
             We use 1.2px to compensate for diagonal anti-aliasing softness. -->
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
    box-sizing: border-box;
  }

  .band-a { 
    background-color: transparent; 
  }

  .band-b {
    position: relative;
    background-color: transparent;
    /* Ensure the chevron sits above the border of band-c */
    z-index: 2;
  }

  .pointer-wrapper {
    position: absolute;
    /* Push down 1px to overlap the border-top of band-c exactly */
    bottom: -1px;
    transform: translateX(-50%);
    display: flex;
    align-items: flex-end;
  }

  .pointer-wrapper svg {
    display: block;
    overflow: visible;
  }

  .band-c {
    position: relative;
    z-index: 1;
    background-color: var(--background-drawer);
    /* Full border containment */
    border: 1px solid var(--border-muted);
    box-sizing: border-box;
  }

  .drawer-content {
    padding: 0 40px 40px 40px;
  }

  h2 { 
    margin: 0 0 5px 0; 
    color: var(--text-main); 
    font-size: 28px; 
    font-weight: normal; 
  }
  
  h3 { 
    margin: 0 0 30px 0; 
    color: var(--text-muted); 
    font-size: 16px; 
    font-weight: normal; 
  }
  
  .tracklist { 
    list-style: none; 
    padding: 0; 
    margin: 0; 
  }

  .tracklist li { 
    padding: 12px 0; 
    border-bottom: 1px solid var(--border-muted); 
    font-size: 14px; 
    color: var(--text-main); 
  }

  .tracklist li:last-child { 
    border-bottom: none; 
  }
</style>
