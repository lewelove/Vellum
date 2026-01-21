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
    theme.albumGrid["font-line-height-title"] +     
    theme.albumGrid["text-gap-lesser"] +
    theme.albumGrid["font-line-height-artist"]
  );

  cols = $derived(Math.floor((Math.max(0, this.containerWidth - 40) + this.gapX) / (this.cardSize + this.gapX)) || 1);
  gridWidth = $derived(Math.floor((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX)));

  // --- DRAWER LAYOUT ENGINE ---
  
  // Available width for the content column (Text + Tracks)
  drawerTrackWidth = $derived(
    this.containerWidth 
    - (theme.drawer["drawer-padding-x"] * 2) 
    - theme.drawer["drawer-cover-size"] 
    - theme.drawer["drawer-split-gap"]
  );

  // Dynamic column calculation for the tracklist section
  // If we have enough space, split tracks into columns
  trackCols = $derived(this.drawerTrackWidth > 550 ? 2 : 1);

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

  getNaturalDrawer(trackCount) {
    const dSettings = theme.drawer;
    const chevronHeight = theme.albumGrid["drawer-chevron-height"];
    const gapMain = theme.albumGrid["drawer-gap-main"]; 
    
    // Spacers (Band A + Band B)
    // Band A: Gap between grid row and drawer start
    // Band B: Chevron area
    const bandA = gapMain; 
    const bandB = chevronHeight; 
    const overhead = bandA + bandB;
    
    const paddingTotal = dSettings["drawer-padding-y"] * 2;
    
    // 1. Calculate Left Column Height (Fixed Cover)
    const leftColHeight = dSettings["drawer-cover-size"];

    // 2. Calculate Right Column Height (Header + Tracks)
    // Approximate Header Block
    const titleH = dSettings["drawer-font-size-album"] * 1.3;
    const artistH = dSettings["drawer-font-size-artist"] * 1.3;
    const headerGap = 24; 
    const headerBlock = titleH + artistH + headerGap;

    const tracksPerCol = Math.ceil(trackCount / this.trackCols);
    const tracksBlock = tracksPerCol * dSettings["drawer-track-y"];
    
    const rightColHeight = headerBlock + tracksBlock;
    
    // 3. Natural Height (Max of cols + Padding)
    const naturalContentHeight = paddingTotal + Math.max(leftColHeight, rightColHeight);
    
    const totalHeight = overhead + naturalContentHeight;
    
    return {
      height: totalHeight,
      bandA,
      bandB,
      trackCols: this.trackCols,
      chevronWidth: theme.albumGrid["drawer-chevron-width"],
      bandCHeight: naturalContentHeight
    };
  }
}
