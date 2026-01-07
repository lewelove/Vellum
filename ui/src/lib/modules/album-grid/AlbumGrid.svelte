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
  <Scrollbar 
    viewportHeight={ctrl.viewportHeight} 
    contentHeight={ctrl.contentHeight + ctrl.layout.rowHeight} 
    currentY={ctrl.scroll.currentY} 
  />

  <div class="scroll-content" style="padding-bottom: {ctrl.layout.rowHeight}px;">
    {#each ctrl.rows as row, i (i)}
      <div 
        class="row" 
        style="width: {ctrl.layout.gridWidth}px; height: {ctrl.layout.rowHeight}px;"
      >
        <div class="row-inner" style="gap: {ctrl.layout.gap}px; padding-top: {ctrl.layout.gap}px;">
            {#each row as album (album.id)}
              <Album 
                {album} 
                size={ctrl.layout.cardSize} 
                textHeight={ctrl.layout.textHeight}
                active={library.expandedAlbumId === album.id}
                onclick={() => library.toggleExpand(album.id)} 
              />
            {/each}
        </div>
      </div>

      {#if ctrl.drawerInfo && row.find(a => a.id === library.expandedAlbumId)}
        <Drawer 
          activeAlbum={row.find(a => a.id === library.expandedAlbumId)}
          activeIndexInRow={row.findIndex(a => a.id === library.expandedAlbumId)}
          width={ctrl.layout.gridWidth} 
          cardSize={ctrl.layout.cardSize}
          gap={ctrl.layout.gap}
          height={ctrl.drawerInfo.height}
          pointerSize={24}
        />
      {/if}
    {/each}
  </div>
</div>

<style>
    .grid-container {
      width: 100%;
      height: 100%;
      position: relative;
      overflow: hidden;
      overscroll-behavior: none;
    }
    .scroll-content {
      width: 100%;
      padding: 0;
      display: flex;
      flex-direction: column;
    }
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
