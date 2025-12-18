<script>
import { onMount, tick, onDestroy } from "svelte";
  import { ScrollEngine } from "$lib/engines/scroll.svelte.js";
  import { LayoutManager } from "$lib/engines/layout.svelte.js";
  import { getLibrary } from "$lib/api.js";
  
  import Album from "$lib/components/Album.svelte";
  import Drawer from "$lib/components/Drawer.svelte";
  import Scrollbar from "$lib/components/Scrollbar.svelte";

  // State & Engines
  let albums = $state([]);
  let expandedAlbumId = $state(null);
  let viewportHeight = $state(0);
  let contentHeight = $state(0);
  
  const scroll = new ScrollEngine();
  const layout = new LayoutManager();

  // DOM Refs
  let contentEl;
  let rowElements = [];
  let rafId;

  // Derived Layout
  let rows = $derived(layout.chunk(albums));

  function loop() {
    scroll.update(rowElements);
    if (contentEl) {
      contentEl.style.transform = `translate3d(0, -${scroll.currentY}px, 0)`;
    }
    rafId = requestAnimationFrame(loop);
  }

  async function toggleAlbum(id) {
    expandedAlbumId = expandedAlbumId === id ? null : id;
    const rowIndex = rows.findIndex(row => row.find(a => a.id === id));
    if (rowIndex !== -1) scroll.targetIndex = rowIndex;
    
    await tick();
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
  bind:clientWidth={layout.containerWidth} 
  bind:clientHeight={viewportHeight}
  onwheel={(e) => { e.preventDefault(); scroll.handleWheel(e, rows.length); }}
>
  <Scrollbar {viewportHeight} {contentHeight} currentY={scroll.currentY} />

  <div class="scroll-content" bind:this={contentEl}>
    {#each rows as row, i}
      <div 
        class="row" 
        style="width: {layout.gridWidth}px; gap: {layout.gap}px; margin-bottom: {layout.gap}px"
        bind:this={rowElements[i]}
      >
        {#each row as album}
          <Album 
            {album} 
            size={layout.cardSize} 
            active={expandedAlbumId === album.id}
            onclick={() => toggleAlbum(album.id)} 
          />
        {/each}
      </div>

      {#if row.find(a => a.id === expandedAlbumId)}
        <Drawer 
          activeAlbum={row.find(a => a.id === expandedAlbumId)} 
          width={layout.gridWidth} 
        />
      {/if}
    {/each}
  </div>
</main>
