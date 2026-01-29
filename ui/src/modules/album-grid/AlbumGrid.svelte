<script>
  import { onMount, onDestroy } from "svelte";
  import { library } from "../../library.svelte.js";
  import { GridController } from "./GridController.svelte.js";
  
  import Album from "./Album.svelte";
  import Drawer from "./Drawer.svelte";

  const ctrl = new GridController();
  let rafId;

  const activeKeys = new Set();
  const SCROLL_SPEED = 0.20;

  function loop() {
    let delta = 0;
    
    if (activeKeys.has('j') || activeKeys.has('arrowdown')) delta += SCROLL_SPEED;
    if (activeKeys.has('k') || activeKeys.has('arrowup')) delta -= SCROLL_SPEED;

    if (delta !== 0) ctrl.scrollRow(delta);

    ctrl.update(null);
    rafId = requestAnimationFrame(loop);
  }

  function handleKeydown(e) {
    if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;
    const key = e.key.toLowerCase();
    if (['j', 'k', 'arrowdown', 'arrowup'].includes(key)) {
      e.preventDefault();
      activeKeys.add(key);
    }
  }

  function handleKeyup(e) {
    const key = e.key.toLowerCase();
    if (activeKeys.has(key)) activeKeys.delete(key);
  }

  function handleBlur() {
    activeKeys.clear();
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

  $effect(() => {
    const _v = library.viewVersion;
    ctrl.resetScroll();
  });

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    window.addEventListener("keyup", handleKeyup);
    window.addEventListener("blur", handleBlur);
    loop();
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
    window.removeEventListener("keyup", handleKeyup);
    window.removeEventListener("blur", handleBlur);
    if (rafId) cancelAnimationFrame(rafId);
  });
</script>

<div class="album-grid-viewport">
  <div 
    class="grid-container"
    bind:clientWidth={ctrl.layout.containerWidth} 
    bind:clientHeight={ctrl.viewportHeight}
    onwheel={(e) => { 
      e.preventDefault(); 
      ctrl.handleWheel(e); 
    }}
  >
    <div 
      class="scroll-content" 
      style="
        height: {ctrl.contentHeight}px; 
        background-color: var(--background-main);
        transform: translate3d(0, -{ctrl.scroll.currentY}px, 0);
        will-change: transform;
      "
    >
      {#each ctrl.virtualRows as row (row.index)}
        <div 
          class="row" 
          style="
            transform: translateY({row.y}px); 
            width: {ctrl.layout.gridWidth}px; 
            height: {ctrl.layout.rowHeight}px;
            z-index: {row.isExpandedRow ? 20 : 1};
          "
        >
          <div class="row-inner" style="gap: var(--gap-x);">
              {#each row.data as album (album.id)}
                <Album 
                  {album} 
                  active={library.expandedAlbumId === album.id}
                  onclick={() => ctrl.toggleAlbum(album.id)} 
                  scrollY={ctrl.scroll.currentY}
                  rowY={row.y}
                />
              {/each}
          </div>

          {#if row.isExpandedRow && ctrl.drawerInfo && row.data.find(a => a.id === library.expandedAlbumId)}
            <div class="drawer-plane" style="top: {ctrl.layout.rowHeight}px;">
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
      contain: content;
    }

    .scroll-content {
      width: 100%;
      position: absolute;
      top: 0;
      left: 0;
      pointer-events: auto;
    }
    
    .row {
        position: absolute;
        margin: 0 auto;
        right: 0;
        left: 0;
        display: flex;
        flex-direction: column;
        overflow: visible; 
        will-change: transform;
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
    }
</style>
