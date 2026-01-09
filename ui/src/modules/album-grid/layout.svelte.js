import { theme } from "$core/theme.svelte.js";

export class LayoutManager {
  containerWidth = $state(0);

  gapX = $derived(theme.albumGrid["gap-x"]);
  gapY = $derived(theme.albumGrid["gap-y"]);
  topOffset = $derived(Math.max(0, this.gapX - this.gapY));
  creaseHeight = $derived(theme.albumGrid["crease-height"]);
  cardSize = $derived(theme.albumGrid["cover-size"]);
  
  rowHeight = $derived(
    theme.albumGrid["gap-y"] +       
    theme.albumGrid["cover-size"] +     
    theme.albumGrid["text-gap-main"] +  
    theme.typography["font-size-title"] +     
    theme.albumGrid["text-gap-lesser"] +
    theme.typography["font-size-artist"]
  );

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gapX) / (this.cardSize + this.gapX)) || 1);
  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX)));

  // Determine how many columns to use for the track list inside the drawer
  trackCols = $derived(this.containerWidth > 800 ? 2 : 1);

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
    const chevronHeight = theme.albumGrid["drawer-chevron-height"];
    const gapMain = theme.albumGrid["drawer-gap-main"]; 
    const dSettings = theme.drawer;

    const bandA = gapMain; 
    const bandB = chevronHeight; 
    const overhead = bandA + bandB;
    
    const paddingTotal = dSettings["drawer-padding-y"] * 2;
    const headerTotal = dSettings["drawer-font-size-album"] + dSettings["drawer-font-size-artist"] + 24; 
    
    // Distribution Logic: Calculate vertical height based on track columns
    const tracksPerCol = Math.ceil(trackCount / this.trackCols);
    const tracksTotalHeight = tracksPerCol * dSettings["drawer-track-y"];
    
    const naturalContentHeight = paddingTotal + headerTotal + tracksTotalHeight; 
    
    const totalRequired = overhead + naturalContentHeight;
    const virtualRows = Math.ceil(totalRequired / this.rowHeight);
    const totalHeight = virtualRows * this.rowHeight;
    
    return {
      height: totalHeight,
      rows: virtualRows,
      bandA,
      bandB,
      trackCols: this.trackCols,
      chevronWidth: theme.albumGrid["drawer-chevron-width"],
      bandCHeight: totalHeight - overhead
    };
  }
}
