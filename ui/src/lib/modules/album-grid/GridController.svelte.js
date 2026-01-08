import { LayoutManager } from "./layout.svelte.js";
import { ScrollEngine } from "./scroll.svelte.js";
import { library } from "$lib/state/library.svelte.js";

export class GridController {
  layout = new LayoutManager();
  scroll = new ScrollEngine();
  viewportHeight = $state(0);

  rows = $derived(this.layout.chunk(library.albums));
  
  drawerInfo = $derived.by(() => {
    if (!library.expandedAlbumId) return null;
    const album = library.albums.find(a => a.id === library.expandedAlbumId);
    return album ? this.layout.getQuantizedDrawer(album.tracks.length) : null;
  });

  totalRowsCount = $derived(this.rows.length + (this.drawerInfo ? this.drawerInfo.rows : 0));
  contentHeight = $derived(this.layout.getContentHeight(this.totalRowsCount));
  visibleRows = $derived(Math.ceil(this.viewportHeight / this.layout.rowHeight));
  maxSlots = $derived(Math.max(0, (this.totalRowsCount + 1 - this.visibleRows)));

  update(mainEl) {
    this.scroll.update(this.layout.rowHeight);
    if (mainEl) {
      mainEl.scrollTop = Math.round(this.scroll.currentY);
    }
  }

  handleWheel(e) {
    this.scroll.handleWheel(e, this.maxSlots);
  }
}
