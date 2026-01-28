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

  drawerTrackWidth = $derived.by(() => {
    const availableTotalWidth = this.containerWidth - (theme.drawer["drawer-padding-x"] * 2);
    const layoutWidth = Math.min(availableTotalWidth, theme.drawer["drawer-contents-x-max"]);
    
    return layoutWidth 
      - theme.drawer["drawer-cover-size"] 
      - theme.drawer["drawer-split-gap"];
  });

  trackCols = $derived(this.drawerTrackWidth > 550 ? 2 : 1);

  chunk(arr) {
    const results = [];
    for (let i = 0; i < arr.length; i += this.cols) {
      results.push(arr.slice(i, i + this.cols));
    }
    return results;
  }

  getNaturalDrawer() {
    const chevronHeight = theme.albumGrid["drawer-chevron-height"];
    const gapMain = theme.albumGrid["drawer-gap-main"]; 
    
    const bandA = gapMain; 
    const bandB = chevronHeight; 
    const overhead = bandA + bandB;
    
    const totalHeight = this.rowHeight * 2;
    const naturalContentHeight = totalHeight - overhead;
    
    return {
      height: totalHeight,
      bandA,
      bandB,
      trackCols: this.trackCols,
      chevronWidth: theme.albumGrid["drawer-chevron-width"],
      bandCHeight: naturalContentHeight,
      drawerCoverSize: theme.drawer["drawer-cover-size"]
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

  getVisibleIndices(scrollY, viewportHeight, rowCount) {
    const buffer = 4;
    const start = Math.floor(scrollY / this.rowHeight) - buffer;
    const end = Math.ceil((scrollY + viewportHeight) / this.rowHeight) + buffer;
    
    return {
      start: Math.max(0, start),
      end: Math.min(rowCount - 1, end)
    };
  }
}
