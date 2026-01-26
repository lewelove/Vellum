import { LayoutManager } from "./layout.svelte.js";
import { ScrollEngine } from "./scroll.svelte.js";
import { library } from "../../library.svelte.js";

export class GridController {
  layout = new LayoutManager();
  scroll = new ScrollEngine();
  viewportHeight = $state(0);

  allRows = $derived(this.layout.chunk(library.albums));

  expandedRowIndex = $derived.by(() => {
    if (!library.expandedAlbumId) return -1;
    const flatIndex = library.albums.findIndex(a => a.id === library.expandedAlbumId);
    if (flatIndex === -1) return -1;
    return Math.floor(flatIndex / this.layout.cols);
  });

  drawerInfo = $derived.by(() => {
    if (this.expandedRowIndex === -1) return null;
    const album = library.albums.find(a => a.id === library.expandedAlbumId);
    const count = album ? album.tracks.length : 0;
    return album ? this.layout.getNaturalDrawer(count) : null;
  });

  drawerHeight = $derived(this.drawerInfo ? this.drawerInfo.height : 0);

  // Convert drawer pixels to "row units" to integrate with slot-based logic
  drawerRows = $derived(this.drawerHeight / this.layout.rowHeight);
  
  // Total virtual rows (Data rows + Drawer space)
  totalRowsCount = $derived(this.allRows.length + this.drawerRows);

  // How many rows fit in viewport?
  visibleRows = $derived(Math.ceil(this.viewportHeight / this.layout.rowHeight));

  // Clamp max slot: Stop scrolling when the last item hits the bottom of the viewport
  maxSlots = $derived(Math.max(0, (this.totalRowsCount + 1 - this.visibleRows)));

  // Includes the "+1" virtual row buffer at the end
  contentHeight = $derived(
    this.layout.getTotalHeight(this.allRows.length, this.drawerInfo) + this.layout.rowHeight
  );

  virtualRows = $derived.by(() => {
    // 1. Determine standard visible range
    const { start, end } = this.layout.getVisibleIndices(
      this.scroll.currentY, 
      this.viewportHeight, 
      this.allRows.length
    );

    // 2. Build a set of indices to render
    const indicesToRender = new Set();
    for (let i = start; i <= end; i++) {
      indicesToRender.add(i);
    }

    // 3. CRITICAL: Force include the expanded row index so it never unloads
    if (this.expandedRowIndex !== -1) {
      indicesToRender.add(this.expandedRowIndex);
    }

    // 4. Map to row objects
    return Array.from(indicesToRender).map(i => ({
      index: i,
      y: this.layout.getRowY(i, this.expandedRowIndex, this.drawerHeight),
      data: this.allRows[i],
      isExpandedRow: (i === this.expandedRowIndex)
    }));
  });

  update(mainEl) {
    // Inject DPR for hybrid pixel snapping (Sub-pixel motion -> Pixel-perfect rest)
    const dpr = window.devicePixelRatio || 1;
    
    // ScrollEngine updates currentY based on targetSlot * rowHeight
    this.scroll.update(this.layout.rowHeight, dpr);
    
    if (mainEl) {
      mainEl.scrollTop = this.scroll.currentY;
    }
  }

  handleWheel(e) {
    this.scroll.handleWheel(e, this.maxSlots);
  }

  // Accepts float values for smooth continuous scrolling
  scrollRow(delta) {
    const newSlot = this.scroll.targetSlot + delta;
    this.scroll.targetSlot = Math.max(0, Math.min(newSlot, this.maxSlots));
  }

  resetScroll() {
    this.scroll.syncToSlot(0);
    this.scroll.currentY = 0;
  }
}
