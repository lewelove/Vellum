<script>
  import { onMount, onDestroy } from "svelte";
  import { library } from "../../../library.svelte.js";
  import { GridController } from "./GridController.svelte.js";
  
  import Album from "./Album.svelte";

  const ctrl = new GridController();
  let rafId;
  let dpr = $state(1);

  const activeKeys = new Set();
  const SCROLL_SPEED = 0.20;

  let renderY = $derived(ctrl.scroll.currentY);

  function loop() {
    let delta = 0;
    if (activeKeys.has('j') || activeKeys.has('arrowdown')) delta += SCROLL_SPEED;
    if (activeKeys.has('k') || activeKeys.has('arrowup')) delta -= SCROLL_SPEED;

    if (delta !== 0) ctrl.scrollRow(delta);

    ctrl.update(null, dpr);
    rafId = requestAnimationFrame(loop);
  }

  function handleKeydown(e) {
    if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;
    if (library.focusedAlbum) return;

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
    dpr = window.devicePixelRatio || 1;
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
      if (library.focusedAlbum) return;
      e.preventDefault(); 
      ctrl.handleWheel(e); 
    }}
  >
    <div 
      class="scroll-content" 
      style="
        height: {ctrl.contentHeight}px; 
        background-color: var(--background-main);
        transform: translate3d(0, -{renderY}px, 0);
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
          "
        >
          <div class="row-inner" style="gap: var(--gap-x);">
              {#each row.data as album (album.id)}
                <Album 
                  {album} 
                  active={library.focusedAlbum?.id === album.id}
                  onclick={() => library.setFocus(album)} 
                  scrollY={renderY}
                  rowY={row.y}
                />
              {/each}
          </div>
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
      backface-visibility: hidden;
      transform-style: preserve-3d;
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
        backface-visibility: hidden;
    }
    
    .row-inner {
        display: flex;
        justify-content: flex-start;
        height: 100%;
    }
</style>
