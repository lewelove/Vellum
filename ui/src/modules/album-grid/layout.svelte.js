import { theme } from "$core/theme.svelte.js";

export class LayoutManager {
  containerWidth = $state(0);

  gapX = $derived(theme["album-grid"]["gap-x"]);
  gapY = $derived(theme["album-grid"]["gap-y"]);
  topOffset = $derived(Math.max(0, this.gapX - this.gapY));
  creaseHeight = $derived(theme["album-grid"]["crease-height"]);
  cardSize = $derived(theme["album-grid"]["cover-size"]);
  
  rowHeight = $derived(
    theme["album-grid"]["gap-y"] +       
    theme["album-grid"]["cover-size"] +     
    theme["album-grid"]["text-gap-main"] +  
    theme.typography["font-size-title"] +     
    theme["album-grid"]["text-gap-lesser"] +
    theme.typography["font-size-artist"]
  );

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gapX) / (this.cardSize + this.gapX)) || 1);
  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX)));

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
  }

  getContentHeight(totalRowsCount) {
    return (totalRowsCount * this.rowHeight) + this.topOffset;
  }

  getQuantizedDrawer(trackCount) {
    const chevronHeight = theme["album-grid"]["drawer-chevron-height"];
    const bandA = this.gapY; 
    const bandB = chevronHeight; 
    const overhead = bandA + bandB;
    
    // Band C: Content area (Header + Tracks + Padding)
    const naturalContentHeight = 100 + (trackCount * 40) + 40; 
    
    const totalRequired = overhead + naturalContentHeight;
    const virtualRows = Math.ceil(totalRequired / this.rowHeight);
    const totalHeight = virtualRows * this.rowHeight;
    
    return {
      height: totalHeight,
      rows: virtualRows,
      bandA,
      bandB,
      bandCHeight: totalHeight - overhead
    };
  }
}
