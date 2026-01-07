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

  let contentHeight = $derived.by(() => {
    const baseRows = rows.length;
    const extraRows = drawerInfo ? drawerInfo.rows : 0;
    return (baseRows + extraRows) * layout.rowHeight;
  });

  let totalSlots = $derived(rows.length + (drawerInfo ? drawerInfo.rows : 0));

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
    scroll.update(viewportHeight, contentHeight, layout.rowHeight);
    if (mainEl) {
      mainEl.scrollTop = Math.round(scroll.currentY);
    }
    rafId = requestAnimationFrame(loop);
  }

  async function toggleAlbum(id) {
    const oldId = expandedAlbumId;
    expandedAlbumId = expandedAlbumId === id ? null : id;
    await tick();
    
    if (expandedAlbumId && oldId === null) {
        const rowIndex = rows.findIndex(row => row.find(a => a.id === id));
        if (rowIndex > scroll.targetSlot + 2) scroll.targetSlot = rowIndex;
    }
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
    scroll.handleWheel(e, totalSlots); 
  }}
>
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
        <!-- Horizontal Gap Fixed: Added layout.gap to the gap property -->
        <div 
          class="row-inner" 
          style="gap: {layout.gap}px; padding-top: {layout.gap}px;"
        >
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
        box-sizing: border-box;
    }

    .row-inner {
        display: flex;
        justify-content: flex-start;
        height: 100%;
        box-sizing: border-box;
    }
</style>
