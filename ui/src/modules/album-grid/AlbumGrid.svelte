<script>
  import { onMount, onDestroy } from "svelte";
  import { library } from "$state/library.svelte.js";
  import { GridController } from "./GridController.svelte.js";
  
  import Album from "./Album.svelte";
  import Drawer from "./Drawer.svelte";
  import Scrollbar from "./Scrollbar.svelte";

  const ctrl = new GridController();
  let mainEl;
  let rafId;

  function loop() {
    ctrl.update(mainEl);
    rafId = requestAnimationFrame(loop);
  }

  let prevCols = 0;
  $effect(() => {
    if (ctrl.layout.cols !== prevCols && prevCols !== 0) {
      // Recalculate slot when resizing columns to maintain relative position
      const topAlbumIdx = ctrl.scroll.targetSlot * prevCols;
      const newSlot = Math.floor(topAlbumIdx / ctrl.layout.cols);
      ctrl.scroll.syncToSlot(newSlot);
    }
    prevCols = ctrl.layout.cols;
  });

  // Watch for Sidebar "view resets" (Filters, Sorts)
  $effect(() => {
    // Create dependency on viewVersion
    const _v = library.viewVersion;
    // Reset scroll to top
    ctrl.resetScroll();
  });

  onMount(() => {
    loop();
  });

  onDestroy(() => {
    if (rafId) cancelAnimationFrame(rafId);
  });
</script>

<div class="album-grid-viewport">
  <div 
    class="grid-container"
    bind:this={mainEl}
    bind:clientWidth={ctrl.layout.containerWidth} 
    bind:clientHeight={ctrl.viewportHeight}
    onwheel={(e) => { 
      e.preventDefault(); 
      ctrl.handleWheel(e); 
    }}
  >
    <!-- 
      CREASE RESTORATION:
      1. margin-bottom negative to allow content to flow "under" it physically.
      2. z-index: 1 to act as the baseline layer (Above text, Below covers).
    -->
    <div
      class="top-crease"
      style="height: {ctrl.layout.creaseHeight}px; margin-bottom: -{ctrl.layout.creaseHeight}px;"
    ></div>

    <Scrollbar 
      viewportHeight={ctrl.viewportHeight} 
      contentHeight={ctrl.contentHeight} 
      currentY={ctrl.scroll.currentY} 
    />

    <!-- Phantom container for scroll height -->
    <div 
      class="scroll-content" 
      style="height: {ctrl.contentHeight}px;"
    >
      <!-- Absolute positioned virtual rows -->
      {#each ctrl.virtualRows as row (row.index)}
        <!-- 
          ROW Z-INDEX LOGIC (REFACTORED for INTERWEAVING):
          We switch from `transform` to `top` to prevent the creation of a Stacking Context.
          This allows the children (Text z0, Covers z2) to interleave with the Crease (z1).
          z-index is strictly 'auto' to ensure the row container remains transparent to the stack.
        -->
        <div 
          class="row" 
          style="
            top: {row.y}px; 
            left: 0;
            width: {ctrl.layout.gridWidth}px; 
            height: {ctrl.layout.rowHeight}px; 
            z-index: auto;
          "
        >
          <div class="row-inner" style="gap: var(--gap-x);">
              {#each row.data as album (album.id)}
                <Album 
                  {album} 
                  active={library.expandedAlbumId === album.id}
                  onclick={() => library.toggleExpand(album.id)} 
                />
              {/each}
          </div>

          {#if row.isExpandedRow && ctrl.drawerInfo && row.data.find(a => a.id === library.expandedAlbumId)}
            <div 
              class="drawer-plane"
              style="top: {ctrl.layout.rowHeight}px;"
            >
              {#key library.expandedAlbumId}
                <Drawer 
                  activeAlbum={row.data.find(a => a.id === library.expandedAlbumId)}
                  activeIndexInRow={row.data.findIndex(a => a.id === library.expandedAlbumId)}
                  width={ctrl.layout.gridWidth} 
                  cardSize={ctrl.layout.cardSize}
                  gap={ctrl.layout.gapX}
                  height={ctrl.drawerInfo.height}
                  bandA={ctrl.drawerInfo.bandA}
                  bandB={ctrl.drawerInfo.bandB}
                  trackCols={ctrl.drawerInfo.trackCols}
                  chevronWidth={ctrl.drawerInfo.chevronWidth}
                  bandCHeight={ctrl.drawerInfo.bandCHeight}
                  drawerCoverSize={ctrl.drawerInfo.drawerCoverSize}
                />
              {/key}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
    .album-grid-viewport {
      position: relative;
      width: 100%;
      height: 100%;
      overflow: hidden;
    }

    .grid-container {
      width: 100%;
      height: 100%;
      position: relative;
      overflow: hidden;
      overscroll-behavior: none;
    }

    .top-crease {
      position: sticky;
      top: 0;
      left: 0;
      width: 100%;
      height: var(--crease-height);
      background-color: var(--background-main);
      z-index: 1; /* Low index: Below Covers (z2), Above Text (z0) */
      pointer-events: none; /* Let clicks pass through to Covers */
    }

    .scroll-content {
      width: 100%;
      position: relative;
    }
    
    .row {
        position: absolute;
        margin: 0 auto;
        right: 0;
        left: 0;
        display: flex;
        flex-direction: column;
        overflow: visible; 
    }
    
    .row-inner {
        display: flex;
        justify-content: flex-start;
        height: 100%;
    }

    .drawer-plane {
      position: absolute;
      left: 0;
      width: 100%;
      z-index: 5;
    }
</style>
