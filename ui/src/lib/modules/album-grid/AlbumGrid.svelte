<script>
  import { onMount, onDestroy } from "svelte";
  import { library } from "$lib/state/library.svelte.js";
  import { GridController } from "./GridController.svelte.js";
  
  import Album from "./components/Album.svelte";
  import Drawer from "./components/Drawer.svelte";
  import Scrollbar from "./components/Scrollbar.svelte";

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
      const topAlbumIdx = ctrl.scroll.targetSlot * prevCols;
      const newSlot = Math.floor(topAlbumIdx / ctrl.layout.cols);
      ctrl.scroll.syncToSlot(newSlot);
    }
    prevCols = ctrl.layout.cols;
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
      The Crease: 
      1. Sits at the top of the scroll container flow, providing the initial 8px shift.
      2. Sticks to the top during scroll to hide peeking text from the previous row.
    -->
    <div
      class="top-crease"
      style="height: {ctrl.layout.creaseHeight}px; margin-bottom: -{ctrl.layout.creaseHeight}px;"
    ></div>

    <Scrollbar 
      viewportHeight={ctrl.viewportHeight} 
      contentHeight={ctrl.contentHeight + ctrl.layout.rowHeight} 
      currentY={ctrl.scroll.currentY} 
    />

    <div 
      class="scroll-content" 
      style="padding-top: {ctrl.layout.topOffset}px; padding-bottom: {ctrl.layout.rowHeight}px;"
    >
      {#each ctrl.rows as row, i (i)}
        <div 
          class="row" 
          style="width: {ctrl.layout.gridWidth}px; height: {ctrl.layout.rowHeight}px;"
        >
          <div class="row-inner" style="gap: var(--gap-x);">
              {#each row as album (album.id)}
                <Album 
                  {album} 
                  active={library.expandedAlbumId === album.id}
                  onclick={() => library.toggleExpand(album.id)} 
                />
              {/each}
          </div>
        </div>

        {#if ctrl.drawerInfo && row.find(a => a.id === library.expandedAlbumId)}
          <div class="drawer-plane">
            <Drawer 
              activeAlbum={row.find(a => a.id === library.expandedAlbumId)}
              activeIndexInRow={row.findIndex(a => a.id === library.expandedAlbumId)}
              width={ctrl.layout.gridWidth} 
              cardSize={ctrl.layout.cardSize}
              gap={ctrl.layout.gapX}
              height={ctrl.drawerInfo.height}
              pointerSize={24}
            />
          </div>
        {/if}
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
      z-index: 1;
      pointer-events: none;
    }

    .scroll-content {
      width: 100%;
      display: flex;
      flex-direction: column;
    }
    
    .row {
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        overflow: visible; 
        position: relative;
    }
    
    .row-inner {
        display: flex;
        justify-content: flex-start;
        height: 100%;
    }

    .drawer-plane {
      position: relative;
      z-index: 0;
    }
</style>
