<script>
  import { onMount, tick, onDestroy } from "svelte";

  // --- DATA ---
  let albums = [];
  let expandedAlbumId = null;
  
  // --- LAYOUT CONSTANTS ---
  const CARD_SIZE = 200;
  const GAP = 20;

  // --- VIEWPORT STATE ---
  let containerWidth = 0;
  let viewportHeight = 0;
  let contentHeight = 0;

  // --- PHYSICS ENGINE STATE ---
  let currentY = 0;      
  let targetIndex = 0;   
  let targetY = 0;       
  let rafId;             
  let wheelAccumulator = 0; 

  // DOM References
  let contentEl;
  let rowElements = []; 

  // --- SCROLLBAR STATE ---
  // We clamp scrollProgress for visual purposes only (scrollbar shouldn't fly off screen)
  $: scrollProgress = contentHeight > viewportHeight 
      ? Math.min(1, Math.max(0, currentY / (contentHeight - viewportHeight)))
      : 0;
      
  $: thumbHeight = contentHeight > viewportHeight 
      ? Math.max(50, (viewportHeight / contentHeight) * viewportHeight) 
      : 0;
      
  $: thumbY = scrollProgress * (viewportHeight - thumbHeight);


  // --- PHYSICS LOOP ---
  function loop() {
    // 1. Calculate Target Y
    if (rowElements[targetIndex]) {
      // 20 is the top padding
      targetY = rowElements[targetIndex].offsetTop - 20;
    } else {
      targetY = 0;
    }

    // 2. Bottom Limit Logic
    // We intentionally DO NOT clamp targetY to maxScroll here.
    // This ensures the last row can always snap to the top, 
    // even if it creates empty space at the bottom.
    // However, we prevent negative scrolling (top bound).
    if (targetY < 0) targetY = 0;

    // 3. Lerp (Smooth Animation)
    const diff = targetY - currentY;
    
    if (Math.abs(diff) > 0.5) {
      currentY += diff * 0.12; 
    } else {
      currentY = targetY;
    }

    // 4. Apply Transform
    if (contentEl) {
      contentEl.style.transform = `translate3d(0, -${currentY}px, 0)`;
    }

    rafId = requestAnimationFrame(loop);
  }

  // --- INPUT HANDLER ---
  function handleWheel(e) {
    e.preventDefault();
    wheelAccumulator += e.deltaY;
    const THRESHOLD = 40; 

    if (Math.abs(wheelAccumulator) > THRESHOLD) {
      const direction = wheelAccumulator > 0 ? 1 : -1;
      targetIndex += direction;
      
      // Strict Index Bounding
      if (targetIndex < 0) targetIndex = 0;
      if (targetIndex >= rows.length) targetIndex = rows.length - 1;

      wheelAccumulator = 0;
    }
  }

  // --- LIFECYCLE ---
  onMount(async () => {
    loop();
    try {
      const response = await fetch("/api/library");
      if (response.ok) {
        albums = await response.json();
        await tick();
        updateDimensions();
      }
    } catch (error) {
      console.error("Backend connection failed.", error);
    }
  });

  onDestroy(() => {
    if (rafId) cancelAnimationFrame(rafId);
  });

  function updateDimensions() {
    if (contentEl) {
      contentHeight = contentEl.scrollHeight;
    }
  }

  // --- GRID MATH (CENTERED LOGIC) ---
  // 40px subtraction accounts for wrapper padding (20px left + 20px right)
  $: availableWidth = Math.max(0, containerWidth - 40);
  
  $: cols = Math.floor((availableWidth + GAP) / (CARD_SIZE + GAP)) || 1;
  
  // Strict Grid Width: (Cols * Card) + (Gaps)
  // We use this to force the rows to be exactly this wide, centered via margin:auto
  $: gridWidth = (cols * CARD_SIZE) + ((cols - 1) * GAP);
  
  $: rows = chunkArray(albums, cols);
  $: if (rows) { tick().then(updateDimensions); }

  function chunkArray(arr, size) {
    const results = [];
    for (let i = 0; i < arr.length; i += size) {
      results.push(arr.slice(i, i + size));
    }
    return results;
  }

  async function toggleAlbum(id) {
    // 1. Toggle State
    expandedAlbumId = expandedAlbumId === id ? null : id;
    
    // 2. Find the row index of the clicked album
    //    We do this to snap the view to the clicked row
    const rowIndex = rows.findIndex(row => row.find(a => a.id === id));
    if (rowIndex !== -1) {
      targetIndex = rowIndex;
    }

    // 3. Update Layout
    await tick();
    updateDimensions();
  }
</script>

<main 
  bind:clientWidth={containerWidth} 
  bind:clientHeight={viewportHeight}
  on:wheel={handleWheel}
>
  
  <!-- CUSTOM SCROLLBAR -->
  <div class="custom-scrollbar-track">
    <div 
      class="custom-scrollbar-thumb"
      style="height: {thumbHeight}px; transform: translateY({thumbY}px);"
    ></div>
  </div>

  <!-- CONTENT LAYER -->
  <div class="scroll-content" bind:this={contentEl}>
    
    {#each rows as row, i}
      <div 
        class="row" 
        style="width: {gridWidth}px; gap: {GAP}px; margin-bottom: {GAP}px"
        bind:this={rowElements[i]}
      >
        {#each row as album}
          <div class="album-unit" style="width: {CARD_SIZE}px;">
            <button 
              class="album-cover" 
              class:active={expandedAlbumId === album.id}
              style="width: {CARD_SIZE}px; height: {CARD_SIZE}px; background-color: {album.color};"
              on:click={() => toggleAlbum(album.id)}
              aria-label="View details for {album.title}"
            >
            </button>
            <div class="album-info">
              <span class="album-title">{album.title}</span>
              <span class="album-artist">{album.artist}</span>
            </div>
          </div>
        {/each}
      </div>

      {#if row.find(a => a.id === expandedAlbumId)}
        {@const activeAlbum = row.find(a => a.id === expandedAlbumId)}
        <!-- Drawer shares the same fixed gridWidth to stay aligned -->
        <div class="drawer" style="width: {gridWidth}px;">
          <div class="drawer-content">
            <h2>{activeAlbum.title}</h2>
            <h3>{activeAlbum.artist}</h3>
            <ul>
              {#each activeAlbum.tracks as track}
                <li>{track}</li>
              {/each}
            </ul>
          </div>
        </div>
      {/if}

    {/each}
  </div>

</main>
