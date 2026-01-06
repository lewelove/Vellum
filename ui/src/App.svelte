<script>
  import { onMount, tick, onDestroy } from "svelte";
  import { ScrollEngine } from "$lib/engines/scroll.svelte.js";
  import { LayoutManager } from "$lib/engines/layout.svelte.js";
  import { getLibrary } from "$lib/api.js";
  
  import Album from "$lib/components/Album.svelte";
  import Drawer from "$lib/components/Drawer.svelte";
  import Scrollbar from "$lib/components/Scrollbar.svelte";

  let albums = $state([]);
  let expandedAlbumId = $state(null);
  let viewportHeight = $state(0);
  let contentHeight = $state(0);
  
  const scroll = new ScrollEngine();
  const layout = new LayoutManager();

  let mainEl;
  let contentEl;
  
  // We use a plain array to avoid Proxy overhead in the RAF loop
  let rowElements = []; 
  let rafId;

  let rows = $derived(layout.chunk(albums));

  // Fix 1: Sync content height whenever rows or layout change
  $effect(() => {
    if (rows && contentEl) {
      // Use ResizeObserver-like behavior: wait for the browser to finish reflow
      tick().then(() => {
        contentHeight = contentEl.scrollHeight;
      });
    }
  });

  // Fix 2: Prevent targetIndex from pointing to a non-existent row after resize
  $effect(() => {
    if (rows.length > 0 && scroll.targetIndex >= rows.length) {
      scroll.targetIndex = rows.length - 1;
    }
  });

  function loop() {
    // Fix 3: Clean up the elements list on every frame. 
    // This handles rows being removed/added during resize without needing to reset the array.
    const activeRows = rowElements.filter(el => el && document.contains(el));
    
    scroll.update(activeRows, viewportHeight, contentHeight);
    
    if (mainEl) {
      mainEl.scrollTop = Math.round(scroll.currentY);
    }
    rafId = requestAnimationFrame(loop);
  }

  async function toggleAlbum(id) {
    expandedAlbumId = expandedAlbumId === id ? null : id;
    await tick();
    const rowIndex = rows.findIndex(row => row.find(a => a.id === id));
    if (rowIndex !== -1) scroll.targetIndex = rowIndex;
    if (contentEl) contentHeight = contentEl.scrollHeight;
  }

  onMount(async () => {
    loop();
    try {
      albums = await getLibrary();
      await tick();
      if (contentEl) contentHeight = contentEl.scrollHeight;
    } catch (e) { console.error(e); }
  });

  onDestroy(() => cancelAnimationFrame(rafId));
</script>

<main 
  bind:this={mainEl}
  bind:clientWidth={layout.containerWidth} 
  bind:clientHeight={viewportHeight}
  onwheel={(e) => { 
    e.preventDefault(); 
    scroll.handleWheel(e, rows.length); 
  }}
>
  <Scrollbar 
    {viewportHeight} 
    {contentHeight} 
    currentY={scroll.currentY} 
  />

  <div class="scroll-content" bind:this={contentEl}>
    {#each rows as row, i (i)}
      <div 
        class="row" 
        style="width: {layout.gridWidth}px; gap: {layout.gap}px; margin-bottom: {layout.gap}px"
        bind:this={rowElements[i]}
      >
        {#each row as album (album.id)}
          <Album 
            {album} 
            size={layout.cardSize} 
            active={expandedAlbumId === album.id}
            onclick={() => toggleAlbum(album.id)} 
          />
        {/each}
      </div>

      {#if row.find(a => a.id === expandedAlbumId)}
        {@const activeAlbum = row.find(a => a.id === expandedAlbumId)}
        {@const activeIndexInRow = row.findIndex(a => a.id === expandedAlbumId)}
        
        <Drawer 
          {activeAlbum}
          {activeIndexInRow}
          width={layout.gridWidth} 
          cardSize={layout.cardSize}
          gap={layout.gap}
          pointerSize={24}
        />
      {/if}
    {/each}
  </div>
</main>
