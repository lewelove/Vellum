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
    return album ? this.layout.getNaturalDrawer() : null;
  });

  drawerHeight = $derived(this.drawerInfo ? this.drawerInfo.height : 0);

  drawerRows = $derived(this.expandedRowIndex === -1 ? 0 : 2);
  
  totalRowsCount = $derived(this.allRows.length + this.drawerRows);

  visibleRows = $derived(Math.ceil(this.viewportHeight / this.layout.rowHeight));

  maxSlots = $derived(Math.max(0, (this.totalRowsCount + 1 - this.visibleRows)));

  contentHeight = $derived(
    this.layout.getTotalHeight(this.allRows.length, this.drawerInfo) + this.layout.rowHeight
  );

  virtualRows = $derived.by(() => {
    const { start, end } = this.layout.getVisibleIndices(
      this.scroll.currentY, 
      this.viewportHeight, 
      this.allRows.length
    );

    const indicesToRender = new Set();
    for (let i = start; i <= end; i++) {
      indicesToRender.add(i);
    }

    if (this.expandedRowIndex !== -1) {
      indicesToRender.add(this.expandedRowIndex);
    }

    return Array.from(indicesToRender).map(i => ({
      index: i,
      y: this.layout.getRowY(i, this.expandedRowIndex, this.drawerHeight),
      data: this.allRows[i],
      isExpandedRow: (i === this.expandedRowIndex)
    }));
  });

  update(mainEl) {
    const dpr = window.devicePixelRatio || 1;
    this.scroll.update(this.layout.rowHeight, dpr);
    
    if (mainEl) {
      mainEl.scrollTop = this.scroll.currentY;
    }
  }

  handleWheel(e) {
    this.scroll.handleWheel(e, this.maxSlots);
  }

  scrollRow(delta) {
    const newSlot = this.scroll.targetSlot + delta;
    this.scroll.targetSlot = Math.max(0, Math.min(newSlot, this.maxSlots));
  }

  resetScroll() {
    this.scroll.syncToSlot(0);
    this.scroll.currentY = 0;
  }

  toggleAlbum(id) {
    const flatIndex = library.albums.findIndex(a => a.id === id);
    if (flatIndex === -1) return;

    const targetRowIdx = Math.floor(flatIndex / this.layout.cols);
    const oldY = this.layout.getRowY(targetRowIdx, this.expandedRowIndex, this.drawerHeight);

    library.toggleExpand(id);

    const newY = this.layout.getRowY(targetRowIdx, this.expandedRowIndex, this.drawerHeight);
    const deltaY = newY - oldY;

    if (deltaY !== 0) {
      this.scroll.shiftPosition(deltaY, this.layout.rowHeight, this.maxSlots);
    }
  }
}
