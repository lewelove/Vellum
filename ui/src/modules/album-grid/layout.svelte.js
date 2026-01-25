import { theme } from "../../theme.svelte.js";

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
    theme.albumGrid["font-line-height-title"] +     
    theme.albumGrid["text-gap-lesser"] +
    theme.albumGrid["font-line-height-artist"]
  );

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gapX) / (this.cardSize + this.gapX)) || 1);
  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX)));

  drawerTrackWidth = $derived(
    this.containerWidth 
    - (theme.drawer["drawer-padding-x"] * 2) 
    - theme.drawer["drawer-cover-size"] 
    - theme.drawer["drawer-split-gap"]
  );

  trackCols = $derived(this.drawerTrackWidth > 550 ? 2 : 1);

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
  }

  getNaturalDrawer(trackCount) {
    const dSettings = theme.drawer;
    const chevronHeight = theme.albumGrid["drawer-chevron-height"];
    const gapMain = theme.albumGrid["drawer-gap-main"]; 
    
    const bandA = gapMain; 
    const bandB = chevronHeight; 
    const overhead = bandA + bandB;
    
    const paddingTotal = dSettings["drawer-padding-y"] * 2;
    
    const leftColHeight = dSettings["drawer-cover-size"];

    const titleH = dSettings["drawer-font-size-album"] * 1.3;
    const artistH = dSettings["drawer-font-size-artist"] * 1.3;
    const headerGap = 24; 
    const headerBlock = titleH + artistH + headerGap;

    const tracksPerCol = Math.ceil(trackCount / this.trackCols);
    const tracksBlock = tracksPerCol * dSettings["drawer-track-y"];
    
    const rightColHeight = headerBlock + tracksBlock;
    
    // +2px accounts for the 1px top/bottom borders on the drawer content box
    const naturalContentHeight = paddingTotal + Math.max(leftColHeight, rightColHeight) + 2;
    
    const totalHeight = overhead + naturalContentHeight;
    
    return {
      height: totalHeight,
      bandA,
      bandB,
      trackCols: this.trackCols,
      chevronWidth: theme.albumGrid["drawer-chevron-width"],
      bandCHeight: naturalContentHeight,
      drawerCoverSize: dSettings["drawer-cover-size"]
    };
  }

  getTotalHeight(rowCount, drawerInfo) {
    return (rowCount * this.rowHeight) + (drawerInfo ? drawerInfo.height : 0) + this.topOffset;
  }

  getRowY(index, expandedRowIndex, drawerHeight) {
    let y = (index * this.rowHeight) + this.topOffset;
    if (expandedRowIndex !== -1 && index > expandedRowIndex) {
      y += drawerHeight;
    }
    return y;
  }

  /**
   * Calculates which row indices are currently visible in the viewport.
   * Includes a buffer to handle inertia/fast scrolling.
   */
  getVisibleIndices(scrollY, viewportHeight, rowCount) {
    const buffer = 4;
    // Basic projection based on fixed row height
    // (Actual positions might vary due to drawer, but this covers the scan area)
    const start = Math.floor(scrollY / this.rowHeight) - buffer;
    const end = Math.ceil((scrollY + viewportHeight) / this.rowHeight) + buffer;
    
    return {
      start: Math.max(0, start),
      end: Math.min(rowCount - 1, end)
    };
  }
}
