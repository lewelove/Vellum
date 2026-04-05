import { LayoutManager } from "./Layout.svelte.js";
import { ScrollEngine } from "./Scroll.svelte.js";
import { library } from "../../../library.svelte.js";

export class GridController {
  layout = new LayoutManager();
  scroll = new ScrollEngine();
  viewportHeight = $state(0);

  allRows = $derived(this.layout.chunk(library.albums));
  
  visibleRows = $derived(Math.ceil(this.viewportHeight / this.layout.rowHeight));

  maxSlots = $derived(Math.max(0, (this.allRows.length + 1 - this.visibleRows)));

  contentHeight = $derived(
    this.layout.getTotalHeight(this.allRows.length) + this.layout.rowHeight
  );

  virtualRows = $derived.by(() => {
    const { start, end } = this.layout.getVisibleIndices(
      this.scroll.currentY, 
      this.viewportHeight, 
      this.allRows.length
    );

    const indicesToRender = [];
    for (let i = start; i <= end; i++) {
      indicesToRender.push(i);
    }

    return indicesToRender.map(i => ({
      index: i,
      y: this.layout.getRowY(i),
      data: this.allRows[i]
    }));
  });

  update(mainEl, dpr = 1) {
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
}
