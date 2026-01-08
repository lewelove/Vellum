<script>
  let { 
    activeAlbum, 
    width, 
    cardSize, 
    gap, 
    activeIndexInRow, 
    height, // Forced quantized height
    pointerSize = 12 
  } = $props();

  let pointerOffset = $derived((activeIndexInRow * (cardSize + gap)) + (cardSize / 2));
  let topOffset = $derived(-(pointerSize / 2) - 1); 
</script>

<div class="drawer" style="width: {width}px; height: {height}px; --p-size: {pointerSize}px; --p-top: {topOffset}px;">
  <div class="pointer" style="left: {pointerOffset}px;"></div>

  <div class="drawer-content">
    <div class="header">
      <h2>{activeAlbum.title}</h2>
      <h3>{activeAlbum.artist}</h3>
    </div>
    <ul>
      {#each activeAlbum.tracks as track}
        <li>{track}</li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .drawer {
    position: relative;
    background-color: var(--background-drawer);
    border: 1px solid var(--border-muted);
    box-sizing: border-box; /* Crucial for height math */
    margin: 0 auto; /* Removed bottom margin to keep grid tight */
    overflow: hidden;
  }

  .pointer {
    position: absolute;
    top: var(--p-top); 
    width: var(--p-size);
    height: var(--p-size);
    background-color: var(--background-drawer);
    border-top: 1px solid var(--border-muted);
    border-left: 1px solid var(--border-muted);
    transform: translateX(-50%) rotate(45deg);
    z-index: 1;
  }

  .drawer-content {
    padding: 30px;
  }

  h2 { margin: 0 0 5px 0; color: var(--text-main); font-size: 28px; font-weight: normal; }
  h3 { margin: 0 0 30px 0; color: var(--text-muted); font-size: 16px; font-weight: normal; }
  
  ul { list-style: none; padding: 0; margin: 0; }
  li { 
    padding: 10px 0; 
    border-bottom: 1px solid var(--border-muted); 
    font-size: 14px; 
    color: var(--text-main); 
  }
</style>
