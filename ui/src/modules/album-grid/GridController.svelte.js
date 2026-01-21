import { LayoutManager } from "./layout.svelte.js";
import { ScrollEngine } from "./scroll.svelte.js";
import { library } from "$state/library.svelte.js";

export class GridController {
  layout = new LayoutManager();
  scroll = new ScrollEngine();
  viewportHeight = $state(0);

  rows = $derived(this.layout.chunk(library.albums));
  
  drawerInfo = $derived.by(() => {
    if (!library.expandedAlbumId) return null;
    const album = library.albums.find(a => a.id === library.expandedAlbumId);
    
    // Use the actual tracks array length from the DTO.
    const count = album ? album.tracks.length : 0;
    
    // Use Natural Height calculation (no quantization)
    return album ? this.layout.getNaturalDrawer(count) : null;
  });

  // Calculate total Pixel Height of the content
  contentHeight = $derived(
    this.layout.getContentHeight(this.rows.length) + 
    (this.drawerInfo ? this.drawerInfo.height : 0)
  );
  
  // Calculate Virtual "Slots" for the Scroll Engine
  // We represent the drawer as fractional rows for scrolling limits
  drawerRows = $derived(this.drawerInfo ? (this.drawerInfo.height / this.layout.rowHeight) : 0);
  totalRowsCount = $derived(this.rows.length + this.drawerRows);

  visibleRows = $derived(Math.ceil(this.viewportHeight / this.layout.rowHeight));
  maxSlots = $derived(Math.max(0, (this.totalRowsCount + 1 - this.visibleRows)));

  update(mainEl) {
    this.scroll.update(this.layout.rowHeight);
    if (mainEl) {
      // Direct binding of scroll engine Y to element scrollTop
      // Note: Because of non-quantized drawer, this might feel slightly "loose" 
      // around the drawer area, but ensures 1:1 mechanics elsewhere.
      mainEl.scrollTop = Math.round(this.scroll.currentY);
    }
  }

  handleWheel(e) {
    this.scroll.handleWheel(e, this.maxSlots);
  }
}
