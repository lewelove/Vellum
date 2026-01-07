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
  
  const scroll = new ScrollEngine();
  const layout = new LayoutManager();

  let mainEl;
  let rafId;

  let rows = $derived(layout.chunk(albums));
  
  let drawerInfo = $derived.by(() => {
    if (!expandedAlbumId) return null;
    const album = albums.find(a => a.id === expandedAlbumId);
    if (!album) return null;
    return layout.getQuantizedDrawer(album.tracks.length);
  });

  // Total Virtual Rows in the system
  let totalRowsCount = $derived(rows.length + (drawerInfo ? drawerInfo.rows : 0));
  
  // The height for the Scrollbar logic
  let contentHeight = $derived(totalRowsCount * layout.rowHeight);

  let visibleRows = $derived(Math.ceil(viewportHeight / layout.rowHeight));

  let maxSlots = $derived(Math.max(0, (totalRowsCount + 1 - visibleRows)));

  // Handle Resize / Re-anchoring
  let prevCols = 0;
  $effect(() => {
    if (layout.cols !== prevCols && prevCols !== 0) {
        const topAlbumIdx = scroll.targetSlot * prevCols;
        const newSlot = Math.floor(topAlbumIdx / layout.cols);
        scroll.syncToSlot(newSlot);
    }
    prevCols = layout.cols;
  });

  function loop() {
    // Engine only needs rowHeight to calculate pixel target from Slot Index
    scroll.update(layout.rowHeight);
    if (mainEl) {
      mainEl.scrollTop = Math.round(scroll.currentY);
    }
    rafId = requestAnimationFrame(loop);
  }

  async function toggleAlbum(id) {
    expandedAlbumId = expandedAlbumId === id ? null : id;
    await tick();
  }

  onMount(async () => {
    loop();
    try {
      albums = await getLibrary();
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
    scroll.handleWheel(e, maxSlots); 
  }}
>
  <!-- Scrollbar uses contentHeight + Hero Padding row for thumb calculation -->
  <Scrollbar 
    {viewportHeight} 
    contentHeight={contentHeight + layout.rowHeight} 
    currentY={scroll.currentY} 
  />

  <div 
    class="scroll-content" 
    style="padding-bottom: {layout.rowHeight}px;"
  >
    {#each rows as row, i (i)}
      <div 
        class="row" 
        style="width: {layout.gridWidth}px; height: {layout.rowHeight}px;"
      >
        <div class="row-inner" style="gap: {layout.gap}px; padding-top: {layout.gap}px;">
            {#each row as album (album.id)}
              <Album 
                {album} 
                size={layout.cardSize} 
                textHeight={layout.textHeight}
                active={expandedAlbumId === album.id}
                onclick={() => toggleAlbum(album.id)} 
              />
            {/each}
        </div>
      </div>

      {#if drawerInfo && row.find(a => a.id === expandedAlbumId)}
        <Drawer 
          activeAlbum={row.find(a => a.id === expandedAlbumId)}
          activeIndexInRow={row.findIndex(a => a.id === expandedAlbumId)}
          width={layout.gridWidth} 
          cardSize={layout.cardSize}
          gap={layout.gap}
          height={drawerInfo.height}
          pointerSize={24}
        />
      {/if}
    {/each}
  </div>
</main>

<style>
    .row {
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }
    .row-inner {
        display: flex;
        justify-content: flex-start;
        height: 100%;
        box-sizing: border-box;
    }
</style>
