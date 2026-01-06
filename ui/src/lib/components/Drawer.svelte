<script>
  /** 
   * @typedef {Object} Props
   * @property {Object} activeAlbum
   * @property {number} width
   * @property {number} cardSize
   * @property {number} gap
   * @property {number} activeIndexInRow
   * @property {number} [pointerSize=12] - The side length of the square forming the chevron
   */
  let { 
    activeAlbum, 
    width, 
    cardSize, 
    gap, 
    activeIndexInRow, 
    pointerSize = 12 
  } = $props();

  let pointerOffset = $derived((activeIndexInRow * (cardSize + gap)) + (cardSize / 2));
  
  let topOffset = $derived(-(pointerSize / 2) - 1); 
</script>

<div class="drawer" style="width: {width}px; --p-size: {pointerSize}px; --p-top: {topOffset}px;">
  <div 
    class="pointer" 
    style="left: {pointerOffset}px;"
  ></div>

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
    background-color: var(--drawer-bg);
    border: 1px solid var(--grey-300);
    box-sizing: border-box;
    margin: 0 auto 20px auto;
    animation: slideDown 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .pointer {
    position: absolute;
    top: var(--p-top); 
    width: var(--p-size);
    height: var(--p-size);
    
    background-color: var(--drawer-bg);
    border-top: 1px solid var(--grey-300);
    border-left: 1px solid var(--grey-300);
    
    transform: translateX(-50%) rotate(45deg);
    z-index: 1;
  }

  .drawer-content {
    padding: 30px;
    position: relative;
    z-index: 2;
  }

  h2 { margin: 0 0 5px 0; color: var(--highlight); font-size: 28px; font-weight: normal; letter-spacing: -0.02em; }
  h3 { margin: 0 0 30px 0; color: var(--text-accent); font-size: 16px; font-weight: normal; }
  
  ul { list-style: none; padding: 0; margin: 0; }
  li { 
    padding: 12px 0; 
    border-bottom: 1px solid oklch(0.25 0 0); 
    font-size: 14px; 
    color: var(--text-main); 
  }
  li:last-child { border-bottom: none; }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
