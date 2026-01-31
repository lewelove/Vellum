import { theme } from "../../theme.svelte.js";

export class LayoutManager {
  containerWidth = $state(0);

  // Core dimensions derived from theme
  gapX = $derived(theme.albumGrid["gap-x"] ?? 24);
  gapY = $derived(theme.albumGrid["gap-y"] ?? 12);
  cardSize = $derived(theme.albumGrid["cover-size"] ?? 200);
  
  // The height of a single row of albums
  rowHeight = $derived(
    this.gapY +       
    this.cardSize +     
    (theme.albumGrid["text-gap-main"] ?? 8) +  
    (theme.albumGrid["font-line-height-title"] ?? 18) +     
    (theme.albumGrid["text-gap-lesser"] ?? 2) +
    (theme.albumGrid["font-line-height-artist"] ?? 16)
  );

  // Grid logic
  cols = $derived(Math.max(1, Math.floor((this.containerWidth - 40 + this.gapX) / (this.cardSize + this.gapX))));
  gridWidth = $derived((this.cols * this.cardSize) + ((this.cols - 1) * this.gapX));

  /**
   * Calculates drawer dimensions. 
   * To achieve perfect symmetry, the cover size must account for:
   * (Total Height) - (Chevron Overhead) - (Top/Bottom Paddings) - (Top/Bottom Borders)
   */
  getNaturalDrawer() {
    const totalDrawerHeight = this.rowHeight * 2;
    const overhead = (theme.albumGrid["drawer-gap-main"] ?? 0) + (theme.albumGrid["drawer-chevron-height"] ?? 12);
    const paddingY = theme.drawer["drawer-padding-y"] ?? 18;
    const paddingX = theme.drawer["drawer-padding-x"] ?? 18;
    const borderWeight = 2; // 1px top + 1px bottom border
    
    const bandCHeight = totalDrawerHeight - overhead;
    
    // Dynamic Cover Size: content height - borders - vertical paddings
    const dynamicCoverSize = Math.max(100, bandCHeight - borderWeight - (paddingY * 2));
    
    // Calculate track width based on the dynamic cover
    const availableTotalWidth = Math.max(0, this.containerWidth - (paddingX * 2));
    const layoutWidth = Math.min(availableTotalWidth, theme.drawer["drawer-contents-x-max"] ?? 1600);
    const drawerTrackWidth = layoutWidth - dynamicCoverSize - (theme.drawer["drawer-split-gap"] ?? 24);

    return {
      height: totalDrawerHeight,
      bandA: theme.albumGrid["drawer-gap-main"] ?? 0,
      bandB: theme.albumGrid["drawer-chevron-height"] ?? 12,
      trackCols: drawerTrackWidth > 550 ? 2 : 1,
      chevronWidth: theme.albumGrid["drawer-chevron-width"] ?? 24,
      bandCHeight: bandCHeight,
      drawerCoverSize: dynamicCoverSize
    };
  }

  getTotalHeight(rowCount, drawerInfo) {
    const topOffset = Math.max(0, this.gapX - this.gapY);
    return (rowCount * this.rowHeight) + (drawerInfo ? drawerInfo.height : 0) + topOffset;
  }

  getRowY(index, expandedRowIndex, drawerHeight) {
    const topOffset = Math.max(0, this.gapX - this.gapY);
    let y = (index * this.rowHeight) + topOffset;
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
      end: Math.min(Math.max(0, rowCount - 1), end)
    };
  }

  chunk(arr) {
    const results = [];
    const columns = this.cols;
    for (let i = 0; i < arr.length; i += columns) {
      results.push(arr.slice(i, i + columns));
    }
    return results;
  }
}
